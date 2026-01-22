//! GStreamer camera capture subprocess for bvrd.
//!
//! Captures video from CSI or USB cameras using GStreamer, encodes to JPEG,
//! and sends frames over a Unix domain socket to the parent process.
//!
//! # Why a Subprocess?
//!
//! GStreamer's threading model conflicts with tokio:
//! - NVIDIA plugins make blocking kernel calls that stall the async runtime
//! - Signal handlers can interfere with tokio's signal handling
//! - In-process GStreamer caused complete bvrd freezes on Jetson
//!
//! Running GStreamer in an isolated subprocess keeps the main daemon healthy.

use clap::Parser;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "gst-camera",
    about = "GStreamer camera capture subprocess for bvrd",
    long_about = "Captures video from CSI or USB cameras using GStreamer, encodes to JPEG, \
                  and sends frames over a Unix domain socket to the parent process."
)]
struct Args {
    /// Unix socket path for frame output
    #[arg(long)]
    socket: String,

    /// CSI camera sensor ID (0 or 1, mutually exclusive with --device)
    #[arg(long)]
    sensor_id: Option<u32>,

    /// V4L2 device path for USB cameras (e.g., /dev/video0)
    #[arg(long)]
    device: Option<String>,

    /// Capture width
    #[arg(long, default_value = "640")]
    width: u32,

    /// Capture height
    #[arg(long, default_value = "480")]
    height: u32,

    /// Target framerate
    #[arg(long, default_value = "30")]
    fps: u32,

    /// JPEG quality (1-100)
    #[arg(long, default_value = "60")]
    quality: u8,
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("gst_camera=info")
        .with_target(false)
        .init();

    let args = Args::parse();

    info!("Starting gst-camera");

    // Validate args
    if args.sensor_id.is_none() && args.device.is_none() {
        error!("Must specify either --sensor-id or --device");
        std::process::exit(1);
    }

    // Set up Ctrl-C handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    if let Err(e) = ctrlc::set_handler(move || {
        info!("Received shutdown signal");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }) {
        warn!(?e, "Failed to set Ctrl-C handler");
    }

    // Initialize GStreamer
    info!("Initializing GStreamer...");
    if let Err(e) = gst::init() {
        error!(?e, "Failed to initialize GStreamer");
        std::process::exit(1);
    }
    info!("GStreamer initialized successfully");

    // Remove existing socket file if present
    let socket_path = Path::new(&args.socket);
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(socket_path) {
            warn!(?e, "Failed to remove existing socket file");
        }
    }

    // Bind Unix socket
    let listener = match UnixListener::bind(&args.socket) {
        Ok(l) => l,
        Err(e) => {
            error!(?e, "Failed to bind Unix socket");
            std::process::exit(1);
        }
    };

    // Set non-blocking so we can check the running flag
    listener
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    info!("Listening for connections on {}", args.socket);

    // Wait for connection from bvrd
    let stream = loop {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            info!("gst-camera shutting down");
            std::process::exit(0);
        }

        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => {
                error!(?e, "Failed to accept connection");
                std::process::exit(1);
            }
        }
    };

    // Set blocking mode for the stream with timeout
    stream
        .set_nonblocking(false)
        .expect("Failed to set blocking mode");
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .expect("Failed to set write timeout");

    info!("Client connected");

    // Run the pipeline
    if let Err(e) = run_pipeline(&args, stream, running) {
        error!(?e, "Pipeline error");
        std::process::exit(1);
    }

    // Cleanup socket
    if let Err(e) = std::fs::remove_file(&args.socket) {
        debug!(?e, "Failed to remove socket file (may already be gone)");
    }

    info!("gst-camera shutting down");
}

