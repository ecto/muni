#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Firmware Section
// Updating bvrd and attachment firmware

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

Update the main rover daemon on the Jetson.

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

Flash new firmware to ESP32-based attachments.

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
