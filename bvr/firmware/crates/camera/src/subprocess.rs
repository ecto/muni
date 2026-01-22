//! GStreamer subprocess backend for CSI cameras.
//!
//! Runs GStreamer in an isolated process to avoid tokio runtime conflicts.
//! Communicates via Unix socket with length-prefixed JPEG frames.

use crate::{CameraError, Config, DetectedCamera, CameraType, Frame};
use std::io::Read;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Timeout for initial socket connection
const SOCKET_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout for reading frames
const FRAME_READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum frame size (10MB - sanity check)
const MAX_FRAME_SIZE: u32 = 10 * 1024 * 1024;

/// Handle to a running gst-camera subprocess.
pub struct SubprocessHandle {
    child: Child,
    socket_path: String,
}

impl Drop for SubprocessHandle {
    fn drop(&mut self) {
        info!("Shutting down gst-camera subprocess");

        // Try graceful shutdown first
        if let Err(e) = self.child.kill() {
            debug!(?e, "Failed to kill gst-camera (may have already exited)");
        }

        // Clean up socket file
        if let Err(e) = std::fs::remove_file(&self.socket_path) {
            debug!(?e, "Failed to remove socket file");
        }
    }
}

/// Detect CSI cameras by scanning sysfs.
pub fn detect_csi_cameras() -> Vec<DetectedCamera> {
    let mut cameras = Vec::new();

    // Scan /dev/video* and check sysfs for CSI indicators
    let Ok(entries) = std::fs::read_dir("/dev") else {
        return cameras;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !name.starts_with("video") {
            continue;
        }

        // Read device name from sysfs
        let sysfs_path = format!("/sys/class/video4linux/{}/name", name);
        let Ok(device_name) = std::fs::read_to_string(&sysfs_path) else {
            continue;
        };
        let device_name = device_name.trim();

        // Check for CSI camera indicators (NVIDIA VI, IMX sensors)
        if device_name.contains("vi-output") || device_name.contains("imx") {
            // Extract sensor ID from device number
            let Some(num_str) = name.strip_prefix("video") else {
                continue;
            };
            let Ok(sensor_id) = num_str.parse::<u32>() else {
                continue;
            };

            // Determine camera model from device name
            let model_name = if device_name.contains("imx477") {
                "IMX477 (HQ Camera)"
            } else if device_name.contains("imx219") {
                "IMX219 (Camera Module v2)"
            } else if device_name.contains("imx708") {
                "IMX708 (Camera Module v3)"
            } else {
                "CSI Camera"
            };

            info!(
                sensor_id,
                device_name,
                "Detected CSI camera"
            );

            cameras.push(DetectedCamera {
                camera_type: CameraType::Csi { sensor_id },
                name: format!("{} (sensor {})", model_name, sensor_id),
            });
        }
    }

    cameras
}

/// Spawn a gst-camera subprocess for a CSI camera.
///
/// Returns a channel receiver for frames and a handle to manage the subprocess.
pub fn spawn_subprocess(
    camera: &DetectedCamera,
    config: &Config,
    parent_pid: u32,
) -> Result<(mpsc::Receiver<Frame>, JoinHandle<()>), CameraError> {
    let CameraType::Csi { sensor_id } = camera.camera_type else {
        return Err(CameraError::Capture(
            "subprocess backend only supports CSI cameras".to_string(),
        ));
    };

    let socket_path = format!("/tmp/bvr-camera-{}.sock", parent_pid);

    info!(
        camera = %camera.name,
        socket = %socket_path,
        "Spawning gst-camera subprocess"
    );

    // Spawn gst-camera process
    let child = Command::new("gst-camera")
        .arg("--socket")
        .arg(&socket_path)
        .arg("--sensor-id")
        .arg(sensor_id.to_string())
        .arg("--width")
        .arg(config.width.to_string())
        .arg("--height")
        .arg(config.height.to_string())
        .arg("--fps")
        .arg(config.fps.to_string())
        .arg("--quality")
        .arg(config.jpeg_quality.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            CameraError::Capture(format!(
                "Failed to spawn gst-camera: {}. Is it installed?",
                e
            ))
        })?;

    info!(pid = child.id(), "gst-camera process started");

    let handle = SubprocessHandle {
        child,
        socket_path: socket_path.clone(),
    };

    // Wait for socket to appear
    let socket_path_ref = Path::new(&socket_path);
    let start = Instant::now();
    while !socket_path_ref.exists() {
        if start.elapsed() > SOCKET_CONNECT_TIMEOUT {
            return Err(CameraError::Capture(
                "Timeout waiting for gst-camera socket".to_string(),
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Connect to socket
    let mut stream = UnixStream::connect(&socket_path).map_err(|e| {
        CameraError::Capture(format!("Failed to connect to gst-camera socket: {}", e))
    })?;

    stream.set_read_timeout(Some(FRAME_READ_TIMEOUT)).map_err(|e| {
        CameraError::Capture(format!("Failed to set socket timeout: {}", e))
    })?;

    // Read handshake (width, height as u32 LE)
    let mut handshake = [0u8; 8];
    stream.read_exact(&mut handshake).map_err(|e| {
        CameraError::Capture(format!("Failed to read handshake: {}", e))
    })?;

    let width = u32::from_le_bytes([handshake[0], handshake[1], handshake[2], handshake[3]]);
    let height = u32::from_le_bytes([handshake[4], handshake[5], handshake[6], handshake[7]]);

    info!(
        camera = %camera.name,
        width,
        height,
        "Connected to gst-camera subprocess"
    );

    // Create channel for frames
    let (tx, rx) = mpsc::channel(4);

    let camera_name = camera.name.clone();

    // Spawn blocking task to read frames
    let join_handle = tokio::task::spawn_blocking(move || {
        let _handle = handle; // Move handle into task for cleanup on drop

        info!(camera = %camera_name, "Camera capture task started (subprocess)");

        let mut sequence: u32 = 0;
        let mut len_buf = [0u8; 4];

        loop {
            // Read frame length
            match stream.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                       || e.kind() == std::io::ErrorKind::TimedOut => {
                    warn!(error = %e, "Frame read timeout - gst-camera may have hung");
                    continue;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        info!("gst-camera subprocess terminated");
                    } else {
                        warn!(error = %e, "gst-camera subprocess died");
                    }
                    break;
                }
            }

            let len = u32::from_le_bytes(len_buf);

            // Sanity check
            if len > MAX_FRAME_SIZE {
                warn!(len, "Invalid frame length: {} bytes", len);
                break;
            }

            // Read frame data
            let mut data = vec![0u8; len as usize];
            match stream.read_exact(&mut data) {
                Ok(_) => {}
                Err(e) => {
                    warn!(error = %e, "Failed to read frame data");
                    break;
                }
            }

            sequence = sequence.wrapping_add(1);

            let timestamp_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let frame = Frame {
                data: Arc::new(data),
                width,
                height,
                timestamp_ms,
                sequence,
            };

            // Send frame (non-blocking - drop if channel full)
            match tx.try_send(frame) {
                Ok(_) => {
                    debug!(sequence, len, "Frame received from subprocess");
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    // Channel full, drop frame
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    info!("Frame receiver dropped, stopping capture");
                    break;
                }
            }
        }

        info!(camera = %camera_name, "Camera capture task stopped");
    });

    Ok((rx, join_handle))
}