fn run_pipeline(
    args: &Args,
    mut stream: UnixStream,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build pipeline string based on camera type
    let pipeline_str = if let Some(sensor_id) = args.sensor_id {
        // CSI camera with hardware JPEG encoding
        format!(
            "nvarguscamerasrc sensor-id={sensor_id} ! \
             video/x-raw(memory:NVMM),width={width},height={height},framerate={fps}/1 ! \
             nvjpegenc quality={quality} ! \
             appsink name=sink emit-signals=true max-buffers=2 drop=true sync=false",
            sensor_id = sensor_id,
            width = args.width,
            height = args.height,
            fps = args.fps,
            quality = args.quality,
        )
    } else if let Some(device) = &args.device {
        // USB camera with software JPEG encoding
        format!(
            "v4l2src device={device} ! \
             video/x-raw,width={width},height={height},framerate={fps}/1 ! \
             videoconvert ! \
             jpegenc quality={quality} ! \
             appsink name=sink emit-signals=true max-buffers=2 drop=true sync=false",
            device = device,
            width = args.width,
            height = args.height,
            fps = args.fps,
            quality = args.quality,
        )
    } else {
        return Err("Must specify either --sensor-id or --device".into());
    };

    debug!(pipeline = %pipeline_str, "Creating GStreamer pipeline");

    let pipeline = gst::parse::launch(&pipeline_str)?
        .downcast::<gst::Pipeline>()
        .map_err(|_| "Failed to downcast to Pipeline")?;

    let appsink = pipeline
        .by_name("sink")
        .ok_or("No appsink in pipeline")?
        .downcast::<gst_app::AppSink>()
        .map_err(|_| "Failed to downcast to AppSink")?;

    // Start pipeline
    pipeline.set_state(gst::State::Playing)?;

    // Wait for pipeline to reach Playing state
    let (result, state, _) = pipeline.state(Some(gst::ClockTime::from_seconds(10)));
    if result.is_err() || state != gst::State::Playing {
        return Err(format!("Pipeline failed to reach Playing state: {:?}", result).into());
    }

    info!("Pipeline started and ready");

    // Send initial handshake: width and height as u32 LE
    let mut handshake = Vec::with_capacity(8);
    handshake.extend_from_slice(&args.width.to_le_bytes());
    handshake.extend_from_slice(&args.height.to_le_bytes());
    stream.write_all(&handshake)?;

    let mut frames_sent: u64 = 0;
    let mut bytes_sent: u64 = 0;
    let mut last_stats = Instant::now();
    let mut consecutive_timeouts = 0;
    const MAX_CONSECUTIVE_TIMEOUTS: u32 = 10;

    // Main capture loop
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Pull sample with timeout (blocking but bounded)
        let sample = match appsink.try_pull_sample(gst::ClockTime::from_mseconds(500)) {
            Some(s) => {
                consecutive_timeouts = 0;
                s
            }
            None => {
                // Check if pipeline is still healthy
                let (_, state, _) = pipeline.state(Some(gst::ClockTime::from_mseconds(10)));
                if state != gst::State::Playing {
                    warn!("Pipeline stopped playing");
                    break;
                }

                consecutive_timeouts += 1;
                if consecutive_timeouts >= MAX_CONSECUTIVE_TIMEOUTS {
                    error!("Pipeline stopped playing after multiple timeouts");
                    break;
                }

                continue;
            }
        };

        let buffer = match sample.buffer() {
            Some(b) => b,
            None => continue,
        };

        let map = match buffer.map_readable() {
            Ok(m) => m,
            Err(e) => {
                warn!(?e, "Failed to map buffer");
                continue;
            }
        };

        let data = map.as_slice();
        let len = data.len() as u32;

        // Send frame: 4-byte length prefix + data
        if let Err(e) = stream.write_all(&len.to_le_bytes()) {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                info!("Client disconnected");
            } else {
                warn!(?e, "Failed to write frame length");
            }
            break;
        }

        if let Err(e) = stream.write_all(data) {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                info!("Client disconnected");
            } else {
                warn!(?e, "Failed to write frame data");
            }
            break;
        }

        frames_sent += 1;
        bytes_sent += len as u64;

        // Log stats every 5 seconds
        if last_stats.elapsed() >= Duration::from_secs(5) {
            let elapsed = last_stats.elapsed().as_secs_f64();
            let fps = frames_sent as f64 / elapsed;
            let mbps = (bytes_sent as f64 * 8.0) / (elapsed * 1_000_000.0);
            info!(
                frames = frames_sent,
                fps = format!("{:.1}", fps),
                mbps = format!("{:.2}", mbps),
                "Capture stats"
            );
            frames_sent = 0;
            bytes_sent = 0;
            last_stats = Instant::now();
        }
    }

    // Cleanup
    info!("Stopping pipeline");
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
