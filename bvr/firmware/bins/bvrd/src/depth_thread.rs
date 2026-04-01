//! Dedicated depth perception thread.
//!
//! Runs monocular depth estimation + 3D back-projection on a separate
//! `std::thread` to avoid blocking the 100 Hz control loop (~15ms per
//! inference on Jetson with TensorRT).
//!
//! Architecture (mirrors SLAM thread):
//! ```text
//! Camera threads ──► mpsc::Sender<RawFrame> ──► depth thread
//!                                                   │
//!                         watch::Receiver ◄─────────┘
//!                         (ClassifiedPoints per camera, merged)
//! ```

use depth::{backproject_and_classify, CameraGeometry, DepthPerceptionConfig};
use std::sync::mpsc;
use tokio::sync::watch;
use tracing::{debug, info, warn};

/// A raw frame tagged with camera ID for routing to the correct geometry.
#[allow(dead_code)]
pub(crate) struct TaggedFrame {
    pub cam_id: String,
    pub rgb: std::sync::Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
}

/// Merged output from all cameras for one processing cycle.
#[derive(Clone, Default)]
#[allow(dead_code)]
pub(crate) struct DepthOutput {
    pub ground: Vec<nalgebra::Vector3<f64>>,
    pub obstacles: Vec<nalgebra::Vector3<f64>>,
    pub timestamp_ms: u64,
}

/// Configuration for the depth thread.
pub(crate) struct DepthThreadConfig {
    pub geometries: Vec<(String, CameraGeometry)>,
    pub perception: DepthPerceptionConfig,
    #[cfg(feature = "depth-onnx")]
    pub model_path: String,
    #[cfg(feature = "depth-onnx")]
    pub model_input_width: u32,
    #[cfg(feature = "depth-onnx")]
    pub model_input_height: u32,
}

/// Spawn the depth processing thread.
///
/// Returns:
/// - `mpsc::Sender<TaggedFrame>` — send raw frames here (non-blocking, bounded)
/// - `watch::Receiver<DepthOutput>` — latest classified points from all cameras
pub(crate) fn spawn_depth_thread(
    config: DepthThreadConfig,
) -> (mpsc::Sender<TaggedFrame>, watch::Receiver<DepthOutput>) {
    let (frame_tx, frame_rx) = mpsc::channel::<TaggedFrame>();
    let (output_tx, output_rx) = watch::channel(DepthOutput::default());

    std::thread::Builder::new()
        .name("depth".to_string())
        .spawn(move || {
            depth_thread_main(config, frame_rx, output_tx);
        })
        .expect("Failed to spawn depth thread");

    (frame_tx, output_rx)
}

fn depth_thread_main(
    config: DepthThreadConfig,
    frame_rx: mpsc::Receiver<TaggedFrame>,
    output_tx: watch::Sender<DepthOutput>,
) {
    info!(
        cameras = config.geometries.len(),
        "Depth thread started"
    );

    // Load ONNX model if feature enabled
    #[cfg(feature = "depth-onnx")]
    let mut estimator = {
        match depth::DepthEstimator::load(
            &config.model_path,
            config.model_input_width,
            config.model_input_height,
        ) {
            Ok(est) => {
                info!("Depth model loaded (ONNX)");
                Some(est)
            }
            Err(e) => {
                warn!(?e, "Failed to load depth model — depth thread will idle");
                None
            }
        }
    };

    #[cfg(not(feature = "depth-onnx"))]
    {
        warn!("Depth thread running without ONNX inference (build with --features depth-onnx)");
    }

    // Collect latest frame per camera, then process batch
    let mut latest_frames: std::collections::HashMap<String, TaggedFrame> =
        std::collections::HashMap::new();

    loop {
        // Block on first frame, then drain any queued frames
        let first = match frame_rx.recv() {
            Ok(f) => f,
            Err(_) => {
                info!("Depth thread: channel closed, shutting down");
                break;
            }
        };
        latest_frames.insert(first.cam_id.clone(), first);

        // Drain remaining (non-blocking) — keep only latest per camera
        while let Ok(frame) = frame_rx.try_recv() {
            latest_frames.insert(frame.cam_id.clone(), frame);
        }

        let mut all_ground = Vec::new();
        let mut all_obstacles = Vec::new();
        let mut latest_ts = 0u64;

        for (cam_id, frame) in latest_frames.drain() {
            let geom = match config.geometries.iter().find(|(id, _)| *id == cam_id) {
                Some((_, g)) => g,
                None => continue,
            };

            latest_ts = latest_ts.max(frame.timestamp_ms);

            // Run depth estimation
            #[cfg(feature = "depth-onnx")]
            let depth_map = {
                let est = match estimator.as_mut() {
                    Some(e) => e,
                    None => continue,
                };
                match est.estimate(&frame.rgb, frame.width, frame.height, frame.timestamp_ms) {
                    Ok(mut dm) => {
                        // Convert relative depth to metric if needed
                        if !dm.is_metric {
                            let mount_height = geom.position[2];
                            let mount_pitch = geom.rotation[1];
                            let scale =
                                depth::estimate_metric_scale(&dm, mount_height, mount_pitch);
                            let scaled: Vec<f32> =
                                dm.data.iter().map(|d| d * scale).collect();
                            dm = depth::DepthMap {
                                data: std::sync::Arc::new(scaled),
                                is_metric: true,
                                ..dm
                            };
                        }
                        Some(dm)
                    }
                    Err(e) => {
                        warn!(cam = %cam_id, ?e, "Depth inference failed");
                        None
                    }
                }
            };

            #[cfg(not(feature = "depth-onnx"))]
            let depth_map: Option<depth::DepthMap> = None;

            if let Some(dm) = depth_map {
                let classified = backproject_and_classify(&dm, geom, &config.perception);
                all_ground.extend(classified.ground);
                all_obstacles.extend(classified.obstacles);
            }
        }

        if !all_ground.is_empty() || !all_obstacles.is_empty() {
            debug!(
                ground = all_ground.len(),
                obstacles = all_obstacles.len(),
                "depth.output"
            );
            let _ = output_tx.send(DepthOutput {
                ground: all_ground,
                obstacles: all_obstacles,
                timestamp_ms: latest_ts,
            });
        }
    }

    info!("Depth thread shutting down");
}
