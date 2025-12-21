# Depot Rack

The depot is the base station for fleet operations. It houses the control
server, RTK base station, and operator display in a compact 10" mini rack.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Depot Rack (10")                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          3U Touchscreen                             │   │
│  │                    Grafana / Operator / Fleet Map                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  PoE Switch (USW-Flex) ◄── DC from Power Station            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  Raspberry Pi 5 + P31 HAT ◄── PoE (native)                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  RTK GPS (ZED-F9P) + USB Hub                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1U  │  Patch Panel / Cable Management                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│                   External: Power Station + GNSS Antenna                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Architecture

The depot uses PoE for internal power distribution. A single DC input from an
external power station feeds the PoE switch, which distributes power to all
rack components. The Raspberry Pi 5 with GeeekPi P31 HAT supports native PoE+,
eliminating the need for splitters on the compute side.

```
Power Station (external)
    │
    │ 12V DC (or AC → adapter)
    ▼
┌─────────────────────────────────────────────────┐
│ PoE Switch (USW-Flex)                           │
│   ├── PoE+ ─► RPi5 + P31 HAT (native PoE+)      │
│   ├── PoE ──► Display (via 5V splitter)         │
│   └── ETH ──► Rovers / Uplink                   │
└─────────────────────────────────────────────────┘
         │
         USB
         ▼
    ZED-F9P (RTK)
         │
         SMA cable (to roof)
         ▼
    GNSS Antenna
```

## Bill of Materials

### Rack Components

| U   | Component    | Model                         | Price | Notes                  |
| --- | ------------ | ----------------------------- | ----- | ---------------------- |
| —   | Rack         | GeeekPi 10" 6U Open Frame     | $80   | Wall or desk mount     |
| 3U  | Display      | GeeekPi 10.1" IPS Touchscreen | $110  | 1024×600, HDMI         |
| 1U  | PoE Switch   | Ubiquiti USW-Flex             | $100  | 5-port, 46W PoE budget |
| 1U  | Shelf        | 10" rack shelf                | $15   | For RTK + USB hub      |
| 1U  | Patch Panel  | 10" CAT6 6-port               | $25   | Cable management       |
|     | **Subtotal** |                               | $330  |                        |

### Compute (Raspberry Pi 5)

| Component | Model                 | Price | Notes                             |
| --------- | --------------------- | ----- | --------------------------------- |
| SBC       | Raspberry Pi 5 8GB    | $80   | ARM64, runs Docker                |
| HAT       | GeeekPi P31 NVMe PoE+ | $35   | PoE+ 802.3at, NVMe 2230/2242, fan |
| SSD       | WD SN740 2230 256GB   | $30   | Reliable, plenty for sessions     |
|           | **Subtotal**          | $145  |                                   |

The GeeekPi P31 HAT provides:

- Native PoE+ power (no splitter needed)
- M.2 NVMe slot (2230/2242)
- Active cooling with official fan

#### Alternative: x86 Compute

For more headroom or x86 compatibility:

| Component | Model        | Price | Notes                  |
| --------- | ------------ | ----- | ---------------------- |
| Mini PC   | Beelink EQ12 | $200  | N100, 16GB, 500GB NVMe |
| Splitter  | PoE to 12V   | $18   | 802.3at for PoE power  |
|           | **Subtotal** | $218  | +$73 vs RPi5           |

### PoE Splitters

| Device  | Model                      | Output  | Price | Notes   |
| ------- | -------------------------- | ------- | ----- | ------- |
| Display | UCTRONICS PoE to micro-USB | 5V 2.5A | $12   | 802.3af |
|         | **Subtotal**               |         | $12   |         |

### RTK Base Station

| Component | Model                     | Price | Notes                    |
| --------- | ------------------------- | ----- | ------------------------ |
| GPS Board | SparkFun GPS-RTK2 ZED-F9P | $275  | USB-C to compute         |
| Antenna   | Tallysman TW4721          | $100  | Survey-grade, roof mount |
| Cable     | LMR-400 25ft + SMA        | $50   | Low-loss for roof run    |
| Mount     | Magnetic or pole mount    | $20   | Secure roof mounting     |
|           | **Subtotal**              | $445  |                          |

