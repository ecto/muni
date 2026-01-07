#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Firmware Section
// Initial setup, updating bvrd and attachment firmware

= Firmware

The rover runs two firmware stacks: `bvrd` on the Jetson (the main brain), and embedded firmware on tool attachments (ESP32-based).

Most operators never need to touch firmware. It comes pre-flashed on shipped units, and updates are pushed over-the-air. This section is for those building from scratch or doing development work.

If you're comfortable with Linux command lines and embedded toolchains, this will be familiar. If not, follow the steps exactly, and don't skip the verification steps.

= Initial Jetson Setup

#procedure([Flash and configure Jetson], time: "45 min", difficulty: 3)

#video-link("https://muni.works/docs/jetson-setup", [Full Setup Walkthrough])

#v(1em)

*1. Flash JetPack OS:*

Download JetPack 6.0+ from NVIDIA. Flash using SDK Manager on Ubuntu host:

```bash
# On Ubuntu 20.04/22.04 host machine
sudo apt install nvidia-sdk-manager
sdkmanager  # GUI will launch
```

Select "Jetson Orin NX" and JetPack 6.0. Follow prompts to flash.

#v(1em)

*2. First Boot Configuration:*

```bash
# Set hostname
sudo hostnamectl set-hostname bvr0

# Create muni user (if not done during setup)
sudo adduser muni
sudo usermod -aG sudo,dialout,video muni

# Enable SSH
sudo systemctl enable ssh
```

#v(1em)

*3. Install Dependencies:*

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y can-utils build-essential \
    libclang-dev pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf \
    https://sh.rustup.rs | sh
source ~/.cargo/env
```

#pagebreak()

// =============================================================================

= CAN Bus Setup

#procedure([Configure CAN interface], time: "10 min", difficulty: 2)

#v(1em)

*1. Load CAN Modules:*

```bash
# Add to /etc/modules-load.d/can.conf
echo "can" | sudo tee /etc/modules-load.d/can.conf
echo "can_raw" | sudo tee -a /etc/modules-load.d/can.conf
echo "slcan" | sudo tee -a /etc/modules-load.d/can.conf
```

#v(1em)

*2. Create Startup Service:*

Create `/etc/systemd/system/can.service`:

```ini
[Unit]
Description=CAN Bus Interface
After=network.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/sbin/ip link set can0 type can bitrate 500000
ExecStart=/sbin/ip link set can0 up
ExecStop=/sbin/ip link set can0 down

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable can.service
sudo systemctl start can.service
```

#v(1em)

*3. Verify CAN:*

```bash
# Should show can0 interface
ip link show can0

# Monitor CAN traffic (VESCs should send status)
candump can0
```

#pagebreak()

// =============================================================================

= LiDAR Setup

#procedure([Configure LiDAR network], time: "15 min", difficulty: 2)

#v(1em)

*1. Network Configuration:*

The Mid-360 uses a static IP. Configure the Jetson Ethernet:

```bash
# Add to /etc/netplan/01-lidar.yaml
network:
  version: 2
  ethernets:
    eth0:
      addresses:
        - 192.168.1.50/24
      routes:
        - to: 192.168.1.0/24
          via: 192.168.1.1
```

```bash
sudo netplan apply
```

#v(1em)

*2. LiDAR Default Settings:*

#spec-table(
  [*Parameter*], [*Value*],
  [LiDAR IP], [192.168.1.1xx (xx = last 2 of serial)],
  [Host IP], [192.168.1.50],
  [Data Port], [56000],
  [Command Port], [56001],
)

#v(1em)

*3. Test Connection:*

```bash
# Ping LiDAR (replace with your unit's IP)
ping 192.168.1.100

