//! The batched sim camera.
//!
//! One irys scene holds every environment: environment `i` is a copy of the
//! street and one K1, offset by `i × TILE` along y so that no head camera
//! can see into a neighbour's tile through the wall behind the lawn. Street
//! and K1 meshes are bottom-level structures shared by every copy; each
//! copy is only top-level instances.
//!
//! Depth uses irys's cheapest path: one sample per pixel, one bounce, the
//! megakernel's debug view 3, which writes the first hit's ray distance and
//! stops (primary rays only, hardware ray queries, no shading, no NEE).
//! RGB is the full OpenPBR path tracer at `spp` samples.

use crate::camera::{Eye, HeadCamera};
use crate::k1::{K1Model, K1Pose};
use crate::street::Street;
use crate::xf::Xf;
use crate::Error;
use irys_diff::look::scene::{mask, Camera, Instance, LookScene, Mesh};
use irys_diff::look::scene_gpu::{GpuLook, Opts};
use std::time::Instant;

/// Spacing between environment tiles along y, m.
pub const TILE: f64 = 40.0;

/// Metric depth (z along the optical axis), m; 0 where nothing was hit.
#[derive(Debug, Clone)]
pub struct DepthImage {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// Row-major, top row first.
    pub z: Vec<f32>,
}

/// sRGB, 8 bits per channel.
#[derive(Debug, Clone)]
pub struct RgbImage {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// Row-major RGB, top row first.
    pub rgb: Vec<u8>,
}

/// Batched head cameras over N environments.
pub struct SimCamera {
    /// The camera model.
    pub cam: HeadCamera,
    k1: K1Model,
    scene: LookScene,
    look: GpuLook,
    street_meshes: usize,
    street_instances: usize,
    poses: Vec<K1Pose>,
    /// Seconds the last scene upload (BLAS + TLAS build) took.
    pub upload_seconds: f64,
    /// Seconds the last render spent in its dispatch loop (irys `Frame::seconds`),
    /// i.e. without the clear, the readback and the CPU conversion.
    pub gpu_seconds: std::cell::Cell<f64>,
}

fn tile(i: usize) -> Xf {
    Xf::translation([0.0, i as f64 * TILE, 0.0])
}

fn linear_to_srgb8(x: f32) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let s = if x <= 0.003_130_8 { 12.92 * x } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0 + 0.5) as u8
}

impl SimCamera {
    /// Build the scene for `poses.len()` environments; poses are in each
    /// environment's own street frame.
    pub fn new(k1: K1Model, street: &Street, cam: HeadCamera, poses: Vec<K1Pose>) -> Result<Self, Error> {
        let mut meshes: Vec<Mesh> = street.meshes.clone();
        let mut materials = street.materials.clone();
        let street_meshes = meshes.len();
        for m in &k1.meshes {
            let mi = materials.len();
            materials.push(irys_diff::look::scene::Material::new(
                "k1",
                irys_diff::look::openpbr::OpenPbr {
                    base_color: m.rgb.map(|c| c * c), // URDF colours are authored as sRGB-ish
                    specular_roughness: 0.35,
                    ..Default::default()
                },
            ));
            meshes.push(Mesh {
                positions: m.tris.positions.clone(),
                normals: Vec::new(),
                uvs: Vec::new(),
                indices: m.tris.indices.clone(),
                material: mi,
            });
        }
        let mut instances = Vec::new();
        for i in 0..poses.len() {
            for mesh in 0..street_meshes {
                instances.push(Instance { mesh, xf: tile(i).to_irys(), mask: mask::GEOMETRY });
            }
        }
        let street_instances = instances.len();
        let dummy = Camera { eye: [0.0; 3], forward: [1.0, 0.0, 0.0], right: [0.0, -1.0, 0.0], up: [0.0, 0.0, 1.0], tan_half: 1.0 };
        let scene = LookScene {
            meshes,
            instances,
            materials,
            textures: Vec::new(),
            env: Some(street.env.clone()),
            sun: Some(street.sun),
            portal_skies: Vec::new(),
            camera: dummy,
            size: (cam.width as usize, cam.height as usize),
            frames: Vec::new(),
            fps: cam.rgb_hz,
            dropped: Vec::new(),
        };
        let ctx = irys_diff::gpu::context().map_err(|e| Error::Gpu(format!("{e:?}")))?;
        let t0 = Instant::now();
        // Placeholder upload so `look` exists; `set_poses` rebuilds with the K1s.
        let look = GpuLook::new(ctx, &scene).ok_or_else(|| Error::Gpu("no hardware ray queries on this device".into()))?;
        let mut s = Self {
            cam,
            k1,
            scene,
            look,
            street_meshes,
            street_instances,
            poses: Vec::new(),
            gpu_seconds: Default::default(),
            upload_seconds: t0.elapsed().as_secs_f64(),
        };
        s.set_poses(poses)?;
        Ok(s)
    }

    /// Number of environments.
    pub fn envs(&self) -> usize {
        self.poses.len()
    }

    /// The current poses.
    pub fn poses(&self) -> &[K1Pose] {
        &self.poses
    }

    /// The loaded K1.
    pub fn k1(&self) -> &K1Model {
        &self.k1
    }

