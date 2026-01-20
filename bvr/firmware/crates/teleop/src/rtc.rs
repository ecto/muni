//! WebRTC-based teleop server for low-latency, unreliable command delivery.
//!
//! Uses WebRTC DataChannels configured for unreliable, unordered delivery
//! (essentially UDP over the web). This eliminates TCP's head-of-line blocking
//! and retransmission delays that cause jerky teleop.
//!
//! Architecture:
//! - WebSocket is used only for signaling (SDP offer/answer, ICE candidates)
//! - DataChannel "commands" carries teleop commands (unreliable)
//! - DataChannel "telemetry" sends rover state back (unreliable)
//!
//! Operator Exclusivity:
//! - Only one connection can be the "operator" at a time
//! - Operator is claimed via SetMode(Teleop) command
//! - Movement commands (Twist, Tool) only accepted from operator
//! - Safety commands (EStop) accepted from anyone
//! - Operator status released on disconnect or after 5s inactivity

use crate::video::VideoFrame;
use crate::{Telemetry, TeleopError};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch, Mutex};
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
    video_rx: Option<watch::Receiver<Option<VideoFrame>>>,
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
        video_rx: watch::Receiver<Option<VideoFrame>>,
    ) -> Self {
        Self {
            config,
            command_tx,
            telemetry_rx,
            video_rx: Some(video_rx),
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
    video_rx: Option<watch::Receiver<Option<VideoFrame>>>,
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
        let video_interval = Duration::from_millis(33); // ~30fps max

        video_channel.on_open(Box::new(move || {
            let channel = video_channel_clone.clone();
            let mut rx = vid_rx.clone();
            Box::pin(async move {
                info!("WebRTC video channel opened");
                let mut interval = tokio::time::interval(video_interval);
                loop {
                    interval.tick().await;
                    if channel.ready_state()
                        != webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
                    {
                        break;
                    }

                    // Wait for new frame
                    if rx.changed().await.is_err() {
                        break;
                    }

                    let frame = rx.borrow_and_update().clone();
                    if let Some(frame) = frame {
                        // Encode frame: [0x20] [timestamp:u64] [width:u16] [height:u16] [jpeg...]
                        let mut buf = Vec::with_capacity(13 + frame.data.len());
                        buf.push(0x20); // Video frame marker
                        buf.extend_from_slice(&frame.timestamp_ms.to_le_bytes());
                        buf.extend_from_slice(&(frame.width as u16).to_le_bytes());
                        buf.extend_from_slice(&(frame.height as u16).to_le_bytes());
                        buf.extend_from_slice(&frame.data);

                        if let Err(e) = channel.send(&bytes::Bytes::from(buf)).await {
                            debug!(?e, "Failed to send video frame (channel closing?)");
                            break;
                        }
                    }
                }
                info!("WebRTC video channel closed");
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

        Box::pin(async move {
            info!(label, conn_id, "WebRTC received data channel from browser");

            if label == "commands" {
                // Set up command handler for browser's commands channel
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
                                    trace!(?cmd, conn_id, "WebRTC command accepted");
                                    let _ = cmd_tx.send(cmd).await;
                                }
                                CommandFilter::Reject(reason) => {
                                    debug!(?cmd, conn_id, reason, "WebRTC command rejected");
                                }
                            }
                        } else {
                            warn!(len = msg.data.len(), conn_id, "Failed to parse WebRTC command");
                        }
                    })
                }));
            }
        })
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

/// Parse a command from raw bytes (same format as UDP/WebSocket).
fn parse_command(data: &[u8]) -> Option<Command> {
    if data.is_empty() {
        return None;
    }

    match data[0] {
        // Twist command
        0x01 if data.len() >= 17 => {
            let linear = f64::from_le_bytes(data[1..9].try_into().ok()?);
            let angular = f64::from_le_bytes(data[9..17].try_into().ok()?);
            let boost = data.get(17).map(|&b| b != 0).unwrap_or(false);
            Some(Command::Twist(Twist {
                linear,
                angular,
                boost,
            }))
        }
        // E-Stop
        0x02 => Some(Command::EStop),
        // Heartbeat
        0x03 => Some(Command::Heartbeat),
        // E-Stop Release
        0x06 => Some(Command::EStopRelease),
        // Set mode
        0x04 if data.len() >= 2 => {
            let mode = match data[1] {
                0 => Mode::Disabled,
                1 => Mode::Idle,
                2 => Mode::Teleop,
                3 => Mode::Autonomous,
                _ => return None,
            };
            tracing::warn!(
                mode_byte = data[1],
                data_len = data.len(),
                ?mode,
                "SetMode command parsed from WebRTC"
            );
            Some(Command::SetMode(mode))
        }
        // Tool command
        0x05 if data.len() >= 11 => {
            let axis = f32::from_le_bytes(data[1..5].try_into().ok()?);
            let motor = f32::from_le_bytes(data[5..9].try_into().ok()?);
            let action_a = data.get(9).map(|&b| b != 0).unwrap_or(false);
            let action_b = data.get(10).map(|&b| b != 0).unwrap_or(false);
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

/// Serialize telemetry for transmission (same format as UDP/WebSocket).
fn serialize_telemetry(telemetry: &Telemetry) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity(128);

    buf.push(0x10); // Telemetry message type
    buf.push(telemetry.mode as u8);

    // Pose (x, y, theta) - 24 bytes
    buf.extend_from_slice(&telemetry.pose.x.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.y.to_le_bytes());
    buf.extend_from_slice(&telemetry.pose.theta.to_le_bytes());

    // Power
    buf.extend_from_slice(&telemetry.power.battery_voltage.to_le_bytes());

    // Timestamp
    buf.extend_from_slice(&telemetry.timestamp_ms.to_le_bytes());

    // Velocity
    buf.extend_from_slice(&telemetry.velocity.linear.to_le_bytes());
    buf.extend_from_slice(&telemetry.velocity.angular.to_le_bytes());

    // Motor temps
    for temp in &telemetry.motor_temps {
        buf.extend_from_slice(&temp.to_le_bytes());
    }

    // Motor currents
    for current in &telemetry.motor_currents {
        buf.extend_from_slice(&current.to_le_bytes());
    }

    // Subsystem health (2 bytes)
    buf.extend_from_slice(&telemetry.health.to_bits().to_le_bytes());

    Some(buf)
}
