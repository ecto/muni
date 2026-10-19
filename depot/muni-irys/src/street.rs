//! The snowy street: muni's sidewalk-clearing setting, built from boxes.
//!
//! Z up, metres, the street runs along x. Road for `y < -1.5`, a 15 cm curb,
//! a 3 m sidewalk with a cleared centre strip and snow at both edges, a
//! snow-covered lawn, lamp posts, a hydrant, a bench and a building wall.
//! Snow is a white OpenPBR with subsurface and fuzz set (irys currently
//! shades subsurface as a Lambertian of `subsurface_color`; see the note).

use irys_diff::look::openpbr::OpenPbr;
use irys_diff::look::scene::{Equirect, Material, Mesh, Sun};

/// Material slots, indices into [`Street::materials`].
pub mod mat {
    /// Asphalt.
    pub const ASPHALT: usize = 0;
    /// Concrete (sidewalk, curb).
    pub const CONCRETE: usize = 1;
    /// Snow.
    pub const SNOW: usize = 2;
    /// Painted steel (posts, hydrant cap).
    pub const STEEL: usize = 3;
    /// Brick wall.
    pub const BRICK: usize = 4;
    /// Hydrant red.
    pub const RED: usize = 5;
    /// Bench wood.
    pub const WOOD: usize = 6;
    /// Count.
    pub const COUNT: usize = 7;
}

/// The surface the K1 stands on: the sidewalk's top.
pub const SIDEWALK_Z: f64 = 0.15;

/// The street as meshes in its own frame, ready to be instanced.
#[derive(Debug, Clone)]
pub struct Street {
    /// One mesh per box, in street coordinates (instanced at identity + tile offset).
    pub meshes: Vec<Mesh>,
    /// Materials, indexed by [`mat`].
    pub materials: Vec<Material>,
    /// Sky.
    pub env: Equirect,
    /// Sun.
    pub sun: Sun,
}

/// An axis-aligned box from `min` to `max`, optionally yawed about its centre.
pub fn cuboid(min: [f64; 3], max: [f64; 3], yaw: f64, material: usize) -> Mesh {
    let c = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0, (min[2] + max[2]) / 2.0];
    let h = [(max[0] - min[0]) / 2.0, (max[1] - min[1]) / 2.0, (max[2] - min[2]) / 2.0];
    let (s, co) = yaw.sin_cos();
    let mut positions = Vec::with_capacity(8);
    for i in 0..8 {
        let l = [
            if i & 1 == 0 { -h[0] } else { h[0] },
            if i & 2 == 0 { -h[1] } else { h[1] },
            if i & 4 == 0 { -h[2] } else { h[2] },
        ];
        positions.push([
            (c[0] + co * l[0] - s * l[1]) as f32,
            (c[1] + s * l[0] + co * l[1]) as f32,
            (c[2] + l[2]) as f32,
        ]);
    }
    let indices = vec![
        0, 2, 1, 1, 2, 3, // -z
        4, 5, 6, 5, 7, 6, // +z
        0, 1, 4, 1, 5, 4, // -y
        2, 6, 3, 3, 6, 7, // +y
        0, 4, 2, 2, 4, 6, // -x
        1, 3, 5, 3, 7, 5, // +x
    ];
    Mesh { positions, normals: Vec::new(), uvs: Vec::new(), indices, material }
}

fn pbr(name: &str, rgb: [f64; 3], rough: f64) -> Material {
    Material::new(name, OpenPbr { base_color: rgb, specular_roughness: rough, ..OpenPbr::default() })
}

/// The street's materials.
pub fn materials() -> Vec<Material> {
    let mut m = vec![
        pbr("asphalt", [0.06, 0.06, 0.065], 0.85),
        pbr("concrete", [0.42, 0.41, 0.39], 0.8),
        Material::new(
            "snow",
            OpenPbr {
                base_color: [0.92, 0.94, 0.97],
                specular_roughness: 0.6,
                subsurface_weight: 0.7,
                subsurface_color: [0.9, 0.94, 0.98],
                subsurface_radius: 0.05,
                fuzz_weight: 0.35,
                fuzz_color: [1.0, 1.0, 1.0],
                fuzz_roughness: 0.6,
                ..OpenPbr::default()
            },
        ),
        Material::new(
            "steel",
            OpenPbr { base_color: [0.12, 0.14, 0.12], base_metalness: 0.6, specular_roughness: 0.4, ..OpenPbr::default() },
        ),
        pbr("brick", [0.32, 0.13, 0.08], 0.9),
        pbr("hydrant", [0.55, 0.05, 0.03], 0.45),
        pbr("wood", [0.28, 0.16, 0.08], 0.7),
    ];
    debug_assert_eq!(m.len(), mat::COUNT);
    m.shrink_to_fit();
    m
}

