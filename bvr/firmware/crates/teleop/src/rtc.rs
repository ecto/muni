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

use crate::video::VideoFrame;
use crate::{
    CommandHeader, Telemetry, TeleopError, CMD_HEADER_SIZE, MSG_ESTOP, MSG_ESTOP_RELEASE,
    MSG_HEARTBEAT, MSG_SET_MODE, MSG_TELEMETRY, MSG_TOOL, MSG_TWIST,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::sync::mpsc as tokio_mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, trace, warn};
use types::{Command, Mode, ToolCommand, Twist};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

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
}

impl Default for RtcConfig {
    fn default() -> Self {
        Self {
            signaling_port: 4852,
            telemetry_interval: Duration::from_millis(50), // 20Hz telemetry
            ice_servers: vec![
                // Google's public STUN server as fallback
                "stun:stun.l.google.com:19302".to_string(),
            ],
        }
    }
}

/// WebRTC teleop server.
pub struct RtcServer {
    config: RtcConfig,
    command_tx: mpsc::Sender<Command>,
    telemetry_rx: watch::Receiver<Telemetry>,
    video_rx: Option<Arc<Mutex<tokio_mpsc::Receiver<VideoFrame>>>>,
    operator_state: Arc<OperatorState>,
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
            operator_state: Arc::new(OperatorState::new()),
        }
    }

    /// Create a new RTC server with video streaming support.
    pub fn with_video(
        config: RtcConfig,
        command_tx: mpsc::Sender<Command>,
        telemetry_rx: watch::Receiver<Telemetry>,
        video_rx: tokio_mpsc::Receiver<VideoFrame>,
    ) -> Self {
        Self {
            config,
            command_tx,
            telemetry_rx,
            video_rx: Some(Arc::new(Mutex::new(video_rx))),
            operator_state: Arc::new(OperatorState::new()),
        }
    }

    /// Get a reference to the operator state (for external monitoring).
    pub fn operator_state(&self) -> Arc<OperatorState> {
        self.operator_state.clone()
    }

    /// Run the WebRTC signaling server.
    pub async fn run(self) -> Result<(), TeleopError> {
        let addr = format!("0.0.0.0:{}", self.config.signaling_port);
        let listener = TcpListener::bind(&addr).await?;
        info!(addr, "WebRTC signaling server listening");

        let command_tx = Arc::new(self.command_tx);
        let telemetry_rx = self.telemetry_rx;
        let video_rx = self.video_rx;
        let config = Arc::new(self.config);
        let operator_state = self.operator_state;

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    // Assign unique connection ID
                    let conn_id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::SeqCst);
                    info!(%addr, conn_id, "WebRTC signaling client connected");

                    let cmd_tx = command_tx.clone();
                    let telem_rx = telemetry_rx.clone();
                    let vid_rx = video_rx.clone();
                    let cfg = config.clone();
                    let op_state = operator_state.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_signaling(stream, cmd_tx, telem_rx, vid_rx, cfg, conn_id, op_state.clone())
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
    video_rx: Option<Arc<Mutex<tokio_mpsc::Receiver<VideoFrame>>>>,
    config: Arc<RtcConfig>,
    conn_id: u64,
    operator_state: Arc<OperatorState>,
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

    // Set up telemetry sender
    let telem_rx_clone = telemetry_rx.clone();
    let telem_interval = config.telemetry_interval;
    let telem_channel = Arc::new(telemetry_channel);
    let telem_channel_clone = telem_channel.clone();

    telem_channel.on_open(Box::new(move || {
        let channel = telem_channel_clone.clone();
        let mut rx = telem_rx_clone.clone();
        Box::pin(async move {
            info!("WebRTC telemetry channel opened");
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
                    if let Err(e) = channel.send(&bytes::Bytes::from(data)).await {
                        debug!(?e, "Failed to send telemetry (channel closing?)");
                        break;
                    }
                }
            }
        })
    }));

    // Create video DataChannel if video streaming is enabled
    if let Some(vid_rx) = video_rx {
        let video_channel = peer_connection
            .create_data_channel("video", Some(dc_config))
            .await
            .map_err(|e| {
                TeleopError::Network(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let video_channel = Arc::new(video_channel);
        let video_channel_clone = video_channel.clone();

        video_channel.on_open(Box::new(move || {
            let channel = video_channel_clone.clone();
            let rx = vid_rx.clone();
            Box::pin(async move {
                info!("WebRTC video channel opened, waiting for camera frames");
                let mut frame_count: u64 = 0;
                let mut last_warning = std::time::Instant::now();
                loop {
                    if channel.ready_state()
                        != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
                    {
                        info!("WebRTC video channel no longer open, stopping");
                        break;
                    }

                    // Wait for next frame from mpsc channel with timeout
                    let frame = tokio::select! {
                        result = async {
                            rx.lock().await.recv().await
                        } => {
                            match result {
                                Some(f) => f,
                                None => {
                                    info!("Video frame sender dropped, stopping");
                                    break;
                                }
                            }
                        }
                        _ = tokio::time::sleep(Duration::from_secs(5)) => {
                            if frame_count == 0 && last_warning.elapsed() > Duration::from_secs(5) {
                                warn!("No video frames received after 5s - is camera connected?");
                                last_warning = std::time::Instant::now();
                            }
                            continue;
                        }
                    };

                    // Encode frame: [0x20] [camera_id:u8] [timestamp:u64] [width:u16] [height:u16] [jpeg...]
                    let encode_start = std::time::Instant::now();
                    let frame_size = frame.data.len();
                    let mut buf = Vec::with_capacity(14 + frame_size);
                    buf.push(0x20); // Video frame marker
                    buf.push(frame.camera_id);
                    buf.extend_from_slice(&frame.timestamp_ms.to_le_bytes());
                    buf.extend_from_slice(&(frame.width as u16).to_le_bytes());
                    buf.extend_from_slice(&(frame.height as u16).to_le_bytes());
                    buf.extend_from_slice(&frame.data);
                    let encode_us = encode_start.elapsed().as_micros();

                    let send_start = std::time::Instant::now();
                    if let Err(e) = channel.send(&bytes::Bytes::from(buf)).await {
                        debug!(?e, "Failed to send video frame (channel closing?)");
                        break;
                    }
                    let send_us = send_start.elapsed().as_micros();

                    frame_count += 1;

                    // Log every 100th frame for performance tracking
                    if frame_count % 100 == 0 {
                        trace!(
                            frame_count,
                            frame_size,
                            encode_us,
                            send_us,
                            camera_id = frame.camera_id,
                            "rtc.video_frame_sent"
                        );
                    }
                    if frame_count == 1 {
                        info!(
                            camera_id = frame.camera_id,
                            width = frame.width,
                            height = frame.height,
                            size = frame.data.len(),
                            "First video frame sent via WebRTC"
                        );
                    }
                }
                info!(frames_sent = frame_count, "WebRTC video channel closed");
            })
        }));
    }

    // Handle incoming DataChannels from browser (browser creates "commands" channel)
    let cmd_tx_clone = command_tx.clone();
    let op_state_clone = operator_state.clone();
    peer_connection.on_data_channel(Box::new(move |channel| {
        let cmd_tx = cmd_tx_clone.clone();
        let op_state = op_state_clone.clone();
        let label = channel.label().to_string();

        info!(label, conn_id, "WebRTC received data channel from browser");

        if label == "commands" {
            // Set up command handler SYNCHRONOUSLY to avoid missing early messages
            channel.on_open(Box::new(move || {
                info!(conn_id, "WebRTC command channel (from browser) opened");
                Box::pin(async {})
            }));

            channel.on_message(Box::new(move |msg| {
                let cmd_tx = cmd_tx.clone();
                let op_state = op_state.clone();
                Box::pin(async move {
                    if let Some(cmd) = parse_command(&msg.data) {
                        // Filter commands based on operator status
                        match filter_command(&cmd, conn_id, &op_state) {
                            CommandFilter::Allow => {
                                // Only log mode changes and important commands
                                match &cmd {
                                    Command::SetMode(mode) => {
                                        info!(?mode, conn_id, "Mode change accepted");
                                    }
                                    Command::EStop => {
                                        info!(conn_id, "E-Stop command accepted");
                                    }
                                    _ => {}
                                }
                                let _ = cmd_tx.send(cmd).await;
                            }
                            CommandFilter::Reject(reason) => {
                                debug!(?cmd, conn_id, reason, "Command rejected");
                            }
                        }
                    } else {
                        warn!(len = msg.data.len(), conn_id, "Failed to parse WebRTC command");
                    }
                })
            }));
        }

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
    peer_connection.on_peer_connection_state_change(Box::new(move |state| {
        info!(?state, "WebRTC peer connection state changed");
        let pc = pc_clone.clone();
        Box::pin(async move {
            if state == RTCPeerConnectionState::Failed
                || state == RTCPeerConnectionState::Disconnected
                || state == RTCPeerConnectionState::Closed
            {
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
/// - SetMode(Autonomous): Operator only
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
            // Safe to disable from anyone
            Mode::Disabled | Mode::Idle => {
                // If operator is disabling, release operator status
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
            // Autonomous mode is operator-only
            Mode::Autonomous => {
                if op_state.is_operator(conn_id) {
                    op_state.touch();
                    CommandFilter::Allow
                } else {
                    CommandFilter::Reject("not the operator")
                }
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
        _ => None,
    }
}

/// Serialize telemetry for transmission (120 bytes).
///
/// Binary format:
/// ```text
/// [0]       u8      msg_type (0x11)
/// [1]       u8      mode
/// [2-3]     u16 LE  sequence
/// [4-7]     padding
/// [8-31]    f64×3   pose (x, y, theta)
/// [32-39]   f64     battery_voltage
/// [40-47]   u64 LE  timestamp_us (monotonic)
/// [48-55]   f32×2   cmd_velocity (linear, angular)
/// [56-63]   f32×2   meas_velocity (linear, angular)
/// [64-71]   f32×2   acceleration (linear, angular)
/// [72-87]   f32×4   motor_temps
/// [88-103]  f32×4   motor_currents
/// [104-105] u16 LE  health_bits
/// [106-107] u16 LE  odometry_quality
/// [108-111] f32 LE  dt_ms
/// [112-113] u16 LE  last_cmd_seq (ACK)
/// [114-115] u16 LE  ack_bits
/// [116-119] u32     crc32
/// ```
fn serialize_telemetry(telemetry: &Telemetry) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity(120);

    // Header
    buf.push(MSG_TELEMETRY);            // [0]
    buf.push(telemetry.mode as u8);     // [1]
    buf.extend_from_slice(&telemetry.sequence.to_le_bytes()); // [2-3]
    buf.extend_from_slice(&[0u8; 4]);   // [4-7] padding

    // Pose (x, y, theta) - 24 bytes [8-31]
    buf.extend_from_slice(&telemetry.pose.x.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.y.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.theta.to_le_bytes());

    // Battery voltage [32-39]
    buf.extend_from_slice(&telemetry.power.battery_voltage.to_le_bytes());

    // Timestamp [40-47]
    buf.extend_from_slice(&telemetry.timestamp_us.to_le_bytes());

    // Commanded velocity [48-55]
    buf.extend_from_slice(&(telemetry.cmd_velocity.linear as f32).to_le_bytes());
    buf.extend_from_slice(&(telemetry.cmd_velocity.angular as f32).to_le_bytes());

    // Measured velocity [56-63]
    buf.extend_from_slice(&telemetry.meas_velocity.0.to_le_bytes());
    buf.extend_from_slice(&telemetry.meas_velocity.1.to_le_bytes());

    // Acceleration [64-71]
    buf.extend_from_slice(&telemetry.acceleration.0.to_le_bytes());
    buf.extend_from_slice(&telemetry.acceleration.1.to_le_bytes());

    // Motor temps [72-87]
    for temp in &telemetry.motor_temps {
        buf.extend_from_slice(&temp.to_le_bytes());
    }

    // Motor currents [88-103]
    for current in &telemetry.motor_currents {
        buf.extend_from_slice(&current.to_le_bytes());
    }

    // Health bits [104-105]
    buf.extend_from_slice(&telemetry.health.to_bits().to_le_bytes());

    // Odometry quality [106-107]
    buf.extend_from_slice(&telemetry.odometry_quality.to_le_bytes());

    // dt_ms [108-111]
    buf.extend_from_slice(&telemetry.dt_ms.to_le_bytes());

    // ACK fields [112-115]
    buf.extend_from_slice(&telemetry.last_cmd_seq.to_le_bytes());
    buf.extend_from_slice(&telemetry.ack_bits.to_le_bytes());

    // CRC32 [116-119]
    let crc = crc32fast::hash(&buf);
    buf.extend_from_slice(&crc.to_le_bytes());

    debug_assert_eq!(buf.len(), 120);
    Some(buf)
}