    /// Move every K1. irys's `GpuLook` has no instance update yet, so this
    /// re-uploads the scene (all BLAS and the TLAS); `upload_seconds` says
    /// what that cost. The count of environments must not change.
    pub fn set_poses(&mut self, poses: Vec<K1Pose>) -> Result<(), Error> {
        if !self.poses.is_empty() && poses.len() != self.poses.len() {
            return Err(Error::Gpu("set_poses cannot change the number of environments".into()));
        }
        self.scene.instances.truncate(self.street_instances);
        for (i, p) in poses.iter().enumerate() {
            for (k, xf) in self.k1.mesh_poses(p).iter().enumerate() {
                self.scene.instances.push(Instance {
                    mesh: self.street_meshes + k,
                    xf: tile(i).then(xf).to_irys(),
                    mask: mask::GEOMETRY,
                });
            }
        }
        let ctx = irys_diff::gpu::context().map_err(|e| Error::Gpu(format!("{e:?}")))?;
        let t0 = Instant::now();
        self.look = GpuLook::new(ctx, &self.scene).ok_or_else(|| Error::Gpu("no hardware ray queries".into()))?;
        self.upload_seconds = t0.elapsed().as_secs_f64();
        self.poses = poses;
        Ok(())
    }

    /// The irys camera of environment `env`'s head, `eye` of the pair.
    pub fn camera(&self, env: usize, eye: Eye) -> Camera {
        let head = tile(env).then(&self.k1.head(&self.poses[env]));
        self.cam.pose(&head, eye)
    }

    /// Metric depth for one environment.
    pub fn depth(&self, env: usize, eye: Eye) -> DepthImage {
        let c = self.camera(env, eye);
        let opts = Opts { spp: 1, max_bounces: 1, debug: 3, seed: env as u64, ..Opts::default() };
        let f = self.look.render(&c, &opts);
        self.gpu_seconds.set(f.seconds);
        let (w, h) = (self.cam.width as usize, self.cam.height as usize);
        let aspect = w as f64 / h as f64;
        let mut z = Vec::with_capacity(w * h);
        for (i, px) in f.rgb.chunks_exact(3).enumerate() {
            // Debug view 3 is (transmission weight, front-facing, t / 20).
            // Opaque hits have x == 0 and y ∈ {0, 1}; a miss is shaded sky.
            let hit = px[0] == 0.0 && (px[1] == 0.0 || px[1] == 1.0) && px[2] > 0.0;
            let t = px[2] / 0.05;
            if !hit || t > self.cam.max_range * 1.8 {
                z.push(0.0);
                continue;
            }
            let (x, y) = ((i % w) as f64 + 0.5, (i / w) as f64 + 0.5);
            let sx = (2.0 * x / w as f64 - 1.0) * c.tan_half * aspect;
            let sy = (1.0 - 2.0 * y / h as f64) * c.tan_half;
            let cos = 1.0 / (1.0 + sx * sx + sy * sy).sqrt();
            let d = (t as f64 * cos) as f32;
            z.push(if d <= self.cam.max_range { d } else { 0.0 });
        }
        DepthImage { width: w as u32, height: h as u32, z }
    }

    /// Path-traced RGB for one environment, `spp` samples, `exposure` scale.
    pub fn rgb(&self, env: usize, eye: Eye, spp: u32, exposure: f32) -> RgbImage {
        let c = self.camera(env, eye);
        self.render_rgb_from_seeded(&c, spp, exposure, 0x5eed ^ env as u64)
    }

    /// Path-traced RGB from any camera in the scene (a chase view, a debug view).
    pub fn render_rgb_from(&self, c: &Camera, spp: u32, exposure: f32) -> RgbImage {
        self.render_rgb_from_seeded(c, spp, exposure, 0x5eed)
    }

    fn render_rgb_from_seeded(&self, c: &Camera, spp: u32, exposure: f32, seed: u64) -> RgbImage {
        let opts = Opts { spp, max_bounces: 4, seed, ..Opts::default() };
        let f = self.look.render(c, &opts);
        self.gpu_seconds.set(f.seconds);
        RgbImage {
            width: self.cam.width,
            height: self.cam.height,
            rgb: f.rgb.iter().map(|&v| linear_to_srgb8(v * exposure)).collect(),
        }
    }

    /// Depth for every environment.
    pub fn depth_batch(&self, eye: Eye) -> Vec<DepthImage> {
        (0..self.envs()).map(|e| self.depth(e, eye)).collect()
    }

    /// RGB for every environment.
    pub fn rgb_batch(&self, eye: Eye, spp: u32, exposure: f32) -> Vec<RgbImage> {
        (0..self.envs()).map(|e| self.rgb(e, eye, spp, exposure)).collect()
    }
}

impl DepthImage {
    /// A false-colour-free 8-bit visualisation: near bright, far dark, 0 black.
    pub fn to_gray8(&self, max: f32) -> Vec<u8> {
        self.z
            .iter()
            .map(|&d| if d <= 0.0 { 0 } else { (255.0 * (1.0 - (d / max).clamp(0.0, 1.0)).powf(1.5)) as u8 })
            .collect()
    }

    /// Back-project to a point cloud in the camera's optical frame
    /// (x right, y down, z forward), the form sim-bridge's LiDAR path wants.
    pub fn points(&self, cam: &HeadCamera) -> Vec<[f32; 3]> {
        let (w, h) = (self.width as usize, self.height as usize);
        let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
        let f = cam.f * w as f64 / cam.width as f64;
        let mut out = Vec::new();
        for (i, &z) in self.z.iter().enumerate() {
            if z <= 0.0 {
                continue;
            }
            let (u, v) = ((i % w) as f64 + 0.5, (i / w) as f64 + 0.5);
            let z = z as f64;
            out.push([((u - cx) * z / f) as f32, ((v - cy) * z / f) as f32, z as f32]);
        }
        out
    }
}