### External Power

| Component     | Model               | Price | Notes                   |
| ------------- | ------------------- | ----- | ----------------------- |
| Power Station | EcoFlow River 3     | $200  | 245Wh, solar-ready      |
| DC Cable      | 12V barrel to PoE   | $10   | Or use AC outlet        |
|               | **Subtotal**        | $210  |                         |

### Total Cost

| Category         | Cost    |
| ---------------- | ------- |
| Rack components  | $330    |
| Compute (RPi5)   | $145    |
| PoE splitters    | $12     |
| RTK base station | $445    |
| External power   | $210    |
| **Total**        | ~$1,140 |

## Software Stack

The depot runs Docker services on the Raspberry Pi 5:

| Service   | Port | Purpose                      |
| --------- | ---- | ---------------------------- |
| Discovery | 4860 | Rover registration/discovery |
| Operator  | 8080 | Web-based teleop interface   |
| InfluxDB  | 8086 | Time-series metrics storage  |
| Grafana   | 3000 | Fleet dashboards             |
| SFTP      | 2222 | Session file uploads         |
| NTRIP     | 2101 | RTK corrections broadcast    |

See [depot/README.md](../../depot/README.md) for setup instructions.

## Power Budget

| Component  | Power   | Notes             |
| ---------- | ------- | ----------------- |
| RPi5 + P31 | 5-12W   | Idle vs full load |
| Display    | 5W      | Backlight at 50%  |
| PoE Switch | 3W      | Self-consumption  |
| ZED-F9P    | 0.5W    | USB-powered       |
| **Total**  | ~15-22W |                   |

With the EcoFlow River 3 (245Wh), expect:

- **~11-16 hours** at typical load
- **Indefinite** with 100W solar panel attached

## Assembly

### 1. Rack Setup

1. Assemble 10" rack frame
2. Mount to wall or set on desk
3. Install 10" shelf and patch panel

### 2. Compute Installation

1. Attach GeeekPi P31 HAT to Raspberry Pi 5
2. Install WD SN740 NVMe SSD in M.2 slot
3. Mount Pi + HAT assembly on rack shelf
4. Connect single ethernet cable (provides power + network)
5. Connect HDMI to display
6. Connect USB: ZED-F9P, optional peripherals

### 3. Networking

1. Mount PoE switch in 1U slot
2. Run ethernet from PoE switch to:
   - RPi5 (native PoE+ via P31 HAT)
   - Display (via PoE splitter)
   - Uplink (to LAN/internet)
3. Connect power station to PoE switch

### 4. RTK Base Station

1. Mount GNSS antenna on roof with clear sky view
2. Run LMR-400 cable from antenna to rack
3. Connect antenna to ZED-F9P via SMA
4. Connect ZED-F9P to compute via USB
5. Configure for base station mode (see [rtk.md](rtk.md))

### 5. Software

```bash
cd depot
docker compose up -d
```

Access Grafana at `http://localhost:3000` to verify services.

## Network Ports

| Port | Protocol | Service   | Purpose            |
| ---- | -------- | --------- | ------------------ |
| 2101 | TCP      | NTRIP     | RTK corrections    |
| 2222 | TCP      | SFTP      | Session uploads    |
| 3000 | TCP      | Grafana   | Fleet dashboards   |
| 4860 | TCP      | Discovery | Rover registration |
| 8080 | TCP      | Operator  | Web teleop         |
| 8086 | TCP      | InfluxDB  | Metrics API        |
| 8089 | UDP      | InfluxDB  | Metrics push       |

## Field Deployment

For temporary or field deployments:

1. Charge power station fully
2. Set up rack (desk or wall mount)
3. Deploy GNSS antenna with temporary tripod
4. Connect to rovers via LTE or local WiFi
5. Monitor via touchscreen

The compact form factor fits in a vehicle for mobile base operations.
