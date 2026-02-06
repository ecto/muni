//! WebRTC-based teleop server for low-latency, unreliable command delivery.
//!
//! Uses WebRTC DataChannels configured for unreliable, unordered delivery
//! (essentially UDP over the web). This eliminates TCP's head-of-line blocking
//! and retransmission delays that cause jerky teleop.
//!
//! Architecture:
//! - WebSocket is used only for signaling (SDP offer/answer, ICE candidates)
//! - DataChannel "commands" carries teleop commands (unreliable, UNORDERED)
//! - DataChannel "telemetry" sends rover state back (unreliable)
//!
//! Operator Exclusivity:
//! - Only one connection can be the "operator" at a time
//! - Operator is claimed via SetMode(Teleop) command
//! - Movement commands (Twist, Tool) only accepted from operator
//! - Safety commands (EStop) accepted from anyone
//! - Operator status released on disconnect or after 5s inactivity

use crate::pointcloud;
use crate::costmap_stream;
use crate::obstacle_stream;
use crate::path_stream;
use crate::video::H264Frame;
use crate::{
    serialize_camera_info, serialize_telemetry, CommandHeader, Telemetry, TeleopError,
    CMD_HEADER_SIZE, MSG_ESTOP, MSG_ESTOP_RELEASE, MSG_HEARTBEAT, MSG_LIDAR_TOGGLE, MSG_SET_GOAL,
    MSG_SET_MODE, MSG_TOOL, MSG_TWIST,
};
use costmap::CostmapSnapshot;
use costmap::TrackedObstacle;
use lidar::PointCloud;
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::sync::mpsc as tokio_mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, trace, warn};
use types::{Command, Mode, ToolCommand, Twist};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

/// Per-channel send statistics (updated atomically by channel loops).
#[derive(Debug, Default)]
pub struct ChannelStats {
    /// Number of messages sent since last reset
    pub sent_count: AtomicU32,
    /// Sum of send latencies in microseconds (for averaging)
    pub send_us_sum: AtomicU64,
    /// Maximum send latency in microseconds
    pub send_us_max: AtomicU32,
}

impl ChannelStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a send operation with latency in microseconds.
    pub fn record_send(&self, latency_us: u32) {
        self.sent_count.fetch_add(1, Ordering::Relaxed);
        self.send_us_sum.fetch_add(latency_us as u64, Ordering::Relaxed);
        // Update max using compare-and-swap loop
        let mut current_max = self.send_us_max.load(Ordering::Relaxed);
        while latency_us > current_max {
            match self.send_us_max.compare_exchange_weak(
                current_max,
                latency_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_current) => current_max = new_current,
            }
        }
    }

    /// Get stats and reset counters. Returns (count, avg_us, max_us).
    pub fn take(&self) -> (u32, u32, u32) {
        let count = self.sent_count.swap(0, Ordering::Relaxed);
        let sum = self.send_us_sum.swap(0, Ordering::Relaxed);
        let max = self.send_us_max.swap(0, Ordering::Relaxed);
        let avg = if count > 0 { (sum / count as u64) as u32 } else { 0 };
        (count, avg, max)
    }
}

/// Aggregated WebRTC channel metrics (shared across all connections).
#[derive(Debug, Default)]
pub struct RtcMetrics {
    pub telemetry: ChannelStats,
    pub video: ChannelStats,
    pub pointcloud: ChannelStats,
    pub costmap: ChannelStats,
    pub obstacles: ChannelStats,
    pub path: ChannelStats,
}

impl RtcMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}

/// Timeout for operator inactivity (milliseconds).
const OPERATOR_TIMEOUT_MS: u64 = 5000;

/// Global connection ID counter.
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Shared state tracking the current operator across all connections.
#[derive(Debug)]
pub struct OperatorState {
    /// Connection ID of the current operator (0 = none).
    operator_id: AtomicU64,
    /// Last command timestamp as milliseconds since UNIX epoch.
    last_activity_ms: AtomicU64,
}

impl Default for OperatorState {
    fn default() -> Self {
        Self::new()
    }
}