# Install Livox SDK2 for testing
git clone https://github.com/Livox-SDK/Livox-SDK2
cd Livox-SDK2 && mkdir build && cd build
cmake .. && make -j4
```

#pagebreak()

// =============================================================================

= bvrd Installation

#procedure([Install rover daemon], time: "15 min", difficulty: 2)

#v(1em)

*1. Clone Repository:*

```bash
cd /opt
sudo mkdir muni && sudo chown muni:muni muni
git clone https://github.com/muni-works/bvr.git
cd bvr/firmware
```

#v(1em)

*2. Build:*

```bash
cargo build --release
sudo cp target/release/bvrd /opt/muni/bin/
```

#v(1em)

*3. Install Service:*

```bash
sudo cp config/bvrd.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable bvrd
sudo systemctl start bvrd
```

#v(1em)

*4. Verify:*

```bash
sudo systemctl status bvrd
journalctl -u bvrd -f
```

#pagebreak()

// =============================================================================

= Firmware Overview

The rover uses two main firmware components.

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Jetson box
    rect((-4, 1), (0, 3), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((-2, 2.3), text(size: 10pt, weight: "bold")[Jetson])
    content((-2, 1.7), text(size: 7pt)[bvrd])

    // Attachment box
    rect((2, 1), (6, 3), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
    content((4, 2.3), text(size: 10pt, weight: "bold")[Attachment])
    content((4, 1.7), text(size: 7pt)[ESP32])

    // USB connection
    line((0, 2), (2, 2), stroke: 1.5pt + diagram-black)
    content((1, 2.5), text(size: 6pt)[USB])

    // Labels below
    content((-2, 0.3), text(size: 7pt)[Drive, sensors,\ depot link])
    content((4, 0.3), text(size: 7pt)[LEDs, tools,\ actuators])
  }),
  caption: [Firmware architecture: Jetson runs bvrd, attachments run on ESP32.],
)

#v(2em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *bvrd (Jetson)*
    - Main rover daemon
    - Motor control via CAN
    - Sensor fusion
    - Depot communication
    - Attachment discovery
  ],
  [
    *Attachment (ESP32)*
    - Tool-specific firmware
    - LED control
    - Local sensors
    - SLCAN protocol
    - Status heartbeat
  ]
)

#pagebreak()

// =============================================================================

= Updating bvrd

#procedure([Deploy firmware updates], time: "5 min", difficulty: 1)

#v(1em)

*Prerequisites:*
- SSH access to Jetson
- Rust toolchain with aarch64 target

#v(1em)

*Build and Deploy:*

```bash
# On development machine
cd bvr/firmware
cargo build --release --target aarch64-unknown-linux-gnu

# Copy to Jetson
scp target/aarch64-unknown-linux-gnu/release/bvrd \
    muni@<jetson-ip>:/opt/muni/bin/

# On Jetson - restart service
ssh muni@<jetson-ip>
sudo systemctl restart bvrd
```

#v(1em)

*Verify:*

```bash
# Check service status
sudo systemctl status bvrd

# View logs
journalctl -u bvrd -f
```

#pagebreak()

// =============================================================================

= Updating Attachment Firmware

#procedure([Flash ESP32 attachments], time: "10 min", difficulty: 2)

#v(1em)

*Prerequisites:*
- ESP32 Rust toolchain (`espup install`)
- `espflash` tool installed
- Attachment connected via USB

#v(1em)

*Build:*

```bash
cd mcu/bins/esp32s3
source ~/export-esp.sh
cargo build --release
```

#v(1em)

*Flash:*

```bash
espflash flash \
    --ignore-app-descriptor \
    --partition-table partitions.csv \
    --bootloader bootloader.bin \
    --min-chip-rev 0.0 \
    target/xtensa-esp32s3-none-elf/release/mcu-esp32s3
```

#v(1em)

*Monitor Serial Output:*

```bash
espflash monitor
# Or: screen /dev/cu.usbserial-0001 115200
```

#v(1em)

*If flash fails:*
+ Unplug USB
+ Hold BOOT button
+ Plug in USB (keep holding)
+ Release after 2 seconds
+ Retry flash command

#pagebreak()

// =============================================================================

= Attachment Protocol

Attachments communicate via SLCAN (CAN-over-serial).

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Timeline
    line((-4, 0), (4, 0), stroke: 1pt + diagram-gray)

    // Steps
    let steps = (
      (-3, "O\\r", "Open"),
      (-1, "t2010\\r", "Identify"),
      (1, "t2024...", "Response"),
      (3, "t2008...", "Heartbeat"),
    )

    for (x, cmd, label) in steps {
      circle((x, 0), radius: 0.3, fill: diagram-light, stroke: 1pt + diagram-black)
      content((x, 0.9), text(size: 6pt, style: "italic")[#cmd])
      content((x, -0.7), text(size: 7pt)[#label])
    }

    // Arrows
    line((-2.5, 0), (-1.5, 0), stroke: 1pt + diagram-black, mark: (end: ">"))
    line((-0.5, 0), (0.5, 0), stroke: 1pt + diagram-black, mark: (end: ">"))
    line((1.5, 0), (2.5, 0), stroke: 1pt + diagram-black, mark: (end: ">"))
  }),
  caption: [Discovery flow: open channel, identify, then heartbeat begins.],
)

#v(2em)

*CAN Message IDs (Attachment Slot 0):*

#spec-table(
  [*ID*], [*Direction*], [*Purpose*],
  [0x200], [Attach → Host], [Heartbeat (1Hz)],
  [0x201], [Host → Attach], [Identify request],
  [0x202], [Attach → Host], [Identity response],
  [0x203], [Host → Attach], [Command],
  [0x204], [Attach → Host], [Acknowledgment],
)

#v(1em)

*Text Commands (for debugging):*

```
led 255,0,0    # Set LED red
cycle          # Rainbow cycle mode
state running  # Set state
help           # Show commands
```

#pagebreak()
