//! Camera capture and streaming for bvr.
//!
//! Uses GStreamer for camera capture, supporting both USB (v4l2src) and
//! CSI cameras (nvarguscamerasrc on Jetson).
//!
//! CSI cameras use a subprocess architecture to avoid GStreamer/tokio conflicts.
//! See the `subprocess` module for details.

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use image::codecs::jpeg::JpegEncoder;
use std::io::Cursor;
use std::sync::mpsc;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};

// Subprocess backend for CSI cameras (Linux only)
#[cfg(target_os = "linux")]
pub mod subprocess;

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("No cameras found")]
    NotFound,
    #[error("GStreamer error: {0}")]
    GStreamer(String),
    #[error("Capture error: {0}")]
    Capture(String),
    #[error("Encoding error: {0}")]
    Encoding(String),
}

/// Type of camera detected.
#[derive(Debug, Clone)]
pub enum CameraType {
    /// Jetson CSI camera (via nvarguscamerasrc)
    Csi { sensor_id: u32 },
    /// USB/V4L2 camera (Linux)
    Usb { device: String },
    /// AVFoundation camera (macOS)
    Avf { device_index: u32 },
}

/// A detected camera on the system.
#[derive(Debug, Clone)]
pub struct DetectedCamera {
    /// Camera type and source info
    pub camera_type: CameraType,
    /// Human-readable name
    pub name: String,
}

impl DetectedCamera {
    /// Stable identity key for tracking across hot-plug cycles.
    ///
    /// Returns a unique string per physical camera:
    /// - USB: device path (e.g. `/dev/video0`)
    /// - CSI: `csi:<sensor_id>`
    /// - AVF: `avf:<device_index>`
    pub fn stable_id(&self) -> String {
        match &self.camera_type {
            CameraType::Usb { device } => device.clone(),
            CameraType::Csi { sensor_id } => format!("csi:{sensor_id}"),
            CameraType::Avf { device_index } => format!("avf:{device_index}"),
        }
    }
}

/// Camera configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Capture width
    pub width: u32,
    /// Capture height
    pub height: u32,
    /// Target framerate
    pub fps: u32,
    /// JPEG quality (1-100)
    pub jpeg_quality: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 30,
            jpeg_quality: 60, // Lower quality = faster encoding + less bandwidth
        }
    }
}

/// A captured and encoded frame.
///
/// Uses `Arc<Vec<u8>>` for zero-copy sharing between multiple consumers
/// (WebRTC, UDP streaming). Clone is cheap (~8 bytes atomic increment).
#[derive(Debug, Clone)]
pub struct Frame {
    /// JPEG-encoded image data (Arc for zero-copy sharing)
    pub data: Arc<Vec<u8>>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Capture timestamp (milliseconds since epoch)
    pub timestamp_ms: u64,
    /// Sequence number
    pub sequence: u32,
}

/// Camera mount transform (fixed position on rover).
///
/// The camera is mounted at a known position relative to the rover's center.
/// This can be used to project camera images into 3D space.
#[derive(Debug, Clone, Copy)]
pub struct CameraMount {
    /// Position offset from rover center (meters): [forward, left, up]
    pub position: [f32; 3],
    /// Rotation from rover frame (roll, pitch, yaw in radians)
    /// Pitch is typically negative (looking down)
    pub rotation: [f32; 3],
    /// Horizontal field of view (radians)
    pub fov_h: f32,
    /// Vertical field of view (radians)
    pub fov_v: f32,
}

impl Default for CameraMount {
    fn default() -> Self {
        Self {
            // Front-mounted, slightly elevated, centered
            position: [0.25, 0.0, 0.25],
            // Looking slightly down (-10 degrees pitch)
            rotation: [0.0, -0.175, 0.0],
            // Typical webcam FOV (~60 degrees)
            fov_h: 1.05,
            fov_v: 0.79,
        }
    }
}

impl CameraMount {
    /// Create camera intrinsics matrix for projection.
    pub fn intrinsics(&self, width: u32, height: u32) -> [[f32; 3]; 3] {
        let fx = (width as f32) / (2.0 * (self.fov_h / 2.0).tan());
        let fy = (height as f32) / (2.0 * (self.fov_v / 2.0).tan());
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;

        [[fx, 0.0, cx], [0.0, fy, cy], [0.0, 0.0, 1.0]]
    }
}

