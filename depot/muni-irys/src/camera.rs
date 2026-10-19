//! The K1's head camera: intrinsics, stereo baseline, mount.
//!
//! Numbers are the robot's, measured in ipse (`docs/camera.md` there): the
//! head stereo pair streams 544×448 per eye, `fx = fy = 204.639`,
//! `cx = 277.811`, `cy = 221.246`, no distortion, baseline 0.0525 m; vendor
//! depth (StereoNet) at ~17.7 Hz, raw eyes at ~25 Hz. The mount is ipse's
//! `head_extrinsics`: 6 cm forward and 3 cm up of the `Head_2` link origin,
//! looking along the link's +x.

use crate::xf::Xf;
use irys_diff::look::scene::Camera;

/// Which eye of the stereo pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eye {
    /// Left eye: the colour/depth reference frame (`head_color_optical_frame`).
    Left,
    /// Midpoint of the pair.
    Centre,
    /// Right eye.
    Right,
}

/// A pinhole head camera.
#[derive(Debug, Clone, Copy)]
pub struct HeadCamera {
    /// Width, px.
    pub width: u32,
    /// Height, px.
    pub height: u32,
    /// Focal length, px (fx = fy on the K1).
    pub f: f64,
    /// Principal point, px. irys's pinhole is centred, so this is recorded
    /// but not yet honoured; see the design note.
    pub cx: f64,
    /// Principal point, px.
    pub cy: f64,
    /// Stereo baseline, m.
    pub baseline: f64,
    /// Camera origin in the `Head_2` link frame, m.
    pub mount: [f64; 3],
    /// Depth stream rate on the robot, Hz.
    pub depth_hz: f64,
    /// Raw eye stream rate on the robot, Hz.
    pub rgb_hz: f64,
    /// Depth beyond this is reported as no-return, m.
    pub max_range: f32,
}

impl HeadCamera {
    /// The K1's head stereo camera.
    pub const K1: HeadCamera = HeadCamera {
        width: 544,
        height: 448,
        f: 204.639,
        cx: 277.811,
        cy: 221.246,
        baseline: 0.0525,
        mount: [0.06, 0.0, 0.03],
        depth_hz: 17.7,
        rgb_hz: 25.0,
        max_range: 20.0,
    };

    /// The same camera at another resolution, focal length scaled with it.
    pub fn scaled(&self, width: u32, height: u32) -> Self {
        let s = width as f64 / self.width as f64;
        Self {
            width,
            height,
            f: self.f * s,
            cx: self.cx * s,
            cy: self.cy * height as f64 / self.height as f64,
            ..*self
        }
    }

    /// Horizontal and vertical field of view, rad.
    pub fn fov(&self) -> (f64, f64) {
        (
            2.0 * (self.width as f64 / 2.0 / self.f).atan(),
            2.0 * (self.height as f64 / 2.0 / self.f).atan(),
        )
    }

    /// The irys camera for `eye`, given the `Head_2` link's world pose.
    pub fn pose(&self, head: &Xf, eye: Eye) -> Camera {
        let side = match eye {
            Eye::Left => 0.5 * self.baseline,
            Eye::Centre => 0.0,
            Eye::Right => -0.5 * self.baseline,
        };
        let m = self.mount;
        Camera {
            eye: head.point([m[0], m[1] + side, m[2]]),
            forward: head.dir([1.0, 0.0, 0.0]),
            right: head.dir([0.0, -1.0, 0.0]),
            up: head.dir([0.0, 0.0, 1.0]),
            tan_half: self.height as f64 / 2.0 / self.f,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k1_fov_is_wide() {
        let (h, v) = HeadCamera::K1.fov();
        assert!((h.to_degrees() - 106.1).abs() < 0.5, "{}", h.to_degrees());
        assert!((v.to_degrees() - 95.2).abs() < 0.5, "{}", v.to_degrees());
    }

    #[test]
    fn left_eye_is_left() {
        let c = HeadCamera::K1.pose(&Xf::IDENTITY, Eye::Left);
        assert!(c.eye[1] > 0.0);
        assert_eq!(c.right, [0.0, -1.0, 0.0]);
    }
}
