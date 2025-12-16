# Tool System

BVR supports hot-swappable tools (attachments) that are auto-discovered via CAN bus.

## Overview

Tools are auxiliary equipment attached to the rover:

- Snow auger
- Salt/sand spreader
- Mower deck
- Plow blade

Each tool has its own microcontroller (RP2040) that communicates with the Jetson over CAN.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              bvrd (Jetson)                                   │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │  tools crate                                                           │ │
│  │                                                                        │ │
│  │  ┌────────────────┐                                                   │ │
│  │  │ Registry       │  Manages discovered tools                         │ │
│  │  │                │                                                   │ │
│  │  │ • active_slot  │  Currently selected tool                         │ │
│  │  │ • tools[]      │  HashMap of Tool implementations                 │ │
│  │  └───────┬────────┘                                                   │ │
│  │          │                                                            │ │
│  │  ┌───────┴────────┐  ┌─────────────┐  ┌─────────────┐               │ │
│  │  │ SnowAuger      │  │ Spreader    │  │ Mower       │  (future)     │ │
│  │  │ impl Tool      │  │ impl Tool   │  │ impl Tool   │               │ │
│  │  └────────────────┘  └─────────────┘  └─────────────┘               │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                         │
│                               CAN bus                                        │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
                    ┌────────────────┴────────────────┐
                    │         Tool MCU (RP2040)       │
                    │                                 │
                    │  • Broadcasts discovery @ 1Hz  │
                    │  • Receives commands            │
                    │  • Sends status @ 20Hz         │
                    │  • Controls actuators          │
                    └─────────────────────────────────┘
```

## Auto-Discovery

Tools announce themselves on the CAN bus every second:

1. Tool MCU powers on
2. Sends discovery frame with type + capabilities
3. Jetson's `Registry` creates appropriate `Tool` instance
4. Tool becomes available for control

```rust
// In bvrd main loop
while let Ok(Some(frame)) = can_bus.recv() {
    state.tool_registry.process_frame(&frame);
}

// Registry handles discovery internally
if let Some((slot, MSG_DISCOVERY)) = can_id::parse(frame.id) {
    self.handle_discovery(slot, &frame.data);
}
```

## Tool Types

### Snow Auger (Type 0x01)

Clears snow from paths.

**Controls:**

- RT/LT: Raise/lower auger head
- A button: Toggle auger spin

**Capabilities:**

- Axis control (lift)
- Motor control (auger)
- Position feedback
- Current feedback

**Status:**

- Position: 0% (down) to 100% (up)
- Auger RPM
- Motor current

### Spreader (Type 0x02)

Spreads salt/sand.

**Controls:**

- RT: Increase spread rate
- LT: Decrease spread rate
- A button: Toggle spreader on/off

**Capabilities:**

- Motor control (spinner)
- Motor control (auger/feeder)

### Mower (Type 0x03) — Future

**Controls:**

- RT/LT: Raise/lower deck
- A button: Toggle blades

### Plow (Type 0x04) — Future

**Controls:**

- RT/LT: Raise/lower blade
- A button: Angle left
- B button: Angle right

## Adding a New Tool

### 1. Define the tool type

In `tools/src/lib.rs`:

```rust
pub enum ToolType {
    // ...existing types...
    MyNewTool = 5,
}
```

### 2. Create implementation

Create `tools/src/my_new_tool.rs`:

```rust
use crate::{Capabilities, Tool, ToolInfo, ToolOutput, ToolStatus, ToolType};
use types::ToolCommand;

pub struct MyNewTool {
    info: ToolInfo,
    // tool-specific state
}

impl MyNewTool {
    pub fn new(slot: u8, serial: u32) -> Self {
        Self {
            info: ToolInfo {
                slot,
                tool_type: ToolType::MyNewTool,
                capabilities: Capabilities::MOTOR_CONTROL,
                serial,
                name: "My New Tool",
            },
        }
    }
}

impl Tool for MyNewTool {
    fn info(&self) -> &ToolInfo { &self.info }

    fn update(&mut self, input: &ToolCommand) -> ToolOutput {
        // Map controller input to tool commands
        ToolOutput::SetMotor(input.motor)
    }

    fn handle_status(&mut self, data: &[u8]) {
        // Parse status from MCU
    }

    fn status(&self) -> ToolStatus {
        ToolStatus {
            name: self.info.name,
            position: None,
            active: false,
            current: None,
            fault: false,
        }
    }
}
```

### 3. Register in discovery

In `tools/src/discovery.rs`:

```rust
let tool: Box<dyn Tool> = match tool_type {
    ToolType::SnowAuger => Box::new(SnowAuger::new(slot, serial)),
    ToolType::MyNewTool => Box::new(MyNewTool::new(slot, serial)),
    // ...
};
```

### 4. Implement MCU firmware

See [Tool MCU Development](#tool-mcu-development).

## Tool MCU Development

### Hardware

| Component    | Recommendation       |
| ------------ | -------------------- |
| MCU          | RP2040 (Pico W 2)    |
| CAN          | MCP2515 module (SPI) |
| Motor driver | Depends on actuators |
| Power        | 12V from rover rail  |

### Wiring (RP2040 + MCP2515)

```
Pico W 2          MCP2515
────────          ───────
GPIO 16 (MISO) ← SO
GPIO 17 (CS)   → CS
GPIO 18 (SCK)  → SCK
GPIO 19 (MOSI) → SI
GPIO 20        ← INT
3.3V           → VCC
GND            → GND

MCP2515           CAN Bus
───────           ───────
CANH             → CANH (to rover)
CANL             → CANL
```

### Firmware Structure (Embassy/Rust)

```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Initialize MCP2515
    let can = Mcp2515::new(spi, cs_pin);

    // Spawn tasks
    spawner.spawn(discovery_task(can.clone())).unwrap();
    spawner.spawn(status_task(can.clone())).unwrap();
    spawner.spawn(command_task(can.clone())).unwrap();
}

#[embassy_executor::task]
async fn discovery_task(can: Mcp2515) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;
        can.send(discovery_frame()).await;
    }
}
```

### CAN Protocol Implementation

```rust
const TOOL_SLOT: u8 = 0;
const TOOL_TYPE: u8 = 1; // Snow auger

fn discovery_frame() -> CanFrame {
    let id = 0x0A00 | ((TOOL_SLOT as u32) << 4) | 0x0;
    let data = [
        TOOL_TYPE,           // Type
        1,                   // Protocol version
        0x0F, 0x00,         // Capabilities (axis + motor + position + current)
        0x12, 0x34, 0x56, 0x78, // Serial
    ];
    CanFrame::new_extended(id, &data)
}

fn parse_command(frame: &CanFrame) -> Option<(i16, i16)> {
    if frame.data.len() < 5 { return None; }
    let axis = i16::from_le_bytes([frame.data[1], frame.data[2]]);
    let motor = i16::from_le_bytes([frame.data[3], frame.data[4]]);
    Some((axis, motor))
}
```

## Testing

### Without Hardware

The `can` crate provides a mock on non-Linux platforms:

```bash
# On macOS/Windows
cargo run --bin bvrd
# Will use mock CAN, logs commands
```

### CAN Bus Simulation

On Linux with `vcan`:

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0

# Run bvrd
cargo run --bin bvrd -- --can-interface vcan0

# In another terminal, simulate tool discovery
cansend vcan0 0A00#0101030012345678
```

### candump for Debugging

```bash
candump can0  # See all traffic
candump -c can0,0A00:0FF0  # Filter to tool messages only
```