/// Initialize GStreamer (safe to call multiple times).
///
/// Note: This can hang on Jetson with NVIDIA plugins if the registry is
/// not properly initialized. The hang occurs during plugin scanning.
fn ensure_gst_init() -> Result<(), CameraError> {
    info!("Initializing GStreamer (may hang on Jetson with NVIDIA plugins)...");
    let result = gst::init().map_err(|e| CameraError::GStreamer(e.to_string()));
    if result.is_ok() {
        info!("GStreamer initialized successfully");
    }
    result
}

/// Check if nvarguscamerasrc is available (Jetson-specific).
fn has_nvargus() -> bool {
    gst::ElementFactory::find("nvarguscamerasrc").is_some()
}

/// Try to probe a CSI camera by sensor ID.
fn probe_csi_camera(sensor_id: u32) -> Option<DetectedCamera> {
    if !has_nvargus() {
        return None;
    }

    // Try to create a brief pipeline to check if camera exists
    let pipeline_str = format!(
        "nvarguscamerasrc sensor-id={} num-buffers=1 ! fakesink",
        sensor_id
    );

    match gst::parse::launch(&pipeline_str) {
        Ok(pipeline) => {
            let pipeline = pipeline.downcast::<gst::Pipeline>().ok()?;

            // Try to set to PAUSED to see if it works
            if pipeline.set_state(gst::State::Paused).is_err() {
                let _ = pipeline.set_state(gst::State::Null);
                return None;
            }

            // Wait briefly for state change
            let (result, _, _) = pipeline.state(Some(gst::ClockTime::from_mseconds(500)));
            let _ = pipeline.set_state(gst::State::Null);

            if result == Ok(gst::StateChangeSuccess::Success)
                || result == Ok(gst::StateChangeSuccess::NoPreroll)
            {
                Some(DetectedCamera {
                    camera_type: CameraType::Csi { sensor_id },
                    name: format!("CSI Camera {}", sensor_id),
                })
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Detect USB cameras by scanning /dev/video* devices (Linux).
#[cfg(target_os = "linux")]
fn detect_usb_cameras() -> Vec<DetectedCamera> {
    let mut cameras = Vec::new();

    // Scan /dev/video* devices
    for entry in std::fs::read_dir("/dev").into_iter().flatten() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !name.starts_with("video") {
            continue;
        }

        let device = path.to_string_lossy().to_string();

        // Try to create a v4l2src element and check if it works
        let pipeline_str = format!(
            "v4l2src device={} num-buffers=1 ! fakesink",
            device
        );

        if let Ok(pipeline) = gst::parse::launch(&pipeline_str) {
            if let Ok(pipeline) = pipeline.downcast::<gst::Pipeline>() {
                if pipeline.set_state(gst::State::Paused).is_ok() {
                    let (result, _, _) = pipeline.state(Some(gst::ClockTime::from_mseconds(500)));
                    let _ = pipeline.set_state(gst::State::Null);

                    if result == Ok(gst::StateChangeSuccess::Success)
                        || result == Ok(gst::StateChangeSuccess::NoPreroll)
                    {
                        cameras.push(DetectedCamera {
                            camera_type: CameraType::Usb { device: device.clone() },
                            name: format!("USB Camera ({})", name),
                        });
                    }
                } else {
                    let _ = pipeline.set_state(gst::State::Null);
                }
            }
        }
    }

    cameras
}

/// Detect cameras via AVFoundation (macOS).
#[cfg(target_os = "macos")]
fn detect_usb_cameras() -> Vec<DetectedCamera> {
    let mut cameras = Vec::new();

    // Check if avfvideosrc is available
    if gst::ElementFactory::find("avfvideosrc").is_none() {
        debug!("avfvideosrc not available");
        return cameras;
    }

    // Try device indices 0-3
    for device_index in 0..4 {
        let pipeline_str = format!(
            "avfvideosrc device-index={} num-buffers=1 ! fakesink",
            device_index
        );

        if let Ok(pipeline) = gst::parse::launch(&pipeline_str) {
            if let Ok(pipeline) = pipeline.downcast::<gst::Pipeline>() {
                if pipeline.set_state(gst::State::Paused).is_ok() {
                    let (result, _, _) = pipeline.state(Some(gst::ClockTime::from_mseconds(1000)));
                    let _ = pipeline.set_state(gst::State::Null);

                    if result == Ok(gst::StateChangeSuccess::Success)
                        || result == Ok(gst::StateChangeSuccess::NoPreroll)
                    {
                        cameras.push(DetectedCamera {
                            camera_type: CameraType::Avf { device_index },
                            name: format!("Camera {}", device_index),
                        });
                        debug!(device_index, "Detected AVFoundation camera");
                    }
                } else {
                    let _ = pipeline.set_state(gst::State::Null);
                }
            }
        }
    }

    cameras
}

/// Fallback for other platforms.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn detect_usb_cameras() -> Vec<DetectedCamera> {
    Vec::new()
}

/// Auto-detect available cameras on the system.
///
/// Uses lightweight filesystem scanning for V4L2 devices to avoid GStreamer
/// initialization overhead (which can hang on Jetson with NVIDIA plugins).
/// GStreamer is only initialized when actually starting capture.
///
/// Checks for:
/// 1. CSI cameras (Jetson only, via nvarguscamerasrc) - detected via sysfs device name
/// 2. USB cameras (via v4l2src) - detected via /dev/video* devices
pub fn detect_cameras() -> Vec<DetectedCamera> {
    debug!("Detecting cameras via filesystem scan (avoiding GStreamer init)...");

    let mut cameras = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Track CSI sensor IDs we've found (to avoid duplicates from multiple /dev/video* entries)
        let mut found_csi_sensors: std::collections::HashSet<u32> = std::collections::HashSet::new();

        // Scan /dev/video* devices without using GStreamer
        for entry in std::fs::read_dir("/dev").into_iter().flatten() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if !name.starts_with("video") {
                continue;
            }

            // Extract video device number
            let num_str = name.strip_prefix("video").unwrap_or("");
            let device_num: u32 = match num_str.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };

            let device = path.to_string_lossy().to_string();

            // Check if device is accessible
            if std::fs::metadata(&path).is_err() {
                continue;
            }

            // Check sysfs for device name to identify CSI vs USB cameras
            let sysfs_name_path = format!("/sys/class/video4linux/{}/name", name);
            if let Ok(device_name) = std::fs::read_to_string(&sysfs_name_path) {
                let device_name = device_name.trim();

                // Check if this is a Jetson CSI camera (NVIDIA VI, IMX sensor, etc.)
                if device_name.contains("vi-output") || device_name.contains("imx") {
                    // CSI cameras on Jetson use sensor IDs 0, 1, etc.
                    // The /dev/video number doesn't directly map to sensor ID
                    // Typically video0 = sensor 0, video1 = sensor 1
                    let sensor_id = device_num;

                    if !found_csi_sensors.contains(&sensor_id) {
                        found_csi_sensors.insert(sensor_id);
                        cameras.push(DetectedCamera {
                            camera_type: CameraType::Csi { sensor_id },
                            name: format!("CSI Camera {} ({})", sensor_id, device_name),
                        });
                        info!(sensor_id, device_name, "Found CSI camera");
                    }
                    continue;
                }
            }

            // Not a CSI camera, treat as USB/V4L2
            cameras.push(DetectedCamera {
                camera_type: CameraType::Usb { device: device.clone() },
                name: format!("V4L2 Camera ({})", name),
            });
            debug!(device = %device, "Found V4L2 device");
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, we can't easily detect cameras without AVFoundation/GStreamer
        // Try device indices 0-2 and let capture validate them
        for device_index in 0..3 {
            cameras.push(DetectedCamera {
                camera_type: CameraType::Avf { device_index },
                name: format!("Camera {}", device_index),
            });
        }
    }

    // Sort by camera type (CSI first) then name for consistent ordering
    cameras.sort_by(|a, b| {
        match (&a.camera_type, &b.camera_type) {
            (CameraType::Csi { sensor_id: a_id }, CameraType::Csi { sensor_id: b_id }) => a_id.cmp(b_id),
            (CameraType::Csi { .. }, _) => std::cmp::Ordering::Less,
            (_, CameraType::Csi { .. }) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    info!(count = cameras.len(), "Camera detection complete");
    cameras
}

/// Auto-detect cameras using full GStreamer probing.
///
/// This is more thorough than `detect_cameras()` but requires GStreamer
/// initialization which can hang on some systems.
#[allow(dead_code)]
pub fn detect_cameras_gstreamer() -> Vec<DetectedCamera> {
    debug!("Starting GStreamer init for camera detection...");
    if let Err(e) = ensure_gst_init() {
        error!(?e, "Failed to initialize GStreamer");
        return Vec::new();
    }
    debug!("GStreamer initialized");

    let mut cameras = Vec::new();

    // Check for CSI cameras (sensor IDs 0 and 1)
    for sensor_id in 0..2 {
        if let Some(cam) = probe_csi_camera(sensor_id) {
            info!(sensor_id, "Detected CSI camera");
            cameras.push(cam);
        }
    }

    // Check for USB cameras
    let usb_cameras = detect_usb_cameras();
    for cam in usb_cameras {
        info!(device = ?cam.name, "Detected USB camera");
        cameras.push(cam);
    }

    cameras
}

/// Check if nvjpegenc (hardware JPEG encoder) is available.
fn has_nvjpegenc() -> bool {
    gst::ElementFactory::find("nvjpegenc").is_some()
}

/// Build GStreamer pipeline string for a camera.
/// Returns (pipeline_string, uses_hw_jpeg) tuple.
fn build_pipeline_string(camera: &DetectedCamera, config: &Config) -> (String, bool) {
    match &camera.camera_type {
        CameraType::Csi { sensor_id } => {
            // Jetson CSI camera pipeline with hardware JPEG encoding
            if has_nvjpegenc() {
                // Use hardware JPEG encoder - outputs JPEG directly
                let pipeline = format!(
                    "nvarguscamerasrc sensor-id={} ! \
                     video/x-raw(memory:NVMM),width={},height={},framerate={}/1 ! \
                     nvjpegenc quality={} ! \
                     appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                    sensor_id, config.width, config.height, config.fps, config.jpeg_quality
                );
                (pipeline, true)
            } else {
                // Fallback to software JPEG encoding - outputs RGB
                let pipeline = format!(
                    "nvarguscamerasrc sensor-id={} ! \
                     video/x-raw(memory:NVMM),width={},height={},framerate={}/1 ! \
                     nvvidconv ! \
                     video/x-raw,format=BGRx ! \
                     videoconvert ! \
                     video/x-raw,format=RGB ! \
                     appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                    sensor_id, config.width, config.height, config.fps
                );
                (pipeline, false)
            }
        }
        CameraType::Usb { device } => {
            // USB/V4L2 camera pipeline (Linux) - software JPEG encoding
            // Don't constrain v4l2src caps — let it negotiate the camera's native
            // format, then videoconvert + videoscale to the desired output.
            let pipeline = format!(
                "v4l2src device={} do-timestamp=true ! \
                 videoconvert ! \
                 videoscale ! \
                 video/x-raw,format=RGB,width={},height={} ! \
                 appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                device, config.width, config.height
            );
            (pipeline, false)
        }
        CameraType::Avf { device_index } => {
            // AVFoundation camera pipeline (macOS) - software JPEG encoding
            // Force landscape orientation and reasonable resolution
            let pipeline = format!(
                "avfvideosrc device-index={} do-timestamp=true ! \
                 video/x-raw,width=1280,height=720 ! \
                 videoconvert ! \
                 video/x-raw,format=RGB ! \
                 appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                device_index
            );
            (pipeline, false)
        }
    }
}

/// Build a fallback pipeline that's more flexible with formats.
/// Returns (pipeline_string, uses_hw_jpeg) tuple.
fn build_fallback_pipeline_string(camera: &DetectedCamera, config: &Config) -> (String, bool) {
    match &camera.camera_type {
        CameraType::Csi { sensor_id } => {
            // CSI camera fallback - try hardware JPEG first
            if has_nvjpegenc() {
                let pipeline = format!(
                    "nvarguscamerasrc sensor-id={} ! \
                     nvvidconv ! \
                     nvjpegenc quality={} ! \
                     appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                    sensor_id, config.jpeg_quality
                );
                (pipeline, true)
            } else {
                let pipeline = format!(
                    "nvarguscamerasrc sensor-id={} ! \
                     nvvidconv ! \
                     video/x-raw,format=BGRx ! \
                     videoconvert ! \
                     video/x-raw,format=RGB ! \
                     videoscale ! \
                     video/x-raw,width={},height={} ! \
                     appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                    sensor_id, config.width, config.height
                );
                (pipeline, false)
            }
        }
        CameraType::Usb { device } => {
            // USB camera with flexible format negotiation (Linux)
            let pipeline = format!(
                "v4l2src device={} do-timestamp=true ! \
                 videoconvert ! \
                 videoscale ! \
                 video/x-raw,format=RGB,width={},height={} ! \
                 appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                device, config.width, config.height
            );
            (pipeline, false)
        }
        CameraType::Avf { device_index } => {
            // AVFoundation camera fallback - more flexible caps (macOS)
            let pipeline = format!(
                "avfvideosrc device-index={} do-timestamp=true ! \
                 videoconvert ! \
                 videoscale add-borders=false ! \
                 video/x-raw,format=RGB,width={},height={} ! \
                 appsink name=sink emit-signals=true max-buffers=1 drop=true sync=false",
                device_index, config.width, config.height
            );
            (pipeline, false)
        }
    }
}

/// Spawn a camera capture thread that sends frames to a channel.
///
/// Returns a receiver for frames and a handle to the capture thread.
/// Uses a bounded channel (capacity 4) to prevent memory accumulation
/// when downstream consumers are slow or disconnected.
pub fn spawn_capture(
    camera: &DetectedCamera,
    config: Config,
) -> Result<(mpsc::Receiver<Frame>, std::thread::JoinHandle<()>), CameraError> {
    ensure_gst_init()?;

    // Use bounded sync_channel to prevent unbounded memory growth.
    // Capacity of 4 allows brief bursts while preventing accumulation.
    // When full, send() blocks which naturally throttles the capture loop.
    let (tx, rx) = mpsc::sync_channel(4);
    let camera = camera.clone();

    let handle = std::thread::spawn(move || {
        if let Err(e) = capture_loop(camera, config, tx) {
            error!(?e, "Camera capture failed");
        }
    });

    Ok((rx, handle))
}

/// Internal capture loop that runs in a dedicated thread.
fn capture_loop(
    camera: DetectedCamera,
    config: Config,
    tx: mpsc::SyncSender<Frame>,
) -> Result<(), CameraError> {
    // Try primary pipeline first, then fallback
    let (pipeline_str, hw_jpeg) = build_pipeline_string(&camera, &config);
    debug!(pipeline = %pipeline_str, hw_jpeg, "Trying primary pipeline");

    let (pipeline, uses_hw_jpeg) = match gst::parse::launch(&pipeline_str) {
        Ok(p) => (p, hw_jpeg),
        Err(e) => {
            warn!(?e, "Primary pipeline failed, trying fallback");
            let (fallback, fallback_hw_jpeg) = build_fallback_pipeline_string(&camera, &config);
            debug!(pipeline = %fallback, hw_jpeg = fallback_hw_jpeg, "Trying fallback pipeline");
            let p = gst::parse::launch(&fallback).map_err(|e| CameraError::GStreamer(e.to_string()))?;
            (p, fallback_hw_jpeg)
        }
    };

    let pipeline = pipeline
        .downcast::<gst::Pipeline>()
        .map_err(|_| CameraError::GStreamer("Failed to downcast pipeline".into()))?;

    // Get the appsink
    let appsink = pipeline
        .by_name("sink")
        .ok_or_else(|| CameraError::GStreamer("No appsink in pipeline".into()))?
        .downcast::<gst_app::AppSink>()
        .map_err(|_| CameraError::GStreamer("Failed to downcast appsink".into()))?;

    // Start the pipeline
    pipeline
        .set_state(gst::State::Playing)
        .map_err(|e| CameraError::GStreamer(format!("Failed to start pipeline: {:?}", e)))?;

    if uses_hw_jpeg {
        info!(camera = ?camera.name, "Camera capture started (hardware JPEG encoding)");
    } else {
        info!(camera = ?camera.name, "Camera capture started (software JPEG encoding)");
    }

    let mut sequence: u32 = 0;
    let jpeg_quality = config.jpeg_quality;

    // Pull samples from appsink
    loop {
        match appsink.pull_sample() {
            Ok(sample) => {
                let buffer = match sample.buffer() {
                    Some(b) => b,
                    None => continue,
                };

                let caps = match sample.caps() {
                    Some(c) => c,
                    None => continue,
                };

                // Get dimensions from caps
                let structure = match caps.structure(0) {
                    Some(s) => s,
                    None => continue,
                };

                let width = structure.get::<i32>("width").unwrap_or(config.width as i32) as u32;
                let height = structure.get::<i32>("height").unwrap_or(config.height as i32) as u32;

                // Map buffer to read data
                let map = match buffer.map_readable() {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(?e, "Failed to map buffer");
                        continue;
                    }
                };

                let encode_start = std::time::Instant::now();
                let jpeg_buf = if uses_hw_jpeg {
                    // Hardware JPEG: buffer already contains JPEG data
                    map.as_slice().to_vec()
                } else {
                    // Software JPEG: encode RGB data
                    let rgb_data = map.as_slice();
                    let mut buf = Vec::with_capacity((width * height) as usize);
                    {
                        let mut cursor = Cursor::new(&mut buf);
                        let mut encoder = JpegEncoder::new_with_quality(&mut cursor, jpeg_quality);
                        if let Err(e) =
                            encoder.encode(rgb_data, width, height, image::ExtendedColorType::Rgb8)
                        {
                            warn!(?e, "JPEG encoding failed");
                            continue;
                        }
                    }
                    buf
                };
                let encode_ms = encode_start.elapsed().as_secs_f64() * 1000.0;

                let timestamp_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                sequence = sequence.wrapping_add(1);

                trace!(
                    seq = sequence,
                    size = jpeg_buf.len(),
                    encode_ms = format!("{:.2}", encode_ms),
                    "camera.frame_captured"
                );

                let frame = Frame {
                    data: Arc::new(jpeg_buf),
                    width,
                    height,
                    timestamp_ms,
                    sequence,
                };

                if tx.send(frame).is_err() {
                    // Receiver dropped
                    break;
                }
            }
            Err(_) => {
                // Check if pipeline is still running
                let (_, state, _) = pipeline.state(Some(gst::ClockTime::from_mseconds(10)));
                if state != gst::State::Playing {
                    warn!("Pipeline stopped playing");
                    break;
                }
            }
        }
    }

    // Cleanup
    let _ = pipeline.set_state(gst::State::Null);
    info!("Camera capture loop exiting");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_id() {
        let usb = DetectedCamera {
            camera_type: CameraType::Usb { device: "/dev/video0".into() },
            name: "USB Camera".into(),
        };
        assert_eq!(usb.stable_id(), "/dev/video0");

        let csi = DetectedCamera {
            camera_type: CameraType::Csi { sensor_id: 1 },
            name: "CSI Camera".into(),
        };
        assert_eq!(csi.stable_id(), "csi:1");

        let avf = DetectedCamera {
            camera_type: CameraType::Avf { device_index: 2 },
            name: "AVF Camera".into(),
        };
        assert_eq!(avf.stable_id(), "avf:2");
    }

    #[test]
    fn test_camera_mount_intrinsics() {
        let mount = CameraMount::default();
        let k = mount.intrinsics(640, 480);

        // Principal point should be at image center
        assert!((k[0][2] - 320.0).abs() < 0.1);
        assert!((k[1][2] - 240.0).abs() < 0.1);

        // Focal lengths should be positive
        assert!(k[0][0] > 0.0);
        assert!(k[1][1] > 0.0);
    }
}



