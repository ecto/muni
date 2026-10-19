//! Rendered depth from irys, in-process (feature `irys`).
//!
//! Replaces the planned ZMQ hop to a Python/gsplat `sim-world`: one thread
//! owns the GPU and a `muni_irys::SimCamera` with one environment per rover.
//! Each rover asks for a scan at its pose; the thread poses that rover's K1,
//! renders its head camera's depth and returns it as LiDAR points in the
//! frame `LidarSim` uses (x forward, y left, z up, origin at the sensor
//! mount height above the rover's ground point).
//!
//! The rendered world is muni-irys's snowy street plus the scenario's
//! obstacles as concrete boxes; collisions still run against the parry
//! world, so the two agree on obstacles but the street itself is visual.

use crate::lidar_sim::LidarPoint;
use crate::world::World;
use muni_irys::street::{cuboid, mat, SIDEWALK_Z};
use muni_irys::{Eye, HeadCamera, K1Model, K1Pose, SimCamera, Street};
use std::collections::HashMap;
use std::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{error, info};

struct Request {
    rover: usize,
    pose: (f64, f64, f64),
    reply: oneshot::Sender<Vec<LidarPoint>>,
}

/// A handle to the render thread. Cheap to clone.
#[derive(Clone)]
pub struct IrysDepth {
    tx: mpsc::Sender<Request>,
}

/// Points kept per scan, about what the geometric LiDAR emits (360 × 8).
const MAX_POINTS: usize = 2880;

fn k1_pose(k1: &K1Model, (x, y, theta): (f64, f64, f64)) -> K1Pose {
    k1.standing(x, y, theta, 0.0, 0.0, HashMap::new(), SIDEWALK_Z)
}

impl IrysDepth {
    /// Start the render thread for `rovers` rovers at their spawn poses.
    /// Fails if the K1 assets or a ray-query GPU are missing.
    pub fn spawn(world: &World, spawns: Vec<(f64, f64, f64)>, mount_height: f32) -> anyhow::Result<Self> {
        let k1 = K1Model::load_default()?;
        let mut street = Street::snowy(80.0);
        for o in &world.obstacles {
            let (a, b) = (o.aabb.min, o.aabb.max);
            street.meshes.push(cuboid(
                [a.x as f64, a.y as f64, a.z as f64 + SIDEWALK_Z],
                [b.x as f64, b.y as f64, b.z as f64 + SIDEWALK_Z],
                0.0,
                mat::CONCRETE,
            ));
        }
        let cam = HeadCamera::K1;
        let poses = spawns.iter().map(|&p| k1_pose(&k1, p)).collect();
        let (tx, rx) = mpsc::channel::<Request>();
        let (ready_tx, ready_rx) = mpsc::channel();
        std::thread::Builder::new().name("irys-depth".into()).spawn(move || {
            let mut sim = match SimCamera::new(k1, &street, cam, poses) {
                Ok(s) => {
                    let _ = ready_tx.send(Ok(()));
                    s
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e.to_string()));
                    return;
                }
            };
            info!(rovers = sim.envs(), "irys depth ready");
            while let Ok(req) = rx.recv() {
                let mut poses = sim.poses().to_vec();
                poses[req.rover] = k1_pose(sim.k1(), req.pose);
                if let Err(e) = sim.set_poses(poses) {
                    error!(?e, "irys set_poses");
                    continue;
                }
                let depth = sim.depth(req.rover, Eye::Left);
                let c = sim.camera(req.rover, Eye::Left);
                let pts = depth.points(&sim.cam);
                let stride = pts.len().div_ceil(MAX_POINTS).max(1);
                let (x0, y0, th) = req.pose;
                let tile_y = req.rover as f64 * muni_irys::render::TILE;
                let (s, co) = th.sin_cos();
                let out = pts
                    .iter()
                    .step_by(stride)
                    .map(|p| {
                        let [px, py, pz] = [p[0] as f64, p[1] as f64, p[2] as f64];
                        let w: Vec<f64> = (0..3).map(|i| c.eye[i] + c.forward[i] * pz + c.right[i] * px - c.up[i] * py).collect();
                        let (dx, dy) = (w[0] - x0, w[1] - tile_y - y0);
                        let r = LidarPoint::new(
                            (co * dx + s * dy) as f32,
                            (-s * dx + co * dy) as f32,
                            (w[2] - SIDEWALK_Z) as f32 - mount_height,
                        );
                        let range = (r.x * r.x + r.y * r.y + r.z * r.z).sqrt();
                        r.with_intensity(((1.0 - range / sim.cam.max_range).clamp(0.0, 1.0) * 255.0) as u8)
                    })
                    .collect();
                let _ = req.reply.send(out);
            }
        })?;
        ready_rx.recv()?.map_err(|e| anyhow::anyhow!(e))?;
        Ok(Self { tx })
    }

    /// Render rover `rover`'s head depth at `(x, y, θ)` as LiDAR points.
    pub async fn scan(&self, rover: usize, x: f64, y: f64, theta: f64) -> Option<Vec<LidarPoint>> {
        let (reply, rx) = oneshot::channel();
        self.tx.send(Request { rover, pose: (x, y, theta), reply }).ok()?;
        rx.await.ok()
    }
}
