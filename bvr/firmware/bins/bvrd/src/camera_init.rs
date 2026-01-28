//! Camera initialization and hot-plug support.

use crate::state_types::SharedState;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use teleop::video::H264Frame;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Start a single camera with H.264 encoding: spawn_h264_capture + sync→async bridge + forwarder.
///
/// Returns `true` if the camera started successfully. The forwarder task removes
/// the camera's stable ID from `active_cameras` when it exits, enabling the
/// hot-plug monitor to re-detect and restart it.
pub(crate) fn start_camera(
    cam: &camera::DetectedCamera,
    config: camera::H264Config,
    next_camera_id: &Arc<AtomicU8>,
    active_cameras: &Arc<Mutex<HashSet<String>>>,
    video_tx_rtc: &tokio::sync::mpsc::Sender<H264Frame>,
    shared: &Arc<Mutex<SharedState>>,
    keyframe_flag: &Arc<std::sync::atomic::AtomicBool>,
) -> bool {
    let stable_id = cam.stable_id();
    let camera_id = next_camera_id.fetch_add(1, Ordering::Relaxed);

    match camera::spawn_h264_capture(cam, config, keyframe_flag.clone()) {
        Ok((frame_rx, _camera_handle)) => {
            info!(
                camera_id,
                stable_id = %stable_id,
                camera = %cam.name,
                "Camera {camera_id} started (H.264 GStreamer)",
            );

            // Register in active set
            active_cameras.lock().unwrap().insert(stable_id.clone());
            shared.lock().unwrap().health.camera_active = true;

            let video_tx_rtc = video_tx_rtc.clone();
            let active_cameras = active_cameras.clone();

            // Bridge sync channel → async channel
            let (async_tx, mut async_rx) = mpsc::channel::<camera::H264Frame>(4);

            tokio::task::spawn_blocking(move || {
                loop {
                    match frame_rx.recv() {
                        Ok(frame) => {
                            if async_tx.blocking_send(frame).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            // Forwarder task: converts camera::H264Frame → teleop H264Frame and sends to RTC
            tokio::spawn(async move {
                let mut frame_count: u64 = 0;
                let mut total_bytes: u64 = 0;
                let mut last_log = std::time::Instant::now();

                while let Some(frame) = async_rx.recv().await {
                    let frame_size = frame.data.len();
                    let h264_frame = H264Frame {
                        camera_id,
                        data: frame.data,
                        is_keyframe: frame.is_keyframe,
                        width: frame.width,
                        height: frame.height,
                        pts_ns: frame.pts_ns,
                        duration_ns: frame.duration_ns,
                        sequence: frame.sequence,
                    };

                    match video_tx_rtc.try_send(h264_frame) {
                        Ok(_) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {}
                        Err(mpsc::error::TrySendError::Closed(_)) => break,
                    }

                    frame_count += 1;
                    total_bytes += frame_size as u64;

                    if last_log.elapsed() >= std::time::Duration::from_secs(5) {
                        let elapsed = last_log.elapsed().as_secs_f64();
                        let fps = frame_count as f64 / elapsed;
                        let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
                        tracing::debug!(
                            camera_id,
                            frame_count,
                            fps = format!("{:.1}", fps),
                            mbps = format!("{:.2}", mbps),
                            "camera.h264_stats"
                        );
                        frame_count = 0;
                        total_bytes = 0;
                        last_log = std::time::Instant::now();
                    }
                }

                // Camera died — remove from active set so monitor can restart it
                active_cameras.lock().unwrap().remove(&stable_id);
                info!(camera_id, stable_id = %stable_id, "H.264 camera forwarder stopped, removed from active set");
            });

            true
        }
        Err(e) => {
            warn!(camera_id, ?e, "Failed to start camera {camera_id} - skipping");
            false
        }
    }
}
