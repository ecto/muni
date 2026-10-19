//! Rigid transforms, just enough for URDF forward kinematics.

/// A rigid transform: `p_parent = r * p_child + t`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xf {
    /// Rotation, row-major.
    pub r: [[f64; 3]; 3],
    /// Translation.
    pub t: [f64; 3],
}

impl Default for Xf {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Xf {
    /// The identity.
    pub const IDENTITY: Xf = Xf {
        r: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        t: [0.0; 3],
    };

    /// A pure translation.
    pub fn translation(t: [f64; 3]) -> Self {
        Self { t, ..Self::IDENTITY }
    }

    /// URDF `<origin xyz rpy>`: `R = Rz(yaw) Ry(pitch) Rx(roll)`.
    pub fn from_xyz_rpy(xyz: [f64; 3], rpy: [f64; 3]) -> Self {
        let (sr, cr) = rpy[0].sin_cos();
        let (sp, cp) = rpy[1].sin_cos();
        let (sy, cy) = rpy[2].sin_cos();
        let r = [
            [cy * cp, cy * sp * sr - sy * cr, cy * sp * cr + sy * sr],
            [sy * cp, sy * sp * sr + cy * cr, sy * sp * cr - cy * sr],
            [-sp, cp * sr, cp * cr],
        ];
        Self { r, t: xyz }
    }

    /// Rotation by `angle` about a (unit) `axis`, Rodrigues.
    pub fn axis_angle(axis: [f64; 3], angle: f64) -> Self {
        let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if n == 0.0 {
            return Self::IDENTITY;
        }
        let [x, y, z] = [axis[0] / n, axis[1] / n, axis[2] / n];
        let (s, c) = angle.sin_cos();
        let v = 1.0 - c;
        let r = [
            [c + x * x * v, x * y * v - z * s, x * z * v + y * s],
            [y * x * v + z * s, c + y * y * v, y * z * v - x * s],
            [z * x * v - y * s, z * y * v + x * s, c + z * z * v],
        ];
        Self { r, t: [0.0; 3] }
    }

    /// `self ∘ other`: apply `other` first.
    pub fn then(&self, other: &Xf) -> Xf {
        let mut r = [[0.0; 3]; 3];
        for (i, row) in r.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v = (0..3).map(|k| self.r[i][k] * other.r[k][j]).sum();
            }
        }
        Xf { r, t: self.point(other.t) }
    }

    /// Transform a point.
    pub fn point(&self, p: [f64; 3]) -> [f64; 3] {
        let mut o = self.t;
        for (i, oi) in o.iter_mut().enumerate() {
            *oi += self.r[i][0] * p[0] + self.r[i][1] * p[1] + self.r[i][2] * p[2];
        }
        o
    }

    /// Transform a direction.
    pub fn dir(&self, d: [f64; 3]) -> [f64; 3] {
        let mut o = [0.0; 3];
        for (i, oi) in o.iter_mut().enumerate() {
            *oi = self.r[i][0] * d[0] + self.r[i][1] * d[1] + self.r[i][2] * d[2];
        }
        o
    }

    /// irys's row-major 3×4 object→world.
    pub fn to_irys(&self) -> [[f32; 4]; 3] {
        let mut m = [[0.0f32; 4]; 3];
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] = self.r[i][j] as f32;
            }
            m[i][3] = self.t[i] as f32;
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpy_yaw_turns_x_into_y() {
        let x = Xf::from_xyz_rpy([0.0; 3], [0.0, 0.0, std::f64::consts::FRAC_PI_2]);
        let p = x.dir([1.0, 0.0, 0.0]);
        assert!((p[1] - 1.0).abs() < 1e-12 && p[0].abs() < 1e-12);
    }

    #[test]
    fn axis_angle_matches_rpy() {
        let a = Xf::axis_angle([0.0, 1.0, 0.0], 0.3);
        let b = Xf::from_xyz_rpy([0.0; 3], [0.0, 0.3, 0.0]);
        for i in 0..3 {
            for j in 0..3 {
                assert!((a.r[i][j] - b.r[i][j]).abs() < 1e-12);
            }
        }
    }
}
