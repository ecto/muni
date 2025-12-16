//! Hardware simulation for development and testing.
//!
//! Provides simulated VESC responses, tool discovery, and physics.

pub mod vesc;
pub mod tool;
pub mod physics;

use can::Frame;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Simulated CAN bus that generates fake responses.
pub struct SimBus {
    /// Frames waiting to be "received"
    rx_queue: Arc<Mutex<VecDeque<Frame>>>,
    /// Simulated VESCs
    vescs: [vesc::SimVesc; 4],
    /// Simulated tool
    tool: Option<tool::SimTool>,
    /// Physics simulation
    physics: physics::Physics,
}

impl SimBus {
    pub fn new(vesc_ids: [u8; 4]) -> Self {
        Self {
            rx_queue: Arc::new(Mutex::new(VecDeque::new())),
            vescs: [
                vesc::SimVesc::new(vesc_ids[0]),
                vesc::SimVesc::new(vesc_ids[1]),
                vesc::SimVesc::new(vesc_ids[2]),
                vesc::SimVesc::new(vesc_ids[3]),
            ],
            tool: Some(tool::SimTool::new_auger(0)),
            physics: physics::Physics::new(),
        }
    }

    /// Process a "sent" frame and generate appropriate responses.
    pub fn process_tx(&mut self, frame: &Frame) {
        // Check if it's a VESC command
        for vesc in &mut self.vescs {
            if vesc.process_command(frame) {
                // Queue status response
                if let Some(status_frames) = vesc.generate_status() {
                    let mut queue = self.rx_queue.lock().unwrap();
                    for f in status_frames {
                        queue.push_back(f);
                    }
                }
            }
        }

        // Check if it's a tool command
        if let Some(ref mut tool) = self.tool {
            tool.process_command(frame);
        }
    }

    /// Get next "received" frame.
    pub fn recv(&mut self) -> Option<Frame> {
        self.rx_queue.lock().unwrap().pop_front()
    }

    /// Tick the simulation (call at ~100Hz).
    pub fn tick(&mut self, dt: f64) {
        // Update physics
        let wheel_rpms: [f64; 4] = [
            self.vescs[0].rpm(),
            self.vescs[1].rpm(),
            self.vescs[2].rpm(),
            self.vescs[3].rpm(),
        ];
        self.physics.update(wheel_rpms, dt);

        // Generate periodic status messages
        let mut queue = self.rx_queue.lock().unwrap();

        for vesc in &mut self.vescs {
            vesc.tick(dt);
            if vesc.should_send_status() {
                if let Some(frames) = vesc.generate_status() {
                    for f in frames {
                        queue.push_back(f);
                    }
                }
            }
        }

        // Tool discovery/status
        if let Some(ref mut tool) = self.tool {
            tool.tick(dt);
            if let Some(frame) = tool.generate_frame() {
                queue.push_back(frame);
            }
        }
    }

    /// Get current simulated position.
    pub fn position(&self) -> (f64, f64, f64) {
        self.physics.position()
    }

    /// Get current simulated velocity.
    pub fn velocity(&self) -> (f64, f64) {
        self.physics.velocity()
    }
}

/// Wrapper that makes SimBus compatible with can::Bus interface.
pub struct SimCanAdapter {
    sim: Arc<Mutex<SimBus>>,
}

impl SimCanAdapter {
    pub fn new(sim: Arc<Mutex<SimBus>>) -> Self {
        Self { sim }
    }

    pub fn send(&self, frame: &Frame) -> Result<(), can::CanError> {
        debug!(id = frame.id, "SimCAN TX");
        self.sim.lock().unwrap().process_tx(frame);
        Ok(())
    }

    pub fn recv(&self) -> Result<Option<Frame>, can::CanError> {
        Ok(self.sim.lock().unwrap().recv())
    }
}