/// A cold winter sky: horizon haze to deep blue, row 0 at +z.
pub fn sky() -> Equirect {
    let (w, h) = (64u32, 32u32);
    let mut rgb = Vec::with_capacity((w * h) as usize);
    for r in 0..h {
        let el = std::f32::consts::FRAC_PI_2 - std::f32::consts::PI * (r as f32 + 0.5) / h as f32;
        let up = el.max(0.0).sin();
        let below = (-el).max(0.0).sin();
        let c = [0.55 - 0.35 * up, 0.62 - 0.27 * up, 0.75 - 0.05 * up].map(|x| x * (1.0 - 0.7 * below));
        for _ in 0..w {
            rgb.push(c);
        }
    }
    Equirect { size: (w, h), rgb }
}

impl Street {
    /// Build the street. `length` along x, centred on the origin.
    pub fn snowy(length: f64) -> Self {
        let half = length / 2.0;
        let (x0, x1) = (-half, half);
        let mut meshes = vec![
            cuboid([x0, -9.0, -0.3], [x1, -1.5, 0.0], 0.0, mat::ASPHALT),
            cuboid([x0, -1.5, -0.3], [x1, -1.35, SIDEWALK_Z], 0.0, mat::CONCRETE),
            cuboid([x0, -1.35, -0.3], [x1, 1.65, SIDEWALK_Z], 0.0, mat::CONCRETE),
            // Ploughed ridge along the gutter, and snow on the road shoulder.
            cuboid([x0, -2.4, 0.0], [x1, -1.55, 0.18], 0.0, mat::SNOW),
            // Edges of the sidewalk not yet cleared.
            cuboid([x0, -1.35, SIDEWALK_Z], [x1, -0.85, SIDEWALK_Z + 0.08], 0.0, mat::SNOW),
            cuboid([x0, 1.05, SIDEWALK_Z], [x1, 1.65, SIDEWALK_Z + 0.1], 0.0, mat::SNOW),
            // The lawn, buried.
            cuboid([x0, 1.65, -0.3], [x1, 14.0, 0.35], 0.0, mat::SNOW),
            cuboid([x0, 14.0, -0.3], [x1, 14.6, 7.0], 0.0, mat::BRICK),
            // The far kerb and a strip of buildings across the road.
            cuboid([x0, -10.0, -0.3], [x1, -9.0, 0.3], 0.0, mat::SNOW),
            cuboid([x0, -16.6, -0.3], [x1, -16.0, 9.0], 0.0, mat::BRICK),
        ];
        // Lumpy windrow on the curb side: a run of yawed boxes.
        let mut x = x0 + 1.0;
        let mut k = 0u32;
        while x < x1 - 1.0 {
            let w = 0.8 + 0.5 * ((k * 7 % 5) as f64 / 4.0);
            let hgt = 0.25 + 0.2 * ((k * 3 % 4) as f64 / 3.0);
            let yaw = 0.15 * ((k * 5 % 7) as f64 / 6.0 - 0.5);
            meshes.push(cuboid([x, -2.2, 0.0], [x + w, -1.6, hgt], yaw, mat::SNOW));
            x += w * 0.8;
            k += 1;
        }
        // Lamp posts with a cap of snow, every 12 m on the lawn side.
        let mut x = x0 + 4.0;
        while x < x1 {
            meshes.push(cuboid([x - 0.06, 1.9, 0.0], [x + 0.06, 2.02, 4.5], 0.0, mat::STEEL));
            meshes.push(cuboid([x - 0.5, 1.85, 4.5], [x + 0.06, 2.07, 4.6], 0.0, mat::STEEL));
            meshes.push(cuboid([x - 0.5, 1.85, 4.6], [x + 0.06, 2.07, 4.66], 0.0, mat::SNOW));
            x += 12.0;
        }
        // A hydrant at the curb and a bench on the lawn side.
        meshes.push(cuboid([2.0, -1.2, SIDEWALK_Z], [2.22, -0.98, SIDEWALK_Z + 0.7], 0.0, mat::RED));
        meshes.push(cuboid([1.97, -1.23, SIDEWALK_Z + 0.7], [2.25, -0.95, SIDEWALK_Z + 0.76], 0.0, mat::SNOW));
        meshes.push(cuboid([6.0, 1.7, 0.35], [7.6, 2.15, 0.8], 0.0, mat::WOOD));
        meshes.push(cuboid([6.0, 1.7, 0.8], [7.6, 2.15, 0.86], 0.0, mat::SNOW));
        meshes.push(cuboid([6.0, 2.1, 0.8], [7.6, 2.2, 1.3], 0.0, mat::WOOD));
        let el: f64 = 22f64.to_radians();
        let az: f64 = -35f64.to_radians();
        let dir = [el.cos() * az.cos(), el.cos() * az.sin(), el.sin()];
        let radius: f64 = 0.0093;
        let irradiance = 2.5;
        let l = irradiance / (std::f64::consts::PI * radius.sin().powi(2));
        Self {
            meshes,
            materials: materials(),
            env: sky(),
            sun: Sun { dir, radius, radiance: [l, l * 0.93, l * 0.82] },
        }
    }
}
