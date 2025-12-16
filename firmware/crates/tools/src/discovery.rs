//! Tool auto-discovery via CAN bus.

use crate::protocol::{can_id, DiscoveryPayload};
use crate::{SnowAuger, Tool, ToolType};
use can::Frame;
use std::collections::HashMap;
use tracing::{info, warn};

/// Tool registry with auto-discovery.
pub struct Registry {
    tools: HashMap<u8, Box<dyn Tool>>,
    active_slot: Option<u8>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            active_slot: None,
        }
    }

    /// Process a received CAN frame for discovery/status.
    ///
    /// Returns true if the frame was handled.
    pub fn process_frame(&mut self, frame: &Frame) -> bool {
        let Some((slot, msg_type)) = can_id::parse(frame.id) else {
            return false;
        };

        match msg_type {
            can_id::MSG_DISCOVERY => {
                self.handle_discovery(slot, &frame.data);
                true
            }
            can_id::MSG_STATUS => {
                if let Some(tool) = self.tools.get_mut(&slot) {
                    tool.handle_status(&frame.data);
                }
                true
            }
            _ => false,
        }
    }

    /// Handle a discovery frame.
    fn handle_discovery(&mut self, slot: u8, data: &[u8]) {
        // Don't re-register existing tools
        if self.tools.contains_key(&slot) {
            return;
        }

        let Some(payload) = DiscoveryPayload::parse(data) else {
            warn!(slot, "Invalid discovery payload");
            return;
        };

        let tool_type = ToolType::from(payload.tool_type);
        let serial = payload.serial;

        let tool: Box<dyn Tool> = match tool_type {
            ToolType::SnowAuger => Box::new(SnowAuger::new(slot, serial)),
            // Add more tool types here as they're implemented
            _ => {
                warn!(slot, ?tool_type, "Unknown tool type, ignoring");
                return;
            }
        };

        info!(
            slot,
            name = tool.info().name,
            serial,
            "Tool discovered"
        );

        // Set as active if no tool is active
        if self.active_slot.is_none() {
            self.active_slot = Some(slot);
        }

        self.tools.insert(slot, tool);
    }

    /// Get the active tool.
    pub fn active(&self) -> Option<&dyn Tool> {
        self.active_slot
            .and_then(|slot| self.tools.get(&slot))
            .map(|t| t.as_ref())
    }

    /// Get the active tool mutably.
    pub fn active_mut(&mut self) -> Option<&mut Box<dyn Tool>> {
        let slot = self.active_slot?;
        self.tools.get_mut(&slot)
    }

    /// Get the active slot.
    pub fn active_slot(&self) -> Option<u8> {
        self.active_slot
    }

    /// Cycle to the next tool (for LB/RB buttons).
    pub fn cycle(&mut self, direction: i8) {
        if self.tools.is_empty() {
            return;
        }

        let mut slots: Vec<u8> = self.tools.keys().copied().collect();
        slots.sort();

        let current_idx = self
            .active_slot
            .and_then(|s| slots.iter().position(|&x| x == s))
            .unwrap_or(0);

        let new_idx = if direction > 0 {
            (current_idx + 1) % slots.len()
        } else {
            (current_idx + slots.len() - 1) % slots.len()
        };

        self.active_slot = Some(slots[new_idx]);

        if let Some(tool) = self.active() {
            info!(name = tool.info().name, "Switched to tool");
        }
    }

    /// Get number of registered tools.
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Iterate over all tools.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Tool> {
        self.tools.values().map(|t| t.as_ref())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