impl OperatorState {
    pub fn new() -> Self {
        Self {
            operator_id: AtomicU64::new(0),
            last_activity_ms: AtomicU64::new(0),
        }
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Try to claim operator status. Returns true if successful.
    pub fn try_claim(&self, conn_id: u64) -> bool {
        let current = self.operator_id.load(Ordering::SeqCst);
        let now = Self::now_ms();

        if current == 0 || current == conn_id {
            // No operator or we're already the operator
            self.operator_id.store(conn_id, Ordering::SeqCst);
            self.last_activity_ms.store(now, Ordering::SeqCst);
            true
        } else {
            // Check if previous operator timed out
            let last = self.last_activity_ms.load(Ordering::SeqCst);
            if now.saturating_sub(last) > OPERATOR_TIMEOUT_MS {
                // Previous operator timed out, take over
                info!(
                    prev_operator = current,
                    new_operator = conn_id,
                    "Operator takeover due to inactivity timeout"
                );
                self.operator_id.store(conn_id, Ordering::SeqCst);
                self.last_activity_ms.store(now, Ordering::SeqCst);
                true
            } else {
                warn!(
                    current_operator = current,
                    requesting_conn = conn_id,
                    "Operator claim rejected - another operator is active"
                );
                false
            }
        }
    }

    /// Check if this connection is the operator.
    pub fn is_operator(&self, conn_id: u64) -> bool {
        self.operator_id.load(Ordering::SeqCst) == conn_id
    }

    /// Get the current operator ID (0 if none).
    pub fn current_operator(&self) -> u64 {
        self.operator_id.load(Ordering::SeqCst)
    }

    /// Update last activity timestamp (called on any valid command from operator).
    pub fn touch(&self) {
        self.last_activity_ms.store(Self::now_ms(), Ordering::SeqCst);
    }

    /// Release operator status (on disconnect or explicit release).
    pub fn release(&self, conn_id: u64) {
        if self
            .operator_id
            .compare_exchange(conn_id, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!(conn_id, "Operator released");
        }
    }
}

/// WebRTC server configuration.
#[derive(Debug, Clone)]
pub struct RtcConfig {
    /// Port for WebSocket signaling server.
    pub signaling_port: u16,
    /// Telemetry send rate.
    pub telemetry_interval: Duration,
    /// STUN servers for ICE (optional - Tailscale handles NAT).
    pub ice_servers: Vec<String>,
    /// Point cloud streaming configuration.
    pub pointcloud: crate::pointcloud::PointCloudConfig,
}

impl Default for RtcConfig {
    fn default() -> Self {
        Self {
            signaling_port: 4852,
            telemetry_interval: Duration::from_millis(20), // 50Hz telemetry
            ice_servers: vec![
                // Google's public STUN server as fallback
                "stun:stun.l.google.com:19302".to_string(),
            ],
            pointcloud: crate::pointcloud::PointCloudConfig::default(),
        }
    }
}

/// WebRTC teleop server.
pub struct RtcServer {
    config: RtcConfig,
    command_tx: mpsc::Sender<Command>,
    telemetry_rx: watch::Receiver<Telemetry>,
    video_rx: Option<Arc<Mutex<tokio_mpsc::Receiver<H264Frame>>>>,
    keyframe_requestor: Option<Arc<AtomicBool>>,
    lidar_rx: Option<watch::Receiver<Option<PointCloud>>>,
    costmap_rx: Option<watch::Receiver<Option<CostmapSnapshot>>>,
    obstacles_rx: Option<watch::Receiver<Option<Vec<TrackedObstacle>>>>,
    path_rx: Option<watch::Receiver<Option<path_stream::PathSnapshot>>>,
    camera_info: Option<crate::CameraInfo>,
    operator_state: Arc<OperatorState>,
    metrics: Arc<RtcMetrics>,
}

impl RtcServer {
    pub fn new(
        config: RtcConfig,
        command_tx: mpsc::Sender<Command>,
        telemetry_rx: watch::Receiver<Telemetry>,
    ) -> Self {
        Self {
            config,
            command_tx,
            telemetry_rx,
            video_rx: None,
            keyframe_requestor: None,
            lidar_rx: None,
            costmap_rx: None,
            obstacles_rx: None,
            path_rx: None,
            camera_info: None,
            operator_state: Arc::new(OperatorState::new()),
            metrics: RtcMetrics::new(),
        }
    }

    /// Create a new RTC server with H.264 video streaming support.
    pub fn with_video(
        config: RtcConfig,
        command_tx: mpsc::Sender<Command>,
        telemetry_rx: watch::Receiver<Telemetry>,
        video_rx: tokio_mpsc::Receiver<H264Frame>,
    ) -> Self {
        Self {
            config,
            command_tx,
            telemetry_rx,
            video_rx: Some(Arc::new(Mutex::new(video_rx))),
            keyframe_requestor: None,
            lidar_rx: None,
            costmap_rx: None,
            obstacles_rx: None,
            path_rx: None,
            camera_info: None,
            operator_state: Arc::new(OperatorState::new()),
            metrics: RtcMetrics::new(),
        }
    }

    /// Create a new RTC server with H.264 video and LiDAR streaming support.
    pub fn with_video_and_lidar(
        config: RtcConfig,
        command_tx: mpsc::Sender<Command>,
        telemetry_rx: watch::Receiver<Telemetry>,
        video_rx: tokio_mpsc::Receiver<H264Frame>,
        lidar_rx: watch::Receiver<Option<PointCloud>>,
    ) -> Self {
        Self {
            config,
            command_tx,
            telemetry_rx,
            video_rx: Some(Arc::new(Mutex::new(video_rx))),
            keyframe_requestor: None,
            lidar_rx: Some(lidar_rx),
            costmap_rx: None,
            obstacles_rx: None,
            path_rx: None,
            camera_info: None,
            operator_state: Arc::new(OperatorState::new()),
            metrics: RtcMetrics::new(),
        }
    }

    /// Set the keyframe requestor for PLI handling.
    /// Set the keyframe request flag for PLI handling.
    /// When set to `true` by the RTCP reader, the camera capture loop
    /// sends a ForceKeyUnit event to the GStreamer encoder.
    pub fn set_keyframe_requestor(&mut self, flag: Arc<AtomicBool>) {
        self.keyframe_requestor = Some(flag);
    }

    /// Set the costmap receiver for streaming costmap data to the console.
    pub fn set_costmap_rx(&mut self, rx: watch::Receiver<Option<CostmapSnapshot>>) {
        self.costmap_rx = Some(rx);
    }

    /// Set the obstacles receiver for streaming obstacle data to the console.
    pub fn set_obstacles_rx(&mut self, rx: watch::Receiver<Option<Vec<TrackedObstacle>>>) {
        self.obstacles_rx = Some(rx);
    }

    /// Set the path receiver for streaming navigation path data to the console.
    pub fn set_path_rx(&mut self, rx: watch::Receiver<Option<path_stream::PathSnapshot>>) {
        self.path_rx = Some(rx);
    }

    /// Set camera info (sent once per connection on the video channel).
    /// The camera_id, width, and height will be filled from the first video frame.
    pub fn set_camera_info(&mut self, info: crate::CameraInfo) {
        self.camera_info = Some(info);
    }

