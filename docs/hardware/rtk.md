# RTK GPS

Centimeter-accurate positioning for georeferenced mapping.

## Quick Start

```bash
# Install the muni CLI
cd bvr/firmware && cargo install --path bins/cli

# Configure base station (with known coordinates)
muni gps configure-base --port /dev/tty.usbmodem1101 \
    --fixed-position 41.481956,-81.8053,213.5

# Configure rover
muni gps configure-rover --port /dev/ttyACM0

# Monitor GPS status (works for both base and rover)
muni gps monitor --port /dev/tty.usbmodem1101
```

Press `q`, `Esc`, or `Ctrl+C` to exit the monitor.

## Overview

RTK (Real-Time Kinematic) GPS uses corrections from a base station to achieve
±1-2cm positioning accuracy, compared to ±2-5m with standard GPS.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              RTK Architecture                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Depot (Base Station)                         │   │
│  │                                                                       │   │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐  │   │
│  │  │ GNSS        │    │ ZED-F9P     │    │ NTRIP Caster            │  │   │
│  │  │ Antenna     │───►│ (Base Mode) │───►│ (broadcasts corrections)│  │   │
│  │  │ (roof)      │    │ USB to Pi   │    │ Port 2101               │  │   │
│  │  └─────────────┘    └─────────────┘    └─────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                        │
│                                     │ RTCM3 corrections                      │
│                                     │ (via 5 GHz mesh network)               │
│                                     ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Rover (Receiver)                             │   │
│  │                                                                       │   │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐  │   │
│  │  │ GNSS        │    │ ZED-F9P     │    │ gps crate               │  │   │
│  │  │ Antenna     │───►│ (Rover Mode)│───►│ (NMEA parsing)          │  │   │
│  │  │ (on rover)  │    │             │    │                          │  │   │
│  │  └─────────────┘    └─────────────┘    └─────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

RTK corrections are delivered over the Ubiquiti mesh network (see [networking.md](networking.md)).
The low latency (5-20ms vs 50-150ms with LTE) improves fix acquisition time.

## When You Need RTK

| Use Case                 | Standard GPS    | RTK GPS       |
| ------------------------ | --------------- | ------------- |
| Fleet tracking           | ✅ Sufficient   | Overkill      |
| Rough geolocation        | ✅ Sufficient   | Overkill      |
| Georeferenced mapping    | ⚠️ ~3m error    | ✅ ~2cm error |
| Multi-session map fusion | ❌ Drift issues | ✅ Consistent |
| Autonomous navigation    | ⚠️ Lane-level   | ✅ Centimeter |

**For bvr0**: Standard GPS is fine for prototyping.
**For bvr1**: RTK enables street-view quality georeferenced mapping.

## Hardware

### Rover Receiver

