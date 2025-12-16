//! Camera capture and streaming for bvr.
//!
//! Captures frames from a webcam and encodes them as JPEG for streaming.

use image::codecs::jpeg::JpegEncoder;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::Camera;
use std::io::Cursor;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("Camera not found")]
    NotFound,
    #[error("Camera error: {0}")]
    Capture(String),
    #[error("Encoding error: {0}")]
    Encoding(String),
}

/// Camera configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Camera index (0 = default webcam)
    pub index: u32,
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
            index: 0,
            width: 640,
            height: 480,
            fps: 15,
            jpeg_quality: 70,
        }
    }
}

/// A captured and encoded frame.
#[derive(Debug, Clone)]
pub struct Frame {
    /// JPEG-encoded image data
    pub data: Vec<u8>,
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

/// Spawn a camera capture thread that sends frames to a channel.
///
/// The camera is opened and captured entirely within the spawned thread
/// to avoid Send requirements on the Camera type.
///
/// Returns a receiver for frames and a handle to the capture thread.
pub fn spawn_capture_thread(
    config: Config,
) -> Result<(mpsc::Receiver<Frame>, std::thread::JoinHandle<()>), CameraError> {
    let (tx, rx) = mpsc::channel();

    // We need to verify camera exists before spawning thread
    // but can't open it here due to Send requirements.
    // Instead, the thread will report errors via the channel closing.

    let handle = std::thread::spawn(move || {
        if let Err(e) = capture_loop(config, tx) {
            error!(?e, "Camera capture failed");
        }
    });

    Ok((rx, handle))
}

/// Internal capture loop that runs in a dedicated thread.
fn capture_loop(config: Config, tx: mpsc::Sender<Frame>) -> Result<(), CameraError> {
    let index = CameraIndex::Index(config.index);

    // List available cameras first for better error messages
    let cameras = nokhwa::query(nokhwa::utils::ApiBackend::Auto).unwrap_or_default();
    if cameras.is_empty() {
        error!("No cameras found on this system");
        return Err(CameraError::NotFound);
    }
    info!(count = cameras.len(), "Found cameras");
    for (i, cam) in cameras.iter().enumerate() {
        debug!(index = i, name = %cam.human_name(), "Camera");
    }

    // Try to open with preferred format, fall back to any format if that fails
    let mut camera = {
        // First try: closest match to requested resolution
        let format = CameraFormat::new(
            Resolution::new(config.width, config.height),
            FrameFormat::MJPEG,
            config.fps,
        );
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(format));

        match Camera::new(index.clone(), requested) {
            Ok(cam) => cam,
            Err(e) => {
                warn!(?e, "Failed with requested format, trying any format");
                // Second try: any format the camera supports
                let requested =
                    RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
                Camera::new(index, requested).map_err(|e| {
                    error!(?e, "Failed to open camera with any format");
                    CameraError::NotFound
                })?
            }
        }
    };

    camera.open_stream().map_err(|e| {
        error!(?e, "Failed to open camera stream");
        CameraError::Capture(e.to_string())
    })?;

    info!(
        index = config.index,
        width = config.width,
        height = config.height,
        fps = config.fps,
        "Camera stream opened"
    );

    let target_interval = Duration::from_secs_f64(1.0 / config.fps as f64);
    let mut sequence: u32 = 0;
    let mut last_frame = Instant::now();

    loop {
        // Rate limit
        let elapsed = last_frame.elapsed();
        if elapsed < target_interval {
            std::thread::sleep(target_interval - elapsed);
        }
        last_frame = Instant::now();

        // Capture frame
        let frame = match camera.frame() {
            Ok(f) => f,
            Err(e) => {
                warn!(?e, "Frame capture failed");
                continue;
            }
        };

        let resolution = frame.resolution();
        let width = resolution.width();
        let height = resolution.height();

        // Decode to RGB buffer
        let rgb_data = match frame.decode_image::<RgbFormat>() {
            Ok(d) => d,
            Err(e) => {
                warn!(?e, "Frame decode failed");
                continue;
            }
        };

        // Encode as JPEG
        let mut jpeg_buf = Vec::with_capacity((width * height) as usize);
        {
            let mut cursor = Cursor::new(&mut jpeg_buf);
            let mut encoder = JpegEncoder::new_with_quality(&mut cursor, config.jpeg_quality);
            if let Err(e) =
                encoder.encode(&rgb_data, width, height, image::ExtendedColorType::Rgb8)
            {
                warn!(?e, "JPEG encoding failed");
                continue;
            }
        }

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        sequence = sequence.wrapping_add(1);

        debug!(seq = sequence, size = jpeg_buf.len(), "Captured frame");

        let frame = Frame {
            data: jpeg_buf,
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

    info!("Camera capture loop exiting");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