    /// Get a reference to the operator state (for external monitoring).
    pub fn operator_state(&self) -> Arc<OperatorState> {
        self.operator_state.clone()
    }

    /// Get a reference to the RTC metrics (for external monitoring/push).
    pub fn metrics(&self) -> Arc<RtcMetrics> {
        self.metrics.clone()
    }

    /// Run the WebRTC signaling server.
    pub async fn run(self) -> Result<(), TeleopError> {
        let addr = format!("0.0.0.0:{}", self.config.signaling_port);
        let listener = TcpListener::bind(&addr).await?;
        info!(addr, "WebRTC signaling server listening");

        let command_tx = Arc::new(self.command_tx);
        let telemetry_rx = self.telemetry_rx;
        let video_rx = self.video_rx;
        let keyframe_requestor = self.keyframe_requestor;
        let lidar_rx = self.lidar_rx;
        let costmap_rx = self.costmap_rx;
        let obstacles_rx = self.obstacles_rx;
        let path_rx = self.path_rx;
        let camera_info = self.camera_info.map(Arc::new);
        let config = Arc::new(self.config);
        let operator_state = self.operator_state;
        let metrics = self.metrics;

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    // Assign unique connection ID
                    let conn_id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::SeqCst);
                    info!(%addr, conn_id, "WebRTC signaling client connected");

                    let cmd_tx = command_tx.clone();
                    let telem_rx = telemetry_rx.clone();
                    let vid_rx = video_rx.clone();
                    let kf_req = keyframe_requestor.clone();
                    let lid_rx = lidar_rx.clone();
                    let cm_rx = costmap_rx.clone();
                    let obs_rx = obstacles_rx.clone();
                    let path_rx_clone = path_rx.clone();
                    let cam_info = camera_info.clone();
                    let cfg = config.clone();
                    let op_state = operator_state.clone();
                    let rtc_metrics = metrics.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_signaling(stream, cmd_tx, telem_rx, vid_rx, kf_req, lid_rx, cm_rx, obs_rx, path_rx_clone, cam_info, cfg, conn_id, op_state.clone(), rtc_metrics)
                                .await
                        {
                            error!(?e, conn_id, "WebRTC connection error");
                        }
                        // Release operator status if this connection was the operator
                        op_state.release(conn_id);
                        info!(%addr, conn_id, "WebRTC client disconnected");
                    });
                }
                Err(e) => {
                    error!(?e, "Failed to accept signaling connection");
                }
            }
        }
    }
}

