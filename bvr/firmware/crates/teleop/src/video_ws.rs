//! WebSocket video streaming for browser-based operators.
//!
//! Streams JPEG frames from the camera (including 360° Insta360) over WebSocket
//! for display in the web-based Operator app.

use crate::video::VideoFrame;
use crate::TeleopError;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// WebSocket video server configuration.
#[derive(Debug, Clone)]
pub struct WsVideoConfig {
    pub port: u16,
    /// Maximum frame rate to send (throttle if camera is faster)
    pub max_fps: u32,
}

impl Default for WsVideoConfig {
    fn default() -> Self {
        Self {
            port: 4851,
            max_fps: 30,
        }
    }
}

/// WebSocket video streaming server.
pub struct WsVideoServer {
    config: WsVideoConfig,
    frame_rx: watch::Receiver<Option<VideoFrame>>,
}

impl WsVideoServer {
    pub fn new(config: WsVideoConfig, frame_rx: watch::Receiver<Option<VideoFrame>>) -> Self {
        Self { config, frame_rx }
    }

    /// Run the WebSocket video server.
    pub async fn run(self) -> Result<(), TeleopError> {
        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!(addr, "WebSocket video server listening");

        let frame_rx = self.frame_rx;
        let min_frame_interval = Duration::from_secs_f64(1.0 / self.config.max_fps as f64);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!(%addr, "Video client connected");
                    let rx = frame_rx.clone();
                    let interval = min_frame_interval;

                    tokio::spawn(async move {
                        if let Err(e) = handle_video_connection(stream, rx, interval).await {
                            error!(?e, "Video connection error");
                        }
                        info!(%addr, "Video client disconnected");
                    });
                }
                Err(e) => {
                    error!(?e, "Failed to accept video connection");
                }
            }
        }
    }
}

async fn handle_video_connection(
    stream: TcpStream,
    mut frame_rx: watch::Receiver<Option<VideoFrame>>,
    min_interval: Duration,
) -> Result<(), TeleopError> {
    // Disable Nagle's algorithm for lower latency
    let _ = stream.set_nodelay(true);

    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| TeleopError::Network(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Frame sender task
    let sender_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(min_interval);

        loop {
            // Wait for next frame interval
            interval.tick().await;

            // Wait for frame change or timeout
            tokio::select! {
                result = frame_rx.changed() => {
                    if result.is_err() {
                        // Channel closed
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // No new frame, skip
                    continue;
                }
            }

            // Get current frame
            let frame = frame_rx.borrow_and_update().clone();
            if let Some(frame) = frame {
                // Encode frame for WebSocket
                // Format: [0x20] [timestamp:u64 LE] [width:u16 LE] [height:u16 LE] [jpeg_data...]
                let mut buf = Vec::with_capacity(13 + frame.data.len());
                buf.push(0x20); // Video frame marker
                buf.extend_from_slice(&frame.timestamp_ms.to_le_bytes());
                buf.extend_from_slice(&(frame.width as u16).to_le_bytes());
                buf.extend_from_slice(&(frame.height as u16).to_le_bytes());
                buf.extend_from_slice(&frame.data);

                if ws_sender.send(Message::Binary(buf.into())).await.is_err() {
                    debug!("Video client disconnected");
                    break;
                }
            }
        }
    });

    // Receive task (just handle close/ping)
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Close(_)) => {
                break;
            }
            Ok(Message::Ping(_)) => {
                // Pong is automatic
            }
            Err(e) => {
                warn!(?e, "Video WebSocket receive error");
                break;
            }
            _ => {}
        }
    }

    sender_task.abort();
    Ok(())
}
