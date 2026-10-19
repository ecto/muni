//! The Booster K1, from Booster's own URDF and meshes, loaded by path.
//!
//! The assets are BSD-3 (BoosterRobotics) and are not copied into muni: set
//! `MUNI_K1_DIR` to the `robots/K1` directory of a `booster_assets`
//! checkout, or it defaults to `$HOME/Developer/booster_assets/robots/K1`.
//! The ZED variant (`K1_22dof-ZED.urdf`) is the one with the stereo head.

use crate::stl::{self, Tris};
use crate::trace::PoseSample;
use crate::urdf::Robot;
use crate::xf::Xf;
use crate::Error;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The URDF this crate renders by default.
pub const URDF: &str = "K1_22dof-ZED.urdf";
/// The link the head camera is bolted to (ipse's `HEAD_LINK`).
pub const HEAD_LINK: &str = "Head_2";

/// Where the K1 assets live: `MUNI_K1_DIR`, else the default checkout path.
pub fn asset_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("MUNI_K1_DIR") {
        return d.into();
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join("Developer/booster_assets/robots/K1")
}

/// One visual mesh, riding on one link.
#[derive(Debug, Clone)]
pub struct LinkMesh {
    /// Link index in [`Robot::links`].
    pub link: usize,
    /// Link → mesh, scale folded in.
    pub origin: Xf,
    /// Triangles.
    pub tris: Tris,
    /// URDF colour.
    pub rgb: [f64; 3],
}

/// A pose of the K1: where the trunk is, and every joint angle.
#[derive(Debug, Clone, Default)]
pub struct K1Pose {
    /// Trunk link → world.
    pub base: Xf,
    /// Joint angles by URDF name, rad. Missing joints sit at zero.
    pub q: HashMap<String, f64>,
}

/// The loaded K1: kinematics plus meshes.
#[derive(Debug, Clone)]
pub struct K1Model {
    /// Kinematic tree.
    pub robot: Robot,
    /// Every visual mesh.
    pub meshes: Vec<LinkMesh>,
    /// `Head_2`'s link index.
    pub head: usize,
}

impl K1Model {
    /// Load the K1 from [`asset_dir`] / [`URDF`].
    pub fn load_default() -> Result<Self, Error> {
        Self::load(&asset_dir().join(URDF))
    }

    /// Load a K1 URDF and every mesh it names.
    pub fn load(urdf: &Path) -> Result<Self, Error> {
        let robot = Robot::load(urdf)?;
        let mut meshes = Vec::new();
        for (li, l) in robot.links.iter().enumerate() {
            for v in &l.visuals {
                let tris = stl::load(&v.mesh)?;
                let mut origin = v.origin;
                for row in origin.r.iter_mut() {
                    for (c, s) in row.iter_mut().zip(v.scale) {
                        *c *= s;
                    }
                }
                meshes.push(LinkMesh { link: li, origin, tris, rgb: v.rgb });
            }
        }
        let head = robot
            .link(HEAD_LINK)
            .ok_or_else(|| Error::Urdf(format!("no {HEAD_LINK} link")))?;
        Ok(Self { robot, meshes, head })
    }

    /// Triangles over all meshes.
    pub fn triangles(&self) -> usize {
        self.meshes.iter().map(|m| m.tris.indices.len() / 3).sum()
    }

    /// World pose of every link.
    pub fn links(&self, pose: &K1Pose) -> Vec<Xf> {
        self.robot.fk(&pose.base, &pose.q)
    }

    /// World pose of every visual mesh (mesh → world).
    pub fn mesh_poses(&self, pose: &K1Pose) -> Vec<Xf> {
        let links = self.links(pose);
        self.meshes.iter().map(|m| links[m.link].then(&m.origin)).collect()
    }

    /// The head link's world pose.
    pub fn head(&self, pose: &K1Pose) -> Xf {
        self.links(pose)[self.head]
    }

    /// A pose standing on `ground_z`: trunk at `(x, y)` heading `yaw`, tilted
    /// by `roll`/`pitch`, joints `q`, lowered until its lowest vertex touches
    /// the ground (the feet when standing, the seat when sitting).
    pub fn standing(&self, x: f64, y: f64, yaw: f64, roll: f64, pitch: f64, q: HashMap<String, f64>, ground_z: f64) -> K1Pose {
        let mut pose = K1Pose { base: Xf::from_xyz_rpy([x, y, 0.0], [roll, pitch, yaw]), q };
        let poses = self.mesh_poses(&pose);
        let lowest = (0..self.meshes.len())
            .flat_map(|i| self.meshes[i].tris.positions.iter().map(move |p| (i, p)))
            .map(|(i, p)| poses[i].point([p[0] as f64, p[1] as f64, p[2] as f64])[2])
            .fold(f64::INFINITY, f64::min);
        if lowest.is_finite() {
            pose.base.t[2] = ground_z - lowest;
        }
        pose
    }

    /// A pose from one trace sample: joints and IMU roll/pitch from the
    /// trace, position and heading from its odometer, offset by `origin`
    /// (x, y, yaw) so a trace can be dropped anywhere on a street.
    pub fn from_sample(&self, s: &PoseSample, origin: [f64; 3], ground_z: f64) -> K1Pose {
        let (sy, cy) = origin[2].sin_cos();
        let [ox, oy, th] = s.odom;
        let x = origin[0] + cy * ox - sy * oy;
        let y = origin[1] + sy * ox + cy * oy;
        self.standing(x, y, origin[2] + th, s.rpy[0], s.rpy[1], s.q.clone(), ground_z)
    }
}