/// Signaling message types.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", content = "data")]
enum SignalingMessage {
    #[serde(rename = "offer")]
    Offer(String),
    #[serde(rename = "answer")]
    Answer(String),
    #[serde(rename = "candidate")]
    Candidate(IceCandidate),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct IceCandidate {
    candidate: String,
    #[serde(rename = "sdpMid")]
    sdp_mid: Option<String>,
    #[serde(rename = "sdpMLineIndex")]
    sdp_mline_index: Option<u16>,
}

async fn handle_signaling(
    stream: TcpStream,
    command_tx: Arc<mpsc::Sender<Command>>,
    telemetry_rx: watch::Receiver<Telemetry>,
    video_rx: Option<Arc<Mutex<tokio_mpsc::Receiver<H264Frame>>>>,
    keyframe_requestor: Option<Arc<AtomicBool>>,
    lidar_rx: Option<watch::Receiver<Option<PointCloud>>>,
    costmap_rx: Option<watch::Receiver<Option<CostmapSnapshot>>>,
    obstacles_rx: Option<watch::Receiver<Option<Vec<TrackedObstacle>>>>,
    path_rx: Option<watch::Receiver<Option<path_stream::PathSnapshot>>>,
    camera_info: Option<Arc<crate::CameraInfo>>,
    config: Arc<RtcConfig>,
    conn_id: u64,
    operator_state: Arc<OperatorState>,
    metrics: Arc<RtcMetrics>,
) -> Result<(), TeleopError> {
    let _ = stream.set_nodelay(true);

    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| TeleopError::Network(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let (ws_sender, mut ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));

    // Create WebRTC API
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs().map_err(|e| {
        TeleopError::Network(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine).map_err(|e| {
        TeleopError::Network(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    // Configure ICE servers
    let ice_servers: Vec<RTCIceServer> = config
        .ice_servers
        .iter()
        .map(|url| RTCIceServer {
            urls: vec![url.clone()],
            ..Default::default()
        })
        .collect();

    let rtc_config = RTCConfiguration {
        ice_servers,
        ..Default::default()
    };

    // Create peer connection
    let peer_connection = Arc::new(api.new_peer_connection(rtc_config).await.map_err(|e| {
        TeleopError::Network(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?);

    // Per-connection LiDAR streaming config (shared with command handler)
    // rate: 0 = disabled, 1-10 = streaming rate in Hz
    // max_points: max points per frame (default from config)
    let lidar_streaming_rate = Arc::new(AtomicU8::new(0));
    let lidar_streaming_max_points = Arc::new(AtomicU16::new(config.pointcloud.max_points as u16));

    // Create command DataChannel as NEGOTIATED for Safari/webrtc-rs compatibility
    // Both sides create the channel with the same ID (0)
    // In webrtc-rs, `negotiated: Some(id)` sets both negotiated mode and channel ID
    let cmd_dc_config = RTCDataChannelInit {
        ordered: Some(true),
        negotiated: Some(0), // Channel ID 0, negotiated mode
        ..Default::default()
    };

    let command_channel = peer_connection
        .create_data_channel("commands", Some(cmd_dc_config))
        .await
        .map_err(|e| {
            TeleopError::Network(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create command channel: {}", e),
            ))
        })?;

    // Create telemetry DataChannel (unreliable, unordered - like UDP)
    // Server creates this channel, browser receives it via ondatachannel
    let dc_config = RTCDataChannelInit {
        ordered: Some(false),
        max_retransmits: Some(0), // No retransmits = truly unreliable
        ..Default::default()
    };

    let telemetry_channel = peer_connection
        .create_data_channel("telemetry", Some(dc_config.clone()))
        .await
        .map_err(|e| {
            TeleopError::Network(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

    // Set up negotiated command channel handler
    let cmd_tx_negotiated = command_tx.clone();
    let op_state_negotiated = operator_state.clone();
    let lidar_rate_negotiated = lidar_streaming_rate.clone();
    let lidar_max_pts_negotiated = lidar_streaming_max_points.clone();

    command_channel.on_open(Box::new(move || {
        info!(conn_id, "WebRTC negotiated command channel opened");
        Box::pin(async {})
    }));

    command_channel.on_message(Box::new(move |msg| {
        let hex: String = msg.data.iter().take(12).map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
        info!(len = msg.data.len(), hex, conn_id, "Command received on negotiated channel");

        let cmd_tx = cmd_tx_negotiated.clone();
        let op_state = op_state_negotiated.clone();
        let lidar_rate = lidar_rate_negotiated.clone();
        let lidar_max_pts = lidar_max_pts_negotiated.clone();
        Box::pin(async move {
            // Check for LiDAR toggle command first
            if msg.data.len() >= CMD_HEADER_SIZE + 1 && msg.data[0] == MSG_LIDAR_TOGGLE {
                let enabled = msg.data[CMD_HEADER_SIZE] != 0;
                // rate_hz in second payload byte; default 5Hz for old 1-byte messages
                let rate_hz = if msg.data.len() >= CMD_HEADER_SIZE + 2 {
                    msg.data[CMD_HEADER_SIZE + 1].clamp(1, 10)
                } else {
                    5
                };
                // max_points as u16 LE in bytes 3-4; default 1000 for old messages
                if msg.data.len() >= CMD_HEADER_SIZE + 4 {
                    let max_pts = u16::from_le_bytes([
                        msg.data[CMD_HEADER_SIZE + 2],
                        msg.data[CMD_HEADER_SIZE + 3],
                    ]).clamp(100, 10000);
                    lidar_max_pts.store(max_pts, Ordering::Relaxed);
                }
                lidar_rate.store(if enabled { rate_hz } else { 0 }, Ordering::Relaxed);
                info!(enabled, rate_hz, max_points = lidar_max_pts.load(Ordering::Relaxed), conn_id, "LiDAR streaming toggled");
                return;
            }

            if let Some(cmd) = parse_command(&msg.data) {
                match filter_command(&cmd, conn_id, &op_state) {
                    CommandFilter::Allow => {
                        match &cmd {
                            Command::SetMode(mode) => {
                                info!(?mode, conn_id, "Mode change accepted");
                            }
                            Command::EStop => {
                                info!(conn_id, "E-Stop command accepted");
                            }
                            _ => {}
                        }
                        // Use try_send to avoid blocking if channel is full.
                        // This prevents cascading stalls when main loop is slow.
                        if let Err(e) = cmd_tx.try_send(cmd) {
                            warn!(?e, conn_id, "Command channel full, dropping WebRTC command");
                        }
                    }
                    CommandFilter::Reject(reason) => {
                        warn!(?cmd, conn_id, reason, "Command rejected by filter");
                    }
                }
            } else {
                let hex: String = msg.data.iter().take(16).map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                warn!(len = msg.data.len(), hex, conn_id, "Failed to parse command");
            }
        })
    }));

    // Set up telemetry sender (also sends camera info once on open)
    let telem_rx_clone = telemetry_rx.clone();
    let telem_interval = config.telemetry_interval;
    let telem_channel = Arc::new(telemetry_channel);
    let telem_channel_clone = telem_channel.clone();
    let telem_metrics = metrics.clone();
    let telem_cam_info = camera_info.clone();

    telem_channel.on_open(Box::new(move || {
        let channel = telem_channel_clone.clone();
        let mut rx = telem_rx_clone.clone();
        let stats = telem_metrics.clone();
        let cam_info = telem_cam_info.clone();
        Box::pin(async move {
            info!("WebRTC telemetry channel opened");

            // Send camera info once at channel open (moved from video DataChannel)
            if let Some(info) = cam_info.as_ref() {
                let msg = serialize_camera_info(info);
                if let Err(e) = channel.send(&bytes::Bytes::from(msg)).await {
                    debug!(?e, "Failed to send camera info on telemetry channel");
                } else {
                    info!(
                        camera_id = info.camera_id,
                        width = info.width,
                        height = info.height,
                        "Camera info sent via telemetry channel"
                    );
                }
            }

            let mut interval = tokio::time::interval(telem_interval);
            loop {
                interval.tick().await;
                if channel.ready_state()
                    != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
                {
                    break;
                }
                let telemetry = rx.borrow_and_update().clone();
                if let Some(data) = serialize_telemetry(&telemetry) {
                    let start = std::time::Instant::now();
                    if let Err(e) = channel.send(&bytes::Bytes::from(data)).await {
                        debug!(?e, "Failed to send telemetry (channel closing?)");
                        break;
                    }
                    let latency_us = start.elapsed().as_micros() as u32;
                    stats.telemetry.record_send(latency_us);
                }
            }
        })
    }));

    // Create H.264 video track if video streaming is enabled
    if let Some(vid_rx) = video_rx {
        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f".to_owned(),
                rtcp_feedback: vec![],
            },
            "camera-0".to_owned(),
            "rover-video".to_owned(),
        ));

        let rtp_sender = peer_connection
            .add_track(video_track.clone() as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|e| {
                TeleopError::Network(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to add video track: {}", e),
                ))
            })?;

        info!("H.264 video track added to peer connection");

        // Spawn RTCP reader to handle PLI (Picture Loss Indication) requests.
        // When the browser detects missing packets, it sends PLI asking for a keyframe.
        if let Some(kf_req) = keyframe_requestor.clone() {
            tokio::spawn(async move {
                loop {
                    match rtp_sender.read_rtcp().await {
                        Ok((packets, _)) => {
                            for packet in &packets {
                                // Check if this is a PLI packet
                                if packet
                                    .as_any()
                                    .downcast_ref::<webrtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication>()
                                    .is_some()
                                {
                                    debug!("PLI received, requesting keyframe");
                                    kf_req.store(true, Ordering::Relaxed);
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // Spawn task to feed H.264 frames to the video track
        let track = video_track.clone();
        let video_metrics = metrics.clone();
        let kf_flag = keyframe_requestor.clone();
        tokio::spawn(async move {
            let mut frame_count: u64 = 0;

            // Drain stale buffered frames before streaming live ones
            {
                let mut drained = 0u32;
                let mut guard = vid_rx.lock().await;
                while guard.try_recv().is_ok() {
                    drained += 1;
                }
                drop(guard);
                if drained > 0 {
                    info!(drained, "Drained stale H.264 frames from buffer");
                }
            }

            // Force a keyframe so the first frame the peer receives is an IDR.
            // Without this, the browser gets P-frames it can't decode and the
            // video stays blank until the next natural keyframe or PLI round-trip.
            if let Some(ref kf) = kf_flag {
                kf.store(true, Ordering::Relaxed);
                debug!("Requested keyframe for new peer");
            }

            loop {
                let frame = {
                    let mut guard = vid_rx.lock().await;
                    guard.recv().await
                };

                let frame = match frame {
                    Some(f) => f,
                    None => {
                        info!("H.264 frame sender dropped, stopping video track feed");
                        break;
                    }
                };

                let sample = Sample {
                    data: bytes::Bytes::from((*frame.data).clone()),
                    duration: Duration::from_nanos(frame.duration_ns),
                    ..Default::default()
                };

                let start = std::time::Instant::now();
                if let Err(e) = track.write_sample(&sample).await {
                    debug!(?e, "Failed to write H.264 sample to track");
                    break;
                }
                let send_us = start.elapsed().as_micros() as u32;
                video_metrics.video.record_send(send_us);

                frame_count += 1;

                if frame_count == 1 {
                    info!(
                        camera_id = frame.camera_id,
                        width = frame.width,
                        height = frame.height,
                        size = frame.data.len(),
                        keyframe = frame.is_keyframe,
                        "First H.264 frame written to track"
                    );
                }
                if frame_count % 300 == 0 {
                    trace!(frame_count, "h264.track_feeding");
                }
            }
            info!(frames_sent = frame_count, "H.264 video track feed stopped");
        });
    }

    // Create pointcloud DataChannel if LiDAR streaming is available
    if let Some(lid_rx) = lidar_rx {
        let pointcloud_dc_config = RTCDataChannelInit {
            ordered: Some(false),
            max_retransmits: Some(0),
            ..Default::default()
        };

        let pointcloud_channel = peer_connection
            .create_data_channel("pointcloud", Some(pointcloud_dc_config))
            .await
            .map_err(|e| {
                TeleopError::Network(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let pointcloud_channel = Arc::new(pointcloud_channel);
        let pointcloud_channel_clone = pointcloud_channel.clone();
        let lidar_rate = lidar_streaming_rate.clone();
        let lidar_max_pts = lidar_streaming_max_points.clone();
        let pc_config = config.pointcloud.clone();
        let pc_metrics = metrics.clone();

        pointcloud_channel.on_open(Box::new(move || {
            let channel = pointcloud_channel_clone.clone();
            let mut rx = lid_rx.clone();
            let rate = lidar_rate.clone();
            let max_pts = lidar_max_pts.clone();
            let stream_config = pc_config.clone();
            let stats = pc_metrics.clone();
            Box::pin(async move {
                info!(
                    voxel_size = stream_config.voxel_size,
                    max_points = stream_config.max_points,
                    "WebRTC pointcloud channel opened"
                );
                let mut frame_count: u64 = 0;

                loop {
                    // Rate 0 = disabled; poll at 10Hz while waiting for enable
                    let rate_hz = rate.load(Ordering::Relaxed);
                    let sleep_ms = if rate_hz > 0 { 1000 / rate_hz as u64 } else { 100 };
                    tokio::time::sleep(Duration::from_millis(sleep_ms)).await;

                    if channel.ready_state()
                        != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
                    {
                        info!("WebRTC pointcloud channel no longer open, stopping");
                        break;
                    }

                    // Only stream when enabled (rate > 0)
                    if rate_hz == 0 {
                        continue;
                    }

                    // Get latest point cloud
                    let cloud = rx.borrow_and_update().clone();
                    if let Some(cloud) = cloud {
                        // Build config with dynamic max_points from operator
                        let mut cfg = stream_config.clone();
                        cfg.max_points = max_pts.load(Ordering::Relaxed) as usize;
                        let data = pointcloud::prepare_for_streaming_with_config(&cloud, &cfg);
                        let start = std::time::Instant::now();
                        if let Err(e) = channel.send(&bytes::Bytes::from(data)).await {
                            debug!(?e, "Failed to send pointcloud (channel closing?)");
                            break;
                        }
                        let latency_us = start.elapsed().as_micros() as u32;
                        stats.pointcloud.record_send(latency_us);

                        frame_count += 1;
                        if frame_count == 1 {
                            info!(
                                points = cloud.points.len(),
                                downsampled = stream_config.max_points,
                                "First pointcloud sent via WebRTC"
                            );
                        }
                        if frame_count % 50 == 0 {
                            trace!(frame_count, "pointcloud.streaming");
                        }
                    }
                }
                info!(frames_sent = frame_count, "WebRTC pointcloud channel closed");
            })
        }));
    }

    // Create costmap DataChannel if costmap streaming is available
    if let Some(cm_rx) = costmap_rx {
        let costmap_dc_config = RTCDataChannelInit {
            ordered: Some(false),
            max_retransmits: Some(0),
            ..Default::default()
        };

        let costmap_channel = peer_connection
            .create_data_channel("costmap", Some(costmap_dc_config))
            .await
            .map_err(|e| {
                TeleopError::Network(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let costmap_channel = Arc::new(costmap_channel);
        let costmap_channel_clone = costmap_channel.clone();
        let cm_metrics = metrics.clone();

        costmap_channel.on_open(Box::new(move || {
            let channel = costmap_channel_clone.clone();
            let mut rx = cm_rx.clone();
            let stats = cm_metrics.clone();
            Box::pin(async move {
                info!("WebRTC costmap channel opened");
                // Stream at 2Hz (500ms interval) — costmap updates at ~10Hz internally
                // but 2Hz is plenty for visualization
                let mut interval = tokio::time::interval(Duration::from_millis(500));
                let mut frame_count: u64 = 0;

                loop {
                    interval.tick().await;

                    if channel.ready_state()
                        != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
                    {
                        info!("WebRTC costmap channel no longer open, stopping");
                        break;
                    }

                    // Get latest costmap snapshot
                    let snapshot = rx.borrow_and_update().clone();
                    if let Some(snapshot) = snapshot {
                        let data = costmap_stream::serialize(&snapshot);
                        let start = std::time::Instant::now();
                        if let Err(e) = channel.send(&bytes::Bytes::from(data)).await {
                            debug!(?e, "Failed to send costmap (channel closing?)");
                            break;
                        }
                        let latency_us = start.elapsed().as_micros() as u32;
                        stats.costmap.record_send(latency_us);

                        frame_count += 1;
                        if frame_count == 1 {
                            info!(
                                width = snapshot.width,
                                height = snapshot.height,
                                resolution = snapshot.resolution,
                                "First costmap sent via WebRTC"
                            );
                        }
                        if frame_count % 20 == 0 {
                            trace!(frame_count, "costmap.streaming");
                        }
                    }
                }
                info!(frames_sent = frame_count, "WebRTC costmap channel closed");
            })
        }));
    }

    // Create obstacles DataChannel if obstacle streaming is available
    if let Some(obs_rx) = obstacles_rx {
        let obstacles_dc_config = RTCDataChannelInit {
            ordered: Some(false),
            max_retransmits: Some(0),
            ..Default::default()
        };

        let obstacles_channel = peer_connection
            .create_data_channel("obstacles", Some(obstacles_dc_config))
            .await
            .map_err(|e| {
                TeleopError::Network(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let obstacles_channel = Arc::new(obstacles_channel);
        let obstacles_channel_clone = obstacles_channel.clone();
        let obs_metrics = metrics.clone();

        obstacles_channel.on_open(Box::new(move || {
            let channel = obstacles_channel_clone.clone();
            let mut rx = obs_rx.clone();
            let stats = obs_metrics.clone();
            Box::pin(async move {
                info!("WebRTC obstacles channel opened");
                // Stream at 10Hz for smooth client-side interpolation
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                let mut frame_count: u64 = 0;

                loop {
                    interval.tick().await;

                    if channel.ready_state()
                        != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
                    {
                        info!("WebRTC obstacles channel no longer open, stopping");
                        break;
                    }

                    // Get latest obstacles
                    let obstacles = rx.borrow_and_update().clone();
                    if let Some(obstacles) = obstacles {
                        let data = obstacle_stream::serialize_tracked(&obstacles);
                        let start = std::time::Instant::now();
                        if let Err(e) = channel.send(&bytes::Bytes::from(data)).await {
                            debug!(?e, "Failed to send obstacles (channel closing?)");
                            break;
                        }
                        let latency_us = start.elapsed().as_micros() as u32;
                        stats.obstacles.record_send(latency_us);

                        frame_count += 1;
                        if frame_count == 1 {
                            info!(
                                count = obstacles.len(),
                                "First obstacles sent via WebRTC"
                            );
                        }
                        if frame_count % 20 == 0 {
                            trace!(frame_count, "obstacles.streaming");
                        }
                    }
                }
                info!(frames_sent = frame_count, "WebRTC obstacles channel closed");
            })
        }));
    }

    // Create path DataChannel if path streaming is available
    if let Some(path_rx) = path_rx {
        let path_dc_config = RTCDataChannelInit {
            ordered: Some(false),
            max_retransmits: Some(0),
            ..Default::default()
        };

        let path_channel = peer_connection
            .create_data_channel("path", Some(path_dc_config))
            .await
            .map_err(|e| {
                TeleopError::Network(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let path_channel = Arc::new(path_channel);
        let path_channel_clone = path_channel.clone();
        let path_metrics = metrics.clone();

        path_channel.on_open(Box::new(move || {
            let channel = path_channel_clone.clone();
            let mut rx = path_rx.clone();
            let stats = path_metrics.clone();
            Box::pin(async move {
                info!("WebRTC path channel opened");
                // Stream at 2Hz — path changes infrequently
                let mut interval = tokio::time::interval(Duration::from_millis(500));
                let mut frame_count: u64 = 0;

                loop {
                    interval.tick().await;

                    if channel.ready_state()
                        != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
                    {
                        info!("WebRTC path channel no longer open, stopping");
                        break;
                    }

                    // Get latest path snapshot
                    let snapshot = rx.borrow_and_update().clone();
                    if let Some(snapshot) = snapshot {
                        let data = path_stream::serialize(&snapshot);
                        let start = std::time::Instant::now();
                        if let Err(e) = channel.send(&bytes::Bytes::from(data)).await {
                            debug!(?e, "Failed to send path (channel closing?)");
                            break;
                        }
                        let latency_us = start.elapsed().as_micros() as u32;
                        stats.path.record_send(latency_us);

                        frame_count += 1;
                        if frame_count == 1 {
                            info!(
                                waypoints = snapshot.waypoints.len(),
                                state = snapshot.state,
                                "First path sent via WebRTC"
                            );
                        }
                        if frame_count % 20 == 0 {
                            trace!(frame_count, "path.streaming");
                        }
                    }
                }
                info!(frames_sent = frame_count, "WebRTC path channel closed");
            })
        }));
    }

    // Handle incoming DataChannels from browser (browser creates "commands" channel)
    let cmd_tx_clone = command_tx.clone();
    let op_state_clone = operator_state.clone();
    let lidar_rate_clone = lidar_streaming_rate.clone();
    let lidar_max_pts_clone = lidar_streaming_max_points.clone();
    peer_connection.on_data_channel(Box::new(move |channel| {
        let cmd_tx = cmd_tx_clone.clone();
        let op_state = op_state_clone.clone();
        let lidar_rate = lidar_rate_clone.clone();
        let lidar_max_pts = lidar_max_pts_clone.clone();
        let label = channel.label().to_string();

        // Commands channel is negotiated (both sides create it), so it won't appear here
        // Log any unexpected channels from browser
        if label == "commands" {
            warn!(conn_id, "Received non-negotiated commands channel from browser - ignoring");
        } else {
            info!(label, conn_id, "WebRTC received data channel from browser");
        }

        // Suppress unused variable warnings for cloned values
        let _ = (&cmd_tx, &op_state, &lidar_rate, &lidar_max_pts);

        Box::pin(async {})
    }));

    // Handle ICE candidates
    let ws_sender_ice = ws_sender.clone();
    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let ws = ws_sender_ice.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                let msg = SignalingMessage::Candidate(IceCandidate {
                    candidate: c.to_json().map(|j| j.candidate).unwrap_or_default(),
                    sdp_mid: c.to_json().ok().and_then(|j| j.sdp_mid),
                    sdp_mline_index: c.to_json().ok().and_then(|j| j.sdp_mline_index),
                });
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = ws.lock().await.send(Message::Text(json.into())).await;
                }
            }
        })
    }));

    // Handle connection state changes
    let pc_clone = peer_connection.clone();
    let op_state_dc = operator_state.clone();
    peer_connection.on_peer_connection_state_change(Box::new(move |state| {
        info!(?state, conn_id, "WebRTC peer connection state changed");
        let pc = pc_clone.clone();
        let op_state = op_state_dc.clone();
        Box::pin(async move {
            if state == RTCPeerConnectionState::Failed
                || state == RTCPeerConnectionState::Disconnected
                || state == RTCPeerConnectionState::Closed
            {
                op_state.release(conn_id);
                let _ = pc.close().await;
            }
        })
    }));

    // Process signaling messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(signal) = serde_json::from_str::<SignalingMessage>(&text) {
                    match signal {
                        SignalingMessage::Offer(sdp) => {
                            debug!("Received SDP offer");
                            let offer = RTCSessionDescription::offer(sdp).map_err(|e| {
                                TeleopError::Network(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    e.to_string(),
                                ))
                            })?;

                            peer_connection.set_remote_description(offer).await.map_err(
                                |e| {
                                    TeleopError::Network(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        e.to_string(),
                                    ))
                                },
                            )?;

                            let answer =
                                peer_connection.create_answer(None).await.map_err(|e| {
                                    TeleopError::Network(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        e.to_string(),
                                    ))
                                })?;

                            peer_connection
                                .set_local_description(answer.clone())
                                .await
                                .map_err(|e| {
                                    TeleopError::Network(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        e.to_string(),
                                    ))
                                })?;

                            let response = SignalingMessage::Answer(answer.sdp);
                            if let Ok(json) = serde_json::to_string(&response) {
                                ws_sender
                                    .lock()
                                    .await
                                    .send(Message::Text(json.into()))
                                    .await
                                    .ok();
                            }
                            debug!("Sent SDP answer");
                        }
                        SignalingMessage::Answer(_) => {
                            // Server doesn't expect answers (it sends offers)
                            warn!("Unexpected SDP answer from client");
                        }
                        SignalingMessage::Candidate(ice) => {
                            debug!("Received ICE candidate");
                            let candidate = webrtc::ice_transport::ice_candidate::RTCIceCandidateInit {
                                candidate: ice.candidate,
                                sdp_mid: ice.sdp_mid,
                                sdp_mline_index: ice.sdp_mline_index,
                                ..Default::default()
                            };
                            if let Err(e) = peer_connection.add_ice_candidate(candidate).await {
                                warn!(?e, "Failed to add ICE candidate");
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Signaling WebSocket closed");
                break;
            }
            Err(e) => {
                warn!(?e, "Signaling WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Clean up
    let _ = peer_connection.close().await;
    Ok(())
}

/// Result of command filtering.
enum CommandFilter {
    Allow,
    Reject(&'static str),
}

/// Filter commands based on operator status.
///
/// Rules:
/// - EStop: Always allowed (safety critical)
/// - Heartbeat: Always allowed (keepalive)
/// - SetMode(Disabled/Idle): Always allowed (safe to disable)
/// - SetMode(Teleop): Requires claiming operator status
/// - SetMode(Autonomous): Allowed from anyone (releases operator, rover drives itself)
/// - Twist, Tool, EStopRelease: Operator only
fn filter_command(cmd: &Command, conn_id: u64, op_state: &OperatorState) -> CommandFilter {
    match cmd {
        // Safety: Anyone can e-stop
        Command::EStop => CommandFilter::Allow,

        // Keepalive: Always allowed
        Command::Heartbeat => {
            // If this connection is the operator, update activity timestamp
            if op_state.is_operator(conn_id) {
                op_state.touch();
            }
            CommandFilter::Allow
        }

        // Mode changes
        Command::SetMode(mode) => match mode {
            // Safe to disable/sleep from anyone
            Mode::Disabled | Mode::Idle | Mode::Sleep => {
                // If operator is disabling/sleeping, release operator status
                if op_state.is_operator(conn_id) {
                    op_state.release(conn_id);
                }
                CommandFilter::Allow
            }
            // Teleop requires claiming operator status
            Mode::Teleop => {
                if op_state.try_claim(conn_id) {
                    info!(conn_id, "Operator claimed");
                    CommandFilter::Allow
                } else {
                    CommandFilter::Reject("another operator is active")
                }
            }
            // Autonomous: anyone can request (rover drives itself, no operator needed)
            Mode::Autonomous => {
                // Release operator if held — autonomous doesn't need one
                if op_state.is_operator(conn_id) {
                    op_state.release(conn_id);
                }
                CommandFilter::Allow
            }
            // E-Stop mode shouldn't be set directly
            Mode::EStop => CommandFilter::Reject("cannot set EStop mode directly"),
            // Fault mode shouldn't be set directly
            Mode::Fault => CommandFilter::Reject("cannot set Fault mode directly"),
        },

        // Movement commands: Operator only
        Command::Twist(_) => {
            if op_state.is_operator(conn_id) {
                op_state.touch();
                CommandFilter::Allow
            } else {
                CommandFilter::Reject("not the operator")
            }
        }

        // Tool commands: Operator only
        Command::Tool(_) => {
            if op_state.is_operator(conn_id) {
                op_state.touch();
                CommandFilter::Allow
            } else {
                CommandFilter::Reject("not the operator")
            }
        }

        // E-Stop release: Operator only (deliberate action)
        Command::EStopRelease => {
            if op_state.is_operator(conn_id) {
                op_state.touch();
                CommandFilter::Allow
            } else {
                CommandFilter::Reject("not the operator")
            }
        }

        // SetGoal: anyone can set a goal (like Autonomous mode request)
        Command::SetGoal { .. } => CommandFilter::Allow,

        // LidarToggle is handled per-connection, not forwarded through command channel
        // This case should never be reached since it's intercepted in on_message
        Command::LidarToggle(_) => CommandFilter::Reject("handled separately"),
    }
}

/// Parse a command from raw bytes.
///
/// Binary format:
/// [0]     u8      msg_type
/// [1]     u8      flags
/// [2-3]   u16 LE  sequence
/// [4-7]   u32 LE  timestamp_ms
/// [8...]          payload
fn parse_command(data: &[u8]) -> Option<Command> {
    let header = CommandHeader::parse(data)?;
    let payload = &data[CMD_HEADER_SIZE..];

    match header.msg_type {
        // Twist: [linear:f32] [angular:f32] [boost:u8]
        MSG_TWIST if payload.len() >= 9 => {
            let linear = f32::from_le_bytes(payload[0..4].try_into().ok()?) as f64;
            let angular = f32::from_le_bytes(payload[4..8].try_into().ok()?) as f64;
            let boost = payload[8] != 0;
            Some(Command::Twist(Twist {
                linear,
                angular,
                boost,
            }))
        }
        // E-Stop (no payload)
        MSG_ESTOP => Some(Command::EStop),
        // Heartbeat (no payload)
        MSG_HEARTBEAT => Some(Command::Heartbeat),
        // E-Stop Release (no payload)
        MSG_ESTOP_RELEASE => Some(Command::EStopRelease),
        // SetMode: [mode:u8]
        MSG_SET_MODE if !payload.is_empty() => {
            let mode = match payload[0] {
                0 => Mode::Disabled,
                1 => Mode::Idle,
                2 => Mode::Teleop,
                3 => Mode::Autonomous,
                6 => Mode::Sleep,
                _ => return None,
            };
            debug!(?mode, seq = header.sequence, "SetMode command");
            Some(Command::SetMode(mode))
        }
        // Tool: [axis:f32] [motor:f32] [action_a:u8] [action_b:u8]
        MSG_TOOL if payload.len() >= 10 => {
            let axis = f32::from_le_bytes(payload[0..4].try_into().ok()?);
            let motor = f32::from_le_bytes(payload[4..8].try_into().ok()?);
            let action_a = payload[8] != 0;
            let action_b = payload[9] != 0;
            Some(Command::Tool(ToolCommand {
                axis,
                motor,
                action_a,
                action_b,
            }))
        }
        MSG_SET_GOAL if payload.len() >= 16 => {
            let x = f64::from_le_bytes(payload[0..8].try_into().ok()?);
            let y = f64::from_le_bytes(payload[8..16].try_into().ok()?);
            Some(Command::SetGoal { x, y })
        }
        _ => None,
    }
}