| Component  | Model                                                                                                     | Price | Source   |
| ---------- | --------------------------------------------------------------------------------------------------------- | ----- | -------- |
| GPS Module | [SparkFun GPS-RTK2 (ZED-F9P)](https://www.amazon.com/SparkFun-GPS-RTK2-Board-ZED-F9P-Qwiic/dp/B07NBPNWNZ) | ~$220 | Amazon   |
| Antenna    | SparkFun GNSS Multi-Band L1/L2 (active)                                                                   | ~$75  | SparkFun |
| SMA Cable  | RG174, 1m                                                                                                 | ~$10  | Amazon   |
| U.FL-SMA   | U.FL to SMA pigtail (if using breakout)                                                                   | ~$8   | Amazon   |

**Important:** The ZED-F9P requires a multi-band (L1+L2) active GNSS antenna. A standard WiFi
antenna will not work: you'll see "NO FIX" with 0 satellites. The SparkFun board has a U.FL
connector: use a U.FL to SMA pigtail to connect the antenna.

### Base Station (Depot)

| Component    | Model                           | Price | Notes                      |
| ------------ | ------------------------------- | ----- | -------------------------- |
| GPS Module   | SparkFun GPS-RTK2 (ZED-F9P)     | ~$220 | Same as rover              |
| Antenna      | Tallysman TW4721 (survey-grade) | ~$100 | Better multipath rejection |
| SMA Cable    | LMR-400, 25ft                   | ~$40  | Low-loss for roof run      |
| Raspberry Pi | Pi 4 (2GB+)                     | ~$50  | Runs NTRIP caster          |

**Total base station: ~$410**

Alternatively, integrate into the depot server directly via USB.

## Rover Integration

### Hardware Connection

**Option 1: USB (Recommended for development)**

```
GNSS Antenna
     │
     │ U.FL → SMA pigtail → SMA cable
     ▼
┌─────────────┐
│ SparkFun    │
│ ZED-F9P     │
│             │
│ USB-C ──────┼──► Computer/Jetson USB
│             │    /dev/ttyACM0 (Linux)
│             │    /dev/tty.usbmodem* (macOS)
└─────────────┘
```

USB provides power and data. Default baud: 38400.

**Option 2: UART (Recommended for production)**

```
GNSS Antenna
     │
     │ SMA cable
     ▼
┌─────────────┐
│ SparkFun    │
│ ZED-F9P     │
│             │
│ TX ─────────┼──► Jetson UART (/dev/ttyTHS1)
│ RX ◄────────┼─── Jetson UART (for RTCM input)
│ VCC ◄───────┼─── 3.3V
│ GND ◄───────┼─── GND
└─────────────┘
```

For UART, configure the baud rate to 115200 for high-rate output.

### Software Integration

The ZED-F9P outputs standard NMEA, compatible with existing `gps` crate:

```rust
// Minimal change to gps crate
let gps = GpsReceiver::new("/dev/ttyTHS1", 115200)?;

// Position now has RTK accuracy when corrections are received
let fix = gps.get_fix()?;
println!("Position: {:.8}, {:.8} (accuracy: {:.2}m)",
    fix.latitude, fix.longitude, fix.horizontal_accuracy);
```

### NTRIP Client

To receive corrections from the depot base station over the mesh network:

```rust
// crates/gps/src/ntrip.rs

pub struct NtripClient {
    server: String,  // depot IP on mesh network
    port: u16,       // 2101
    mountpoint: String,
}

impl NtripClient {
    /// Connect to NTRIP caster and stream corrections to GPS receiver
    pub async fn run(&self, gps_tx: &mut dyn Write) -> Result<(), NtripError> {
        let stream = TcpStream::connect((self.server.as_str(), self.port)).await?;
        // ... NTRIP handshake ...

        loop {
            let rtcm_data = stream.read(...).await?;
            gps_tx.write_all(&rtcm_data)?;  // Forward to ZED-F9P UART
        }
    }
}
```

### Bandwidth Requirements

RTK corrections are lightweight and easily fit alongside teleop traffic:

| Stream            | Bandwidth | Protocol |
| ----------------- | --------- | -------- |
| RTCM3 corrections | 1-2 kbps  | TCP      |
| NMEA output       | <1 kbps   | Serial   |

For comparison, a single rover uses ~5 Mbps total for video and teleop.

## Base Station Setup

### Hardware Installation

1. **Mount antenna** on depot roof with clear sky view (no obstructions above 10°)
2. **Run SMA cable** through building to server rack
3. **Connect ZED-F9P** to depot server via USB
4. **Survey position** to establish precise base coordinates

### Survey Procedure

The base station must know its exact position. Options:

| Method              | Accuracy | Time                         |
| ------------------- | -------- | ---------------------------- |
| Self-survey (1hr)   | ±1m      | 1 hour                       |
| Self-survey (24hr)  | ±10cm    | 24 hours                     |
| PPP service         | ±2cm     | Submit 24hr log, wait 1 week |
| Professional survey | ±1cm     | Hire surveyor                |

For most applications, 24-hour self-survey is sufficient:

```bash
# Configure ZED-F9P for survey-in mode (24hr, 10cm target)
muni gps configure-base --port /dev/ttyACM0 \
    --survey-duration 86400 \
    --survey-accuracy 0.1

# Monitor survey progress
muni gps monitor --port /dev/ttyACM0
```

If you know the exact coordinates (from a previous survey or professional surveyor):

```bash
# Configure with fixed position (lat,lon,alt in meters)
muni gps configure-base --port /dev/ttyACM0 \
    --fixed-position 41.481956,-81.8053,213.5
```

### NTRIP Caster

Run an NTRIP caster to broadcast corrections:

#### Option 1: Docker Container

```yaml
# depot/docker-compose.yml (addition)
services:
  ntrip:
    image: ghcr.io/rtcm/rtkbase:latest
    ports:
      - "2101:2101"
    devices:
      - /dev/ttyUSB0:/dev/ttyUSB0 # ZED-F9P
    volumes:
      - ./ntrip/config:/config
```

#### Option 2: SNIP (Windows/Linux)

[SNIP](http://www.use-snip.com/) is a popular NTRIP caster with a free tier.

#### Option 3: str2str (RTKLIB)

```bash
# Stream ZED-F9P output as NTRIP caster
str2str -in serial://ttyUSB0:115200 \
        -out ntrips://:password@:2101/ROVER
```

### Depot Server Integration

Add to depot services in `docker-compose.yml`:

```yaml
ntrip-caster:
  image: ghcr.io/rtcm/rtkbase:latest
  container_name: ntrip
  restart: unless-stopped
  ports:
    - "2101:2101"
  devices:
    - /dev/ttyUSB0:/dev/ttyUSB0
  volumes:
    - ntrip-config:/config

volumes:
  ntrip-config:
```

## Alternative: NTRIP Network

Instead of running your own base station, use a commercial NTRIP network:

| Provider                                    | Coverage | Price              |
| ------------------------------------------- | -------- | ------------------ |
| [PointOne Nav](https://pointonenav.com/)    | USA/EU   | ~$50/mo            |
| [Skylark](https://www.swiftnav.com/skylark) | Global   | ~$50/mo            |
| State DOT CORS                              | Varies   | Free (some states) |

For a fleet operating in multiple regions, a network subscription may be
more practical than deploying base stations.

## Mounting

### Rover Antenna

- Mount on top of sensor pole (highest point)
- Clear sky view in all directions
- Ground plane improves multipath rejection

```
        ┌─────────┐
        │  GNSS   │  ← Top of stack
        │ Antenna │
        └────┬────┘
        ┌────┴────┐
        │Insta360 │
        │   X4    │
        └────┬────┘
        ┌────┴────┐
        │ Mid-360 │
        └────┬────┘
             │
         Rover
```

### Base Station Antenna

- Roof mount with unobstructed sky
- Away from metal structures, HVAC units
- Secure against wind
- Protect cable entry from weather

## Accuracy Expectations

| Condition         | Horizontal | Vertical |
| ----------------- | ---------- | -------- |
| RTK fixed (ideal) | ±1cm       | ±2cm     |
| RTK float         | ±20cm      | ±40cm    |
| DGPS (no RTK)     | ±50cm      | ±1m      |
| Standard GPS      | ±2-5m      | ±5-10m   |

RTK requires:

- Clear sky view (>5 satellites)
- Corrections from base station
- Base station within ~20km (for single-base RTK)

## Troubleshooting

### No RTK Fix

```bash
# Monitor satellite count, fix quality, and HDOP
muni gps monitor --port /dev/ttyACM0

# Verify NTRIP caster is running
curl -v http://depot:2101/
```

The monitor TUI shows:

- Fix quality (NO FIX, GPS, DGPS, RTK FLOAT, RTK FIXED)
- Satellite counts per constellation (GPS, GLONASS, Galileo, BeiDou)
- Signal strength bars for each satellite

### Frequent Fix Loss

- Check antenna placement (obstructions above 10° elevation?)
- Verify correction stream continuity
- Ensure multi-band active antenna (not a WiFi antenna!)
- Consider longer cable or better antenna

### Base Station Issues

```bash
# Monitor base station: shows survey-in progress and RTCM output
muni gps monitor --port /dev/ttyACM0
```

The base station monitor shows:

- Survey-in duration and accuracy
- RTCM message counts and types being broadcast
- Fixed position coordinates (after survey complete)

### Configuration Not Persisting

If the module reverts to NMEA output after power cycling:

```bash
# Reconfigure (settings are saved to flash)
muni gps configure-base --port /dev/ttyACM0 --fixed-position LAT,LON,ALT

# Power cycle the module, then verify
muni gps monitor --port /dev/ttyACM0
# Should show "BASE" mode, not "ROVER"
```

## CLI Reference

### muni gps monitor

Real-time TUI monitor for GNSS receivers. Auto-detects rover (NMEA) vs base (RTCM/UBX) mode.

```bash
muni gps monitor --port /dev/ttyACM0 [--baud 38400]
```

**Rover mode display:**

- Position (lat/lon/alt)
- Fix quality and satellite count
- Per-constellation satellite signal bars
- Raw NMEA log

**Base station mode display:**

- Survey-in progress (duration, accuracy, observations)
- RTCM message statistics (type, count, bytes)
- Fixed position coordinates

**Controls:** `q`, `Esc`, or `Ctrl+C` to quit.

### muni gps configure-base

Configure ZED-F9P as an RTK base station.

```bash
# Survey-in mode (determines position over time)
muni gps configure-base --port /dev/ttyACM0 \
    --survey-duration 3600 \
    --survey-accuracy 2.0

# Fixed position mode (use known coordinates)
muni gps configure-base --port /dev/ttyACM0 \
    --fixed-position 41.481956,-81.8053,213.5
```

This command:

- Disables NMEA output
- Enables RTCM3 messages (1005, 1074, 1084, 1094, 1124, 1230)
- Enables UBX-NAV-SVIN for survey status
- Configures time mode (survey-in or fixed)
- Saves configuration to flash (persists across power cycles)

### muni gps configure-rover

Configure ZED-F9P as an RTK rover.

```bash
muni gps configure-rover --port /dev/ttyACM0
```

This command:

- Enables NMEA output (GGA, RMC, GSV, GSA, VTG)
- Disables RTCM output
- Disables time mode
- Saves configuration to flash
