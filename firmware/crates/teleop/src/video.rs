//! Video streaming over UDP.
//!
//! Uses a simple chunked protocol to send JPEG frames over UDP:
//! - Frame header: [0x20, seq_hi, seq_lo, chunk_idx, total_chunks, ...]
//! - Each chunk is at most MTU-safe (~1400 bytes payload)

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tracing::{debug, error, trace, warn};

/// Maximum payload per UDP packet (leaving room for headers).
const MAX_CHUNK_SIZE: usize = 1400;

/// Video frame header size.
const HEADER_SIZE: usize = 16;

/// Video streaming configuration.
#[derive(Debug, Clone)]
pub struct VideoConfig {
    /// UDP port for video streaming
    pub port: u16,
    /// Maximum time to wait for all chunks of a frame
    pub frame_timeout: Duration,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            port: 4842,
            frame_timeout: Duration::from_millis(200),
        }
    }
}

/// A video frame ready for display.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// JPEG-encoded image data
    pub data: Vec<u8>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Sequence number
    pub sequence: u32,
    /// Timestamp (milliseconds since epoch)
    pub timestamp_ms: u64,
}

/// Encodes a frame into UDP packets.
pub fn encode_frame(frame: &VideoFrame) -> Vec<Vec<u8>> {
    let total_chunks = (frame.data.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE;
    let total_chunks = total_chunks.max(1) as u8;

    let mut packets = Vec::with_capacity(total_chunks as usize);

    for (chunk_idx, chunk) in frame.data.chunks(MAX_CHUNK_SIZE).enumerate() {
        let mut packet = Vec::with_capacity(HEADER_SIZE + chunk.len());

        // Header
        packet.push(0x20); // Video frame marker
        packet.push((frame.sequence >> 8) as u8);
        packet.push(frame.sequence as u8);
        packet.push(chunk_idx as u8);
        packet.push(total_chunks);
        packet.push((frame.width >> 8) as u8);
        packet.push(frame.width as u8);
        packet.push((frame.height >> 8) as u8);
        packet.push(frame.height as u8);
        // Timestamp (6 bytes, enough for ~8900 years)
        packet.extend_from_slice(&frame.timestamp_ms.to_be_bytes()[2..]);
        // Reserved
        packet.push(0);

        // Payload
        packet.extend_from_slice(chunk);

        packets.push(packet);
    }

    packets
}

/// Video streaming server (runs on bvrd).
pub struct VideoServer {
    config: VideoConfig,
    frame_rx: watch::Receiver<Option<VideoFrame>>,
}

impl VideoServer {
    pub fn new(config: VideoConfig, frame_rx: watch::Receiver<Option<VideoFrame>>) -> Self {
        Self { config, frame_rx }
    }

    /// Run the video server.
    pub async fn run(mut self) -> std::io::Result<()> {
        let addr = format!("0.0.0.0:{}", self.config.port);
        let socket = Arc::new(UdpSocket::bind(&addr).await?);
        tracing::info!(addr, "Video server listening");

        let mut client_addr: Option<SocketAddr> = None;
        let mut buf = [0u8; 64];

        loop {
            tokio::select! {
                // Listen for client registration (any packet = subscribe)
                result = socket.recv_from(&mut buf) => {
                    if let Ok((_, addr)) = result {
                        if client_addr != Some(addr) {
                            tracing::info!(%addr, "Video client connected");
                            client_addr = Some(addr);
                        }
                    }
                }

                // Wait for new frame
                _ = self.frame_rx.changed() => {
                    if let Some(addr) = client_addr {
                        // Clone the frame data immediately to avoid holding borrow across await
                        let frame = self.frame_rx.borrow_and_update().clone();
                        if let Some(frame) = frame {
                            let packets = encode_frame(&frame);
                            let num_packets = packets.len();
                            for packet in packets {
                                if let Err(e) = socket.send_to(&packet, addr).await {
                                    error!(?e, "Failed to send video packet");
                                }
                            }
                            trace!(seq = frame.sequence, chunks = num_packets, "Sent frame");
                        }
                    }
                }
            }
        }
    }
}

/// Reassembles chunked frames on the client side.
pub struct FrameReassembler {
    /// Partial frames being assembled (keyed by sequence number)
    partial: HashMap<u32, PartialFrame>,
    /// Timeout for incomplete frames
    timeout: Duration,
    /// Last cleanup time
    last_cleanup: Instant,
}

struct PartialFrame {
    chunks: Vec<Option<Vec<u8>>>,
    width: u32,
    height: u32,
    timestamp_ms: u64,
    received_at: Instant,
    received_count: u8,
    total_chunks: u8,
}

impl FrameReassembler {
    pub fn new(timeout: Duration) -> Self {
        Self {
            partial: HashMap::new(),
            timeout,
            last_cleanup: Instant::now(),
        }
    }

    /// Process a received packet. Returns a complete frame if one is ready.
    pub fn process(&mut self, data: &[u8]) -> Option<VideoFrame> {
        if data.len() < HEADER_SIZE || data[0] != 0x20 {
            return None;
        }

        let sequence = ((data[1] as u32) << 8) | (data[2] as u32);
        let chunk_idx = data[3] as usize;
        let total_chunks = data[4];
        let width = ((data[5] as u32) << 8) | (data[6] as u32);
        let height = ((data[7] as u32) << 8) | (data[8] as u32);
        let timestamp_ms = u64::from_be_bytes([0, 0, data[9], data[10], data[11], data[12], data[13], data[14]]);

        let payload = data[HEADER_SIZE..].to_vec();

        // Get or create partial frame
        let partial = self.partial.entry(sequence).or_insert_with(|| PartialFrame {
            chunks: vec![None; total_chunks as usize],
            width,
            height,
            timestamp_ms,
            received_at: Instant::now(),
            received_count: 0,
            total_chunks,
        });

        // Store chunk if not already received
        if chunk_idx < partial.chunks.len() && partial.chunks[chunk_idx].is_none() {
            partial.chunks[chunk_idx] = Some(payload);
            partial.received_count += 1;
        }

        // Check if frame is complete
        if partial.received_count == partial.total_chunks {
            let partial = self.partial.remove(&sequence)?;
            let data: Vec<u8> = partial
                .chunks
                .into_iter()
                .filter_map(|c| c)
                .flatten()
                .collect();

            debug!(seq = sequence, size = data.len(), "Frame complete");

            return Some(VideoFrame {
                data,
                width: partial.width,
                height: partial.height,
                sequence,
                timestamp_ms: partial.timestamp_ms,
            });
        }

        // Periodic cleanup of stale partial frames
        if self.last_cleanup.elapsed() > self.timeout {
            self.cleanup();
            self.last_cleanup = Instant::now();
        }

        None
    }

    fn cleanup(&mut self) {
        let timeout = self.timeout;
        self.partial.retain(|seq, partial| {
            let keep = partial.received_at.elapsed() < timeout;
            if !keep {
                warn!(seq, "Dropping incomplete frame");
            }
            keep
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_small_frame() {
        let frame = VideoFrame {
            data: vec![0xFF, 0xD8, 0xFF, 0xE0], // Minimal JPEG header
            width: 640,
            height: 480,
            sequence: 42,
            timestamp_ms: 1234567890,
        };

        let packets = encode_frame(&frame);
        assert_eq!(packets.len(), 1);

        let mut reassembler = FrameReassembler::new(Duration::from_secs(1));
        let decoded = reassembler.process(&packets[0]).unwrap();

        assert_eq!(decoded.sequence, 42);
        assert_eq!(decoded.width, 640);
        assert_eq!(decoded.height, 480);
        assert_eq!(decoded.data, frame.data);
    }

    #[test]
    fn test_encode_decode_large_frame() {
        // Create a frame larger than one packet
        let frame = VideoFrame {
            data: vec![0xAB; 5000],
            width: 1280,
            height: 720,
            sequence: 100,
            timestamp_ms: 9999999999,
        };

        let packets = encode_frame(&frame);
        assert!(packets.len() > 1);

        let mut reassembler = FrameReassembler::new(Duration::from_secs(1));

        // Process all but last packet - should return None
        for packet in &packets[..packets.len()-1] {
            assert!(reassembler.process(packet).is_none());
        }

        // Last packet completes the frame
        let decoded = reassembler.process(packets.last().unwrap()).unwrap();
        assert_eq!(decoded.data.len(), 5000);
    }
}
