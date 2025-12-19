# RTK GPS

Centimeter-accurate positioning for georeferenced mapping.

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
│  │  │ (roof)      │    │             │    │ Port 2101               │  │   │
│  │  └─────────────┘    └─────────────┘    └─────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                     │                                        │
│                                     │ RTCM3 corrections                      │
│                                     │ (via LTE/VPN)                          │
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

## When You Need RTK

| Use Case | Standard GPS | RTK GPS |
| --- | --- | --- |
| Fleet tracking | ✅ Sufficient | Overkill |
| Rough geolocation | ✅ Sufficient | Overkill |
| Georeferenced mapping | ⚠️ ~3m error | ✅ ~2cm error |
| Multi-session map fusion | ❌ Drift issues | ✅ Consistent |
| Autonomous navigation | ⚠️ Lane-level | ✅ Centimeter |

**For bvr0**: Standard GPS is fine for prototyping.
**For bvr1**: RTK enables street-view quality georeferenced mapping.

## Hardware

### Rover Receiver

| Component | Model | Price | Source |
| --- | --- | --- | --- |
| GPS Module | [SparkFun GPS-RTK2 (ZED-F9P)](https://www.amazon.com/SparkFun-GPS-RTK2-Board-ZED-F9P-Qwiic/dp/B07NBPNWNZ) | ~$220 | Amazon |
| Antenna | SparkFun GNSS Multi-Band | ~$75 | SparkFun |
| SMA Cable | RG174, 1m | ~$10 | Amazon |

### Base Station (Depot)

| Component | Model | Price | Notes |
| --- | --- | --- | --- |
| GPS Module | SparkFun GPS-RTK2 (ZED-F9P) | ~$220 | Same as rover |
| Antenna | Tallysman TW4721 (survey-grade) | ~$100 | Better multipath rejection |
| SMA Cable | LMR-400, 25ft | ~$40 | Low-loss for roof run |
| Raspberry Pi | Pi 4 (2GB+) | ~$50 | Runs NTRIP caster |

**Total base station: ~$410**

Alternatively, integrate into the depot server directly via USB.

## Rover Integration

### Hardware Connection

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

To receive corrections from the depot base station:

```rust
// crates/gps/src/ntrip.rs

pub struct NtripClient {
    server: String,
    port: u16,
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

## Base Station Setup

### Hardware Installation

1. **Mount antenna** on depot roof with clear sky view (no obstructions above 10°)
2. **Run SMA cable** through building to server rack
3. **Connect ZED-F9P** to depot server via USB
4. **Survey position** to establish precise base coordinates

### Survey Procedure

The base station must know its exact position. Options:

| Method | Accuracy | Time |
| --- | --- | --- |
| Self-survey (1hr) | ±1m | 1 hour |
| Self-survey (24hr) | ±10cm | 24 hours |
| PPP service | ±2cm | Submit 24hr log, wait 1 week |
| Professional survey | ±1cm | Hire surveyor |

For most applications, 24-hour self-survey is sufficient:

```bash
# Configure ZED-F9P for survey-in mode
# (via u-center or pyubx2)
ubxtool -p CFG-TMODE-MODE,1        # Survey-in mode
ubxtool -p CFG-TMODE-SVIN_MIN_DUR,86400  # 24 hours
ubxtool -p CFG-TMODE-SVIN_ACC_LIMIT,100  # 10cm target
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
      - /dev/ttyUSB0:/dev/ttyUSB0  # ZED-F9P
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

| Provider | Coverage | Price |
| --- | --- | --- |
| [PointOne Nav](https://pointonenav.com/) | USA/EU | ~$50/mo |
| [Skylark](https://www.swiftnav.com/skylark) | Global | ~$50/mo |
| State DOT CORS | Varies | Free (some states) |

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

| Condition | Horizontal | Vertical |
| --- | --- | --- |
| RTK fixed (ideal) | ±1cm | ±2cm |
| RTK float | ±20cm | ±40cm |
| DGPS (no RTK) | ±50cm | ±1m |
| Standard GPS | ±2-5m | ±5-10m |

RTK requires:
- Clear sky view (>5 satellites)
- Corrections from base station
- Base station within ~20km (for single-base RTK)

## Troubleshooting

### No RTK Fix

```bash
# Check satellite count
ubxtool -p NAV-PVT | grep numSV

# Check correction age
ubxtool -p NAV-PVT | grep diffAge

# Verify NTRIP connection
curl -v http://depot:2101/
```

### Frequent Fix Loss

- Check antenna placement (obstructions?)
- Verify correction stream continuity
- Consider longer cable or better antenna

### Base Station Issues

```bash
# Check survey status
ubxtool -p NAV-SVIN

# Verify RTCM output
ubxtool -p CFG-MSGOUT-RTCM*
```
