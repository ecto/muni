//! CAN interface abstraction for real, simulated, remote, or serial hardware.

use anyhow::Result;
use can::Bus;
use sim::SimBus;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use types::Pose;

use crate::vesc_serial::VescSerial;

/// Remote CAN connection to sim-bridge over TCP.
struct RemoteCan {
    /// Outgoing frame writer (shared with background reader)
    writer: Arc<tokio::sync::Mutex<tokio::io::WriteHalf<tokio::net::TcpStream>>>,
    /// Incoming frames from sim-bridge
    rx_queue: Arc<Mutex<std::collections::VecDeque<can::Frame>>>,
    /// Latest pose from sim-bridge
    pose: Arc<Mutex<Pose>>,
}

impl RemoteCan {
    async fn connect(url: &str) -> Result<Self> {
        // Parse tcp://host:port
        let addr = url
            .strip_prefix("tcp://")
            .ok_or_else(|| anyhow::anyhow!("remote-can URL must start with tcp://"))?;

        info!(addr, "Connecting to remote CAN");
        let stream = tokio::net::TcpStream::connect(addr).await?;
        stream.set_nodelay(true)?;
        info!(addr, "Connected to remote CAN");

        let (reader, writer) = tokio::io::split(stream);
        let writer = Arc::new(tokio::sync::Mutex::new(writer));
        let rx_queue = Arc::new(Mutex::new(std::collections::VecDeque::<can::Frame>::new()));
        let pose = Arc::new(Mutex::new(Pose::default()));

        // Spawn background reader
        let rx_q = rx_queue.clone();
        let pose_r = pose.clone();
        tokio::spawn(async move {
            Self::reader_task(reader, rx_q, pose_r).await;
        });

        Ok(Self {
            writer,
            rx_queue,
            pose,
        })
    }

    async fn reader_task(
        mut reader: tokio::io::ReadHalf<tokio::net::TcpStream>,
        rx_queue: Arc<Mutex<std::collections::VecDeque<can::Frame>>>,
        pose: Arc<Mutex<Pose>>,
    ) {
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; 4096];
        let mut pending = Vec::new();

        loop {
            match reader.read(&mut buf).await {
                Ok(0) => {
                    warn!("Remote CAN connection closed");
                    break;
                }
                Ok(n) => {
                    pending.extend_from_slice(&buf[..n]);
                    // Parse messages from pending buffer
                    loop {
                        if pending.is_empty() {
                            break;
                        }
                        match pending[0] {
                            // CAN frame
                            0x01 => {
                                // Need at least: type(1) + flags(1) + id(4) + len(1) = 7
                                if pending.len() < 2 {
                                    break;
                                }
                                if let Some((frame, consumed)) =
                                    can::Frame::from_bytes(&pending[1..])
                                {
                                    rx_queue.lock().expect("rx_queue mutex poisoned").push_back(frame);
                                    pending.drain(..1 + consumed);
                                } else {
                                    break; // Need more data
                                }
                            }
                            // Pose update
                            0x02 => {
                                if pending.len() < 25 {
                                    break;
                                }
                                let x = f64::from_le_bytes(
                                    pending[1..9].try_into().unwrap(),
                                );
                                let y = f64::from_le_bytes(
                                    pending[9..17].try_into().unwrap(),
                                );
                                let theta = f64::from_le_bytes(
                                    pending[17..25].try_into().unwrap(),
                                );
                                *pose.lock().expect("pose mutex poisoned") = Pose { x, y, theta };
                                pending.drain(..25);
                            }
                            _ => {
                                warn!(byte = pending[0], "Unknown remote CAN message type");
                                pending.remove(0);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(?e, "Remote CAN read error");
                    break;
                }
            }
        }
    }

    fn send_frame(&self, frame: &can::Frame) {
        let writer = self.writer.clone();
        let bytes = frame.to_bytes();
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let mut w = writer.lock().await;
            let mut msg = vec![0x01u8];
            msg.extend_from_slice(&bytes);
            if let Err(e) = w.write_all(&msg).await {
                warn!(?e, "Remote CAN write error");
            }
        });
    }
}

/// CAN interface abstraction for real or simulated hardware.
#[allow(private_interfaces)]
pub(crate) enum CanInterface {
    Real(Bus),
    Sim(Arc<Mutex<SimBus>>),
    Remote(RemoteCan),
    Serial(VescSerial),
}

impl CanInterface {
    /// Connect to a remote CAN bus over TCP.
    pub(crate) async fn connect_remote(url: &str) -> Result<Self> {
        let remote = RemoteCan::connect(url).await?;
        Ok(Self::Remote(remote))
    }

    /// Open a VESC serial gateway (USB UART to CAN bridge).
    pub(crate) async fn open_serial(port: &str, gateway_id: u8) -> Result<Self> {
        let serial = VescSerial::open(port, gateway_id).await?;
        Ok(Self::Serial(serial))
    }

    pub(crate) fn send(&self, frame: &can::Frame) -> Result<(), can::CanError> {
        match self {
            Self::Real(bus) => bus.send(frame),
            Self::Sim(sim) => {
                sim.lock().expect("sim mutex poisoned").process_tx(frame);
                Ok(())
            }
            Self::Remote(remote) => {
                remote.send_frame(frame);
                Ok(())
            }
            Self::Serial(serial) => {
                serial.send_frame(frame);
                Ok(())
            }
        }
    }

    pub(crate) fn recv(&self) -> Result<Option<can::Frame>, can::CanError> {
        match self {
            Self::Real(bus) => bus.recv(),
            Self::Sim(sim) => Ok(sim.lock().expect("sim mutex poisoned").recv()),
            Self::Remote(remote) => Ok(remote.rx_queue.lock().expect("remote rx_queue mutex poisoned").pop_front()),
            Self::Serial(serial) => Ok(serial.recv()),
        }
    }

    pub(crate) fn tick(&self, dt: f64) {
        if let Self::Sim(sim) = self {
            sim.lock().expect("sim mutex poisoned").tick(dt);
        }
        // Remote/Serial: no-op (physics runs elsewhere or on real hardware)
    }

    /// Get current pose from simulation (returns default for real hardware).
    pub(crate) fn pose(&self) -> Pose {
        match self {
            Self::Real(_) | Self::Serial(_) => Pose::default(),
            Self::Sim(sim) => {
                let (x, y, theta) = sim.lock().expect("sim mutex poisoned").position();
                Pose { x, y, theta }
            }
            Self::Remote(remote) => *remote.pose.lock().expect("remote pose mutex poisoned"),
        }
    }
}
