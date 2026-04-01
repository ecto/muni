//! Monocular depth estimation, 3D back-projection, and visual odometry.
//!
//! This crate provides:
//! - `DepthMap`: dense per-pixel depth output
//! - `CameraGeometry`: intrinsics + extrinsics for back-projection
//! - `backproject_and_classify`: depth pixels → classified 3D points in rover frame
//! - `VisualOdometry`: frame-to-frame ego-motion from depth-derived 2D scans (ICP)
//! - `DepthEstimator` (behind `onnx` feature): ONNX Runtime inference for Depth Anything V2/V3

pub mod visual_odometry;

use nalgebra::{Matrix3, Rotation3, Vector3};
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum DepthError {
    #[error("Model load failed: {0}")]
    ModelLoad(String),
    #[error("Inference failed: {0}")]
    Inference(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Camera geometry for 3D back-projection.
///
/// Combines intrinsics (focal length, principal point) with extrinsics
/// (mount position and rotation on the rover). Constructed from a camera
/// mount config + image resolution.
#[derive(Debug, Clone)]
pub struct CameraGeometry {
    /// Focal length x (pixels)
    pub fx: f64,
    /// Focal length y (pixels)
    pub fy: f64,
    /// Principal point x (pixels)
    pub cx: f64,
    /// Principal point y (pixels)
    pub cy: f64,
    /// Mount position in rover frame [forward, left, up] (meters)
    pub position: [f32; 3],
    /// Mount rotation [roll, pitch, yaw] (radians)
    pub rotation: [f32; 3],
}

impl CameraGeometry {
    /// Create from FOV angles and mount parameters.
    ///
    /// `fov_h` / `fov_v`: horizontal/vertical field of view in radians.
    /// `position`: [forward, left, up] in meters.
    /// `rotation`: [roll, pitch, yaw] in radians.
    pub fn from_fov(
        fov_h: f32,
        fov_v: f32,
        image_width: u32,
        image_height: u32,
        position: [f32; 3],
        rotation: [f32; 3],
    ) -> Self {
        let fx = (image_width as f64) / (2.0 * (fov_h as f64 / 2.0).tan());
        let fy = (image_height as f64) / (2.0 * (fov_v as f64 / 2.0).tan());
        Self {
            fx,
            fy,
            cx: image_width as f64 / 2.0,
            cy: image_height as f64 / 2.0,
            position,
            rotation,
        }
    }
}

/// Dense depth map from monocular depth estimation.
///
/// Depth values are in meters (metric depth) or relative/inverse depth
/// depending on the model. Use `is_metric` to distinguish.
#[derive(Debug, Clone)]
pub struct DepthMap {
    /// Depth values, row-major (width * height)
    pub data: Arc<Vec<f32>>,
    /// Map width
    pub width: u32,
    /// Map height
    pub height: u32,
    /// Whether values are metric (meters) or relative
    pub is_metric: bool,
    /// Capture timestamp from source frame (ms since epoch)
    pub timestamp_ms: u64,
}

impl DepthMap {
    /// Get depth at pixel (x, y). Returns None if out of bounds.
    pub fn at(&self, x: u32, y: u32) -> Option<f32> {
        if x < self.width && y < self.height {
            Some(self.data[(y * self.width + x) as usize])
        } else {
            None
        }
    }
}

/// Configuration for depth-based 3D perception.
#[derive(Debug, Clone)]
pub struct DepthPerceptionConfig {
    /// Maximum depth to consider (meters). Points beyond are discarded.
    pub max_depth: f32,
    /// Minimum depth to consider (meters). Filters noise near camera.
    pub min_depth: f32,
    /// Height threshold for ground classification (meters above ground plane).
    /// Points with z < threshold are ground, z >= threshold are obstacles.
    pub ground_threshold: f32,
    /// Minimum obstacle height to report (meters). Filters small noise.
    pub min_obstacle_height: f32,
    /// Pixel stride for back-projection (1 = every pixel, 2 = every other, etc.)
    /// Higher = faster, fewer points.
    pub stride: u32,
}

impl Default for DepthPerceptionConfig {
    fn default() -> Self {
        Self {
            max_depth: 8.0,
            min_depth: 0.3,
            ground_threshold: 0.10,
            min_obstacle_height: 0.05,
            stride: 4,
        }
    }
}

/// 3D points classified as ground or obstacle, in rover body frame.
pub struct ClassifiedPoints {
    /// Ground points (z ≈ 0)
    pub ground: Vec<Vector3<f64>>,
    /// Obstacle points (z > ground_threshold)
    pub obstacles: Vec<Vector3<f64>>,
}

/// Back-project a depth map to 3D points in rover body frame, classified as
/// ground or obstacle.
///
/// Coordinate frames:
/// - Camera: Z-forward, X-right, Y-down (OpenCV convention)
/// - Rover: X-forward, Y-left, Z-up (ROS convention)
///
/// `mount_rotation` in `CameraGeometry` uses the bvr convention: Euler angles
/// [roll, pitch, yaw] describing the rotation FROM rover frame TO camera frame
/// (intrinsic ZYX). Negative pitch = looking down.
pub fn backproject_and_classify(
    depth: &DepthMap,
    geom: &CameraGeometry,
    config: &DepthPerceptionConfig,
) -> ClassifiedPoints {
    // Base rotation: camera frame → rover frame (no mount rotation)
    // cam Z (forward) → rover X, cam X (right) → rover -Y, cam Y (down) → rover -Z
    let cam_to_rover_base = Rotation3::from_matrix_unchecked(Matrix3::new(
        0.0, 0.0, 1.0, // rover X = cam Z
        -1.0, 0.0, 0.0, // rover Y = -cam X
        0.0, -1.0, 0.0, // rover Z = -cam Y
    ));

    // Mount rotation (rover → camera); transpose gives camera → rover
    let r_mount = Rotation3::from_euler_angles(
        geom.rotation[0] as f64,
        geom.rotation[1] as f64,
        geom.rotation[2] as f64,
    );
    let rot = r_mount.transpose() * cam_to_rover_base;

    let translation = Vector3::new(
        geom.position[0] as f64,
        geom.position[1] as f64,
        geom.position[2] as f64,
    );

    let stride = config.stride.max(1) as usize;
    let capacity = ((depth.width as usize / stride) * (depth.height as usize / stride)) / 2;
    let mut ground = Vec::with_capacity(capacity);
    let mut obstacles = Vec::with_capacity(capacity);

    for v in (0..depth.height as usize).step_by(stride) {
        for u in (0..depth.width as usize).step_by(stride) {
            let d = depth.data[v * depth.width as usize + u];

            if d < config.min_depth || d > config.max_depth || !d.is_finite() {
                continue;
            }

            let d = d as f64;

            // Back-project to camera frame (Z-forward, X-right, Y-down)
            let x_c = (u as f64 - geom.cx) * d / geom.fx;
            let y_c = (v as f64 - geom.cy) * d / geom.fy;
            let z_c = d;
            let p_cam = Vector3::new(x_c, y_c, z_c);

            // Transform to rover body frame
            let p_rover = rot * p_cam + translation;

            // Classify by height above ground
            let height = p_rover.z;
            let ground_thresh = config.ground_threshold as f64;
            let obs_thresh = config.min_obstacle_height as f64;

            if height < ground_thresh {
                ground.push(p_rover);
            } else if height >= obs_thresh {
                obstacles.push(p_rover);
            }
        }
    }

    debug!(
        ground = ground.len(),
        obstacles = obstacles.len(),
        "depth.backproject"
    );

    ClassifiedPoints { ground, obstacles }
}

/// Convert a relative/inverse depth map to approximate metric depth using
/// known camera mount height as reference.
///
/// Finds the median depth value in the lower portion of the image (assumed
/// ground) and computes a scale factor so those pixels map to the expected
/// geometric distance to the ground plane.
pub fn estimate_metric_scale(
    depth: &DepthMap,
    mount_height: f32,
    mount_pitch: f32,
) -> f32 {
    // Expected depth to ground: height / cos(pitch)
    let expected_ground_depth = mount_height / mount_pitch.abs().cos();

    // Sample the bottom 20% of the image (likely ground)
    let start_row = (depth.height as f32 * 0.8) as usize;
    let mut ground_depths: Vec<f32> = Vec::new();

    for y in start_row..depth.height as usize {
        for x in (0..depth.width as usize).step_by(4) {
            let d = depth.data[y * depth.width as usize + x];
            if d.is_finite() && d > 0.0 {
                ground_depths.push(d);
            }
        }
    }

    if ground_depths.is_empty() {
        return 1.0;
    }

    ground_depths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = ground_depths[ground_depths.len() / 2];

    if median > 0.0 {
        expected_ground_depth / median
    } else {
        1.0
    }
}

// =============================================================================
// ONNX Runtime inference (feature-gated)
// =============================================================================

#[cfg(feature = "onnx")]
mod onnx_inference {
    use super::*;
    use ort::session::Session;
    use ort::value::TensorRef;

    /// Depth model inference using ONNX Runtime.
    ///
    /// Supports Depth Anything V2/V3 Small ONNX models.
    /// On Jetson, automatically uses TensorRT execution provider.
    pub struct DepthEstimator {
        session: Session,
        input_width: u32,
        input_height: u32,
    }

    impl DepthEstimator {
        /// Load an ONNX depth model.
        pub fn load(model_path: &str, input_width: u32, input_height: u32) -> Result<Self, DepthError> {
            let session = Session::builder()
                .map_err(|e| DepthError::ModelLoad(format!("session builder: {e}")))?
                .commit_from_file(model_path)
                .map_err(|e| DepthError::ModelLoad(format!("load {model_path}: {e}")))?;

            tracing::info!(model_path, input_width, input_height, "Depth model loaded");

            Ok(Self {
                session,
                input_width,
                input_height,
            })
        }

        /// Run depth estimation on raw RGB pixel data.
        ///
        /// `rgb`: row-major RGB u8 buffer, `width * height * 3` bytes.
        /// Returns a depth map at the model's native output resolution.
        pub fn estimate(
            &mut self,
            rgb: &[u8],
            width: u32,
            height: u32,
            timestamp_ms: u64,
        ) -> Result<DepthMap, DepthError> {
            let (iw, ih) = (self.input_width as usize, self.input_height as usize);

            let resized = bilinear_resize(rgb, width as usize, height as usize, iw, ih);
            let input = normalize_imagenet(&resized, iw, ih);

            // Shape: [1, 3, H, W]
            let input_array = ndarray::Array4::from_shape_vec(
                (1, 3, ih, iw),
                input,
            )
            .map_err(|e| DepthError::Inference(format!("shape error: {e}")))?;

            let input_ref = TensorRef::from_array_view(&input_array)
                .map_err(|e| DepthError::Inference(format!("tensor ref: {e}")))?;

            let outputs = self.session
                .run(ort::inputs![input_ref])
                .map_err(|e| DepthError::Inference(e.to_string()))?;

            let output = outputs.values().next()
                .ok_or_else(|| DepthError::Inference("no output tensor".into()))?;

            let (shape, data) = output
                .try_extract_tensor::<f32>()
                .map_err(|e| DepthError::Inference(format!("extract tensor: {e}")))?;

            let (out_h, out_w) = match shape.len() {
                3 => (shape[1] as u32, shape[2] as u32),
                4 => (shape[2] as u32, shape[3] as u32),
                _ => return Err(DepthError::Inference(format!("unexpected shape: {shape:?}"))),
            };

            Ok(DepthMap {
                data: Arc::new(data.to_vec()),
                width: out_w,
                height: out_h,
                is_metric: false,
                timestamp_ms,
            })
        }
    }

    /// Bilinear resize RGB u8 buffer.
    fn bilinear_resize(
        src: &[u8],
        src_w: usize,
        src_h: usize,
        dst_w: usize,
        dst_h: usize,
    ) -> Vec<u8> {
        let mut dst = vec![0u8; dst_w * dst_h * 3];

        for y in 0..dst_h {
            for x in 0..dst_w {
                let sx = (x as f32 + 0.5) * src_w as f32 / dst_w as f32 - 0.5;
                let sy = (y as f32 + 0.5) * src_h as f32 / dst_h as f32 - 0.5;

                let x0 = sx.floor().max(0.0) as usize;
                let y0 = sy.floor().max(0.0) as usize;
                let x1 = (x0 + 1).min(src_w - 1);
                let y1 = (y0 + 1).min(src_h - 1);

                let fx = sx - x0 as f32;
                let fy = sy - y0 as f32;

                for c in 0..3 {
                    let p00 = src[(y0 * src_w + x0) * 3 + c] as f32;
                    let p10 = src[(y0 * src_w + x1) * 3 + c] as f32;
                    let p01 = src[(y1 * src_w + x0) * 3 + c] as f32;
                    let p11 = src[(y1 * src_w + x1) * 3 + c] as f32;

                    let val = p00 * (1.0 - fx) * (1.0 - fy)
                        + p10 * fx * (1.0 - fy)
                        + p01 * (1.0 - fx) * fy
                        + p11 * fx * fy;

                    dst[(y * dst_w + x) * 3 + c] = val.round() as u8;
                }
            }
        }
        dst
    }

    /// Normalize RGB u8 to CHW f32 with ImageNet mean/std.
    fn normalize_imagenet(rgb: &[u8], w: usize, h: usize) -> Vec<f32> {
        let mean = [0.485f32, 0.456, 0.406];
        let std = [0.229f32, 0.224, 0.225];
        let pixels = w * h;

        let mut chw = vec![0.0f32; 3 * pixels];
        for i in 0..pixels {
            for c in 0..3 {
                let val = rgb[i * 3 + c] as f32 / 255.0;
                chw[c * pixels + i] = (val - mean[c]) / std[c];
            }
        }
        chw
    }
}

#[cfg(feature = "onnx")]
pub use onnx_inference::DepthEstimator;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_depth_map(width: u32, height: u32, fill: f32) -> DepthMap {
        DepthMap {
            data: Arc::new(vec![fill; (width * height) as usize]),
            width,
            height,
            is_metric: true,
            timestamp_ms: 0,
        }
    }

    fn test_geom_center_cam() -> CameraGeometry {
        // Center camera: forward-looking, slight downward pitch, 0.25m above ground
        CameraGeometry::from_fov(
            1.745, // ~100° HFOV
            1.18,  // ~68° VFOV
            32,
            24,
            [0.25, 0.0, 0.25],
            [0.0, -0.175, 0.0], // -10° pitch (looking down)
        )
    }

    #[test]
    fn test_depth_map_at() {
        let dm = make_depth_map(4, 4, 1.5);
        assert_eq!(dm.at(0, 0), Some(1.5));
        assert_eq!(dm.at(3, 3), Some(1.5));
        assert_eq!(dm.at(4, 0), None);
    }

    #[test]
    fn test_camera_geometry_from_fov() {
        let geom = CameraGeometry::from_fov(1.745, 1.18, 1920, 1080, [0.0; 3], [0.0; 3]);
        assert!((geom.cx - 960.0).abs() < 0.1);
        assert!((geom.cy - 540.0).abs() < 0.1);
        assert!(geom.fx > 0.0);
        assert!(geom.fy > 0.0);
    }

    #[test]
    fn test_backproject_center_camera() {
        let geom = test_geom_center_cam();
        // Depth of 2m: center pixel projects to z ≈ -0.348 + 0.25 = -0.098 (below ground)
        // So most of the image at 2m depth should be classified as ground
        let depth = make_depth_map(32, 24, 2.0);

        let config = DepthPerceptionConfig {
            max_depth: 5.0,
            min_depth: 0.3,
            ground_threshold: 0.15,
            min_obstacle_height: 0.05,
            stride: 1,
        };

        let result = backproject_and_classify(&depth, &geom, &config);
        let total = result.ground.len() + result.obstacles.len();

        assert!(total > 0, "expected some points");
        assert!(!result.ground.is_empty(), "expected ground points");

        // Center pixel at depth 2m with -10° pitch from 0.25m height:
        // z ≈ -2*sin(0.175) + 0.25 ≈ -0.098 → ground
        // So the majority should be ground
        assert!(
            result.ground.len() > result.obstacles.len(),
            "ground {} should > obstacles {}",
            result.ground.len(),
            result.obstacles.len()
        );
    }

    #[test]
    fn test_backproject_center_pixel_position() {
        // Verify center pixel back-projection: camera at 0.25m height, -10° pitch
        // Center pixel at depth 1m should project to ≈ (1.235, 0, 0.076) in rover frame
        let geom = CameraGeometry::from_fov(
            1.745, 1.18,
            64, 48, // reasonable resolution
            [0.25, 0.0, 0.25],
            [0.0, -0.175, 0.0],
        );

        // Only set center pixel (32, 24) to valid depth, rest = 0 (below min_depth)
        let mut data = vec![0.0f32; 64 * 48];
        data[24 * 64 + 32] = 1.0; // center pixel at depth 1m
        let depth = DepthMap {
            data: Arc::new(data),
            width: 64,
            height: 48,
            is_metric: true,
            timestamp_ms: 0,
        };

        let config = DepthPerceptionConfig {
            max_depth: 5.0,
            min_depth: 0.1,
            ground_threshold: 999.0, // classify everything as ground
            min_obstacle_height: 999.0,
            stride: 1,
        };

        let result = backproject_and_classify(&depth, &geom, &config);
        assert_eq!(result.ground.len(), 1, "expected exactly 1 point");

        let p = &result.ground[0];
        // Center pixel (u=cx, v=cy) at depth 1m: p_cam = (0, 0, 1)
        // After cam→rover: ≈ (0.985, 0, -0.174) + (0.25, 0, 0.25) = (1.235, 0, 0.076)
        assert!((p.x - 1.235).abs() < 0.05, "x={:.3} expected ~1.235", p.x);
        assert!(p.y.abs() < 0.05, "y={:.3} expected ~0", p.y);
        assert!((p.z - 0.076).abs() < 0.05, "z={:.3} expected ~0.076", p.z);
    }

    #[test]
    fn test_backproject_filters_range() {
        let geom = CameraGeometry::from_fov(1.0, 0.75, 4, 4, [0.0; 3], [0.0, -0.1, 0.0]);

        let mut data = vec![0.0f32; 16]; // all zero = below min_depth
        data[0] = 999.0; // above max_depth
        data[1] = f32::NAN;
        data[2] = 1.0; // valid

        let depth = DepthMap {
            data: Arc::new(data),
            width: 4,
            height: 4,
            is_metric: true,
            timestamp_ms: 0,
        };

        let config = DepthPerceptionConfig {
            max_depth: 8.0,
            min_depth: 0.3,
            ground_threshold: 0.10,
            min_obstacle_height: 0.05,
            stride: 1,
        };

        let result = backproject_and_classify(&depth, &geom, &config);
        let total = result.ground.len() + result.obstacles.len();
        assert!(total <= 1, "expected at most 1 point, got {total}");
    }

    #[test]
    fn test_estimate_metric_scale() {
        let data = vec![0.5f32; 64 * 48];
        let depth = DepthMap {
            data: Arc::new(data),
            width: 64,
            height: 48,
            is_metric: false,
            timestamp_ms: 0,
        };

        let scale = estimate_metric_scale(&depth, 0.25, -0.175);
        // Expected: 0.25 / cos(0.175) ≈ 0.254, scale ≈ 0.508
        assert!(scale > 0.0 && scale < 2.0, "scale={scale}");
    }

    #[test]
    fn test_default_perception_config() {
        let config = DepthPerceptionConfig::default();
        assert!(config.max_depth > config.min_depth);
        assert!(config.stride >= 1);
    }
}
