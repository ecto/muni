# Rover Networking

Network architecture for BVR rovers and depot communication.

## Overview

BVR uses a **Ubiquiti mesh network** for low-latency rover communication.
This eliminates LTE data costs and provides 5-50ms latency (vs 100-250ms with LTE).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Depot                                           │
│                                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────────────┐ │
│  │ Operator    │    │ InfluxDB    │    │ Rocket 5AC Prism                │ │
│  │ Station     │    │ SFTP, etc.  │    │ + Omni Antenna                  │ │
│  └──────┬──────┘    └──────┬──────┘    └───────────────┬─────────────────┘ │
│         └──────────────────┴───────────────────────────┘                    │
│                                         │ Ethernet                          │
└─────────────────────────────────────────┼───────────────────────────────────┘
                                          │
                         5 GHz Mesh (PtMP + rover-to-rover relay)
                                          │
          ┌───────────────────────────────┼───────────────────────────────┐
          │                               │                               │
          ▼                               ▼                               ▼
   ┌────────────┐                  ┌────────────┐                  ┌────────────┐
   │ Rover A    │ ◄──── mesh ────► │ Rover B    │ ◄──── mesh ────► │ Rover C    │
   │ NanoStation│                  │ NanoStation│                  │ NanoStation│
   └────────────┘                  └────────────┘                  └────────────┘

Optional: Fixed repeaters added where coverage gaps are identified
```

## Network Strategy: Mesh-First, Repeaters Where Needed

**Phase 1: Mesh Only**

- Deploy with depot base station + rover radios only
- Rovers relay for each other when out of direct range
- Mesh health metrics identify coverage gaps

**Phase 2: Data-Driven Repeaters**

- Analyze mesh metrics (RSSI, latency, hop count) from InfluxDB
- Add fixed repeaters only at specific problem locations
- Typical cost: $200-250 per repeater site

This approach minimizes upfront infrastructure while ensuring coverage grows with demand.

## Patron WiFi (Optional)

The depot mesh infrastructure can double as community WiFi, providing value to
the neighborhood and leverage for repeater placement negotiations.

### Requirements

- **ISP with redistribution rights**: AT&T Business (patron WiFi allowed) or
  Starlink Priority ($165+/mo, explicitly permits community hotspots)
- **Additional hardware**: UniFi AP at depot and/or repeater sites (~$99-180 each)
- **QoS configuration**: Prioritize rover traffic over public WiFi

### Architecture

```
Internet (AT&T Business or Starlink Priority)
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Depot Rack                                                   │
│                                                              │
│   USW-Flex PoE Switch                                        │
│       │                                                      │
│       ├── RPi5 (services: InfluxDB, Grafana, NTRIP)         │
│       ├── Display                                            │
│       └── UniFi AP AC Lite ──► "Muni-Public" SSID (2.4/5GHz)│
│                                                              │
│   [Separate 24V PoE Injector]                               │
│       └── Rocket 5AC Prism ──► airMAX mesh (rovers only)    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Traffic Isolation

| Network        | VLAN | Purpose                  | Priority           |
| -------------- | ---- | ------------------------ | ------------------ |
| Rover mesh     | 10   | Teleop, video, telemetry | High               |
| Depot services | 20   | InfluxDB, Grafana, SFTP  | Medium             |
| Patron WiFi    | 30   | Public internet access   | Low (rate-limited) |

### Bandwidth Allocation

| Traffic Class    | Guaranteed | Burst    | Notes               |
| ---------------- | ---------- | -------- | ------------------- |
| Rover operations | 50 Mbps    | 100 Mbps | Never degraded      |
| Patron WiFi      | 10 Mbps    | 50 Mbps  | Uses spare capacity |

### Traffic Breakdown (Per Rover)

| Stream          | Direction        | Protocol      | Port | Bandwidth          |
| --------------- | ---------------- | ------------- | ---- | ------------------ |
| Video           | Rover → Depot    | UDP/WebSocket | 4851 | ~2 Mbps            |
| Teleop commands | Operator → Rover | UDP           | 4840 | ~5 kbps            |
| Telemetry       | Rover → Depot    | UDP           | 8089 | ~10 kbps           |
| RTK corrections | Depot → Rover    | TCP (NTRIP)   | 2101 | ~1-2 kbps          |
| Session sync    | Rover → Depot    | SFTP          | 2222 | Burst (background) |

RTK corrections are negligible bandwidth but benefit from mesh's low latency
(5-20ms vs 50-150ms with LTE) for faster fix acquisition.

### Political Value

Offering free community WiFi changes the conversation with municipalities:

> "We're not just a snow removal company. We're deploying infrastructure that
> provides free internet to underserved neighborhoods."

This can be leveraged for:

- Repeater placement on city property
- Municipal contracts
- Positive press coverage
- Community goodwill

## bvr0: Direct Connection

For the prototype, connect the Mid-360 directly to the Jetson's Ethernet port.

```
Mid-360 ◄────── Ethernet ──────► Jetson Orin NX
                                      │
                                      │ USB
                                      ▼
                                 LTE Modem
```

### Configuration

**Jetson Ethernet (eth0)**:

```
IP: 192.168.1.1
Netmask: 255.255.255.0
```

**Livox Mid-360** (factory default):

```
IP: 192.168.1.10
Netmask: 255.255.255.0
```

No router/DHCP needed — static IPs on isolated network.

### Setup Commands

```bash
# Set static IP on Jetson
sudo nmcli con add type ethernet con-name lidar ifname eth0 \
  ipv4.addresses 192.168.1.1/24 ipv4.method manual

# Verify connectivity
ping 192.168.1.10
```

## bvr1: Switch Configuration

For production rovers, add a small unmanaged switch for flexibility.

```
Mid-360 ────────┐
                ├───── Switch ───── Jetson eth0
Debug Port ─────┘      (GS305)
```

### Recommended Hardware

| Component   | Model            | Price | Notes                     |
| ----------- | ---------------- | ----- | ------------------------- |
| Switch      | Netgear GS305    | ~$20  | 5-port gigabit, unmanaged |
| Alternative | TP-Link TL-SG105 | ~$15  | 5-port gigabit            |

Power consumption: ~2W

### Benefits

- Plug in laptop for on-rover debugging
- Add future sensors (additional cameras, etc.)
- Network tap for traffic analysis

## IP Addressing Scheme

For fleet operations, each rover gets a unique ID reflected in its IP.

| Device        | IP Pattern    | Example (Rover 1) |
| ------------- | ------------- | ----------------- |
| Jetson        | 192.168.1.1   | 192.168.1.1       |
| Mid-360       | 192.168.1.10  | 192.168.1.10      |
| Future sensor | 192.168.1.20+ | 192.168.1.20      |

Note: The Mid-360 IP is configurable via Livox Viewer or SDK. For fleet
deployment, consider giving each unit a unique IP or use the default and
rely on the isolated per-rover network.

## Mesh Hardware

### Depot Base Station

| Component | Model            | Purpose                 |
| --------- | ---------------- | ----------------------- |
| Radio     | Rocket 5AC Prism | High-power base station |
| Antenna   | AMO-5G13         | 360° omni, 13 dBi gain  |
| Switch    | USW-Flex         | PoE for radio           |

```bash
# Base station config (via Ubiquiti web UI)
# - Mode: Access Point PtMP
# - Frequency: 5 GHz (DFS channels for less interference)
# - Channel width: 40 MHz (balance of range/throughput)
# - Output power: Max (for range)
```

Estimated coverage: 1-1.5 km radius in urban, 360° coverage.

Product link: [Rocket 5AC Prism](https://store.ui.com/us/en/category/wireless-airmax-5ghz) ($249)

**Why omni:** Rovers dispatch in all directions from depot. Sector antenna
(AM-5G17-90, 17 dBi, 90°) available if longer range in one direction is needed.

### Rover Radio

For production rovers, use a low-profile combo antenna with external radio:

| Component     | Model                                                                      | Cost | Notes                       |
| ------------- | -------------------------------------------------------------------------- | ---- | --------------------------- |
| Combo antenna | [Proxicast ANT-520-421](https://www.amazon.com/dp/B0D7JDPD8X)              | $367 | 7-in-1: 4x4 5G + WiFi + GPS |
| Mesh radio    | [Bullet AC IP67](https://store.ui.com/us/en/category/wireless-airmax-5ghz) | $129 | External antenna, airMAX    |
| Adapter       | RP-SMA to N-type                                                           | $10  | Connect antenna to radio    |

**Why this setup:**

- Ultra low-profile (1.26" tall) for durability and aesthetics
- IP67 weatherproof, vehicle-grade construction
- GPS integrated (no separate antenna needed)
- LTE antenna ports available for optional fallback
- 16 ft cable leads for flexible mounting

```
┌─────────────────────────────────────────────────────┐
│                  Rover Antenna Stack                 │
│                                                      │
│   Proxicast 5-in-1 (roof mount, 1.25" hole)         │
│         │                                            │
│         ├── WiFi 5GHz ──► Bullet AC ──► Jetson eth1 │
│         ├── WiFi 5GHz ──► (second chain, optional)  │
│         ├── GPS ──► Jetson USB (u-blox module)      │
│         ├── LTE ──► (future: fallback modem)        │
│         └── LTE ──► (MIMO second chain)             │
│                                                      │
└─────────────────────────────────────────────────────┘
```

```bash
# Bullet AC config (via UISP or web UI)
# - Mode: Station PtMP
# - Connect to depot SSID
# - Enable WDS for mesh relay
# - Antenna gain: 3 dBi (match Proxicast spec)
```

**Alternative for prototypes:** NanoStation 5AC Loco ($49) is simpler but taller.

### Fixed Repeater (Optional)

Add only where mesh metrics show coverage gaps.

| Component     | Model                | Cost |
| ------------- | -------------------- | ---- |
| Radio         | NanoStation 5AC Loco | $49  |
| Enclosure     | Outdoor NEMA         | $30  |
| PoE injector  | Ubiquiti 24V         | $15  |
| Mount         | Pole clamps          | $20  |
| Power (grid)  | Outlet access        | -    |
| Power (solar) | 20W panel + battery  | $100 |

Total per site: $114 (grid) or $214 (solar)

### Mesh Health Monitoring

Rovers report mesh metrics to InfluxDB:

| Metric       | Description            | Alert Threshold |
| ------------ | ---------------------- | --------------- |
| `rssi`       | Signal strength (dBm)  | < -75 dBm       |
| `ccq`        | Connection quality (%) | < 70%           |
| `hop_count`  | Hops to depot          | > 4             |
| `latency_ms` | Round-trip to depot    | > 100 ms        |

Query coverage gaps:

```sql
SELECT mean("rssi"), mean("latency")
FROM "mesh"
WHERE time > now() - 7d
GROUP BY "rover", time(1h)
```

Grafana dashboard visualizes this as a coverage heatmap using GPS coordinates.

## Firewall Rules

On the Jetson, allow necessary traffic:

```bash
# Allow LiDAR traffic on local network
sudo ufw allow in on eth0 from 192.168.1.0/24

# Allow teleop ports from VPN
sudo ufw allow 4840/udp   # UDP teleop
sudo ufw allow 4850/tcp   # WebSocket teleop
sudo ufw allow 4851/tcp   # WebSocket video
sudo ufw allow 8080/tcp   # Dashboard
```

## Depot Network Services

| Service      | Port | Protocol | Purpose                |
| ------------ | ---- | -------- | ---------------------- |
| InfluxDB     | 8086 | HTTP     | Metrics storage        |
| InfluxDB UDP | 8089 | UDP      | Metrics ingestion      |
| Grafana      | 3000 | HTTP     | Dashboards             |
| SFTP         | 2222 | SSH      | Session file sync      |
| NTRIP        | 2101 | TCP      | RTK corrections (bvr1) |

## Remote Desktop Access

For GUI access to the Jetson (debugging, configuration, etc.), use VNC to mirror the physical display.

### VNC Setup (x11vnc)

```bash
# Install x11vnc on Jetson
sudo apt install -y x11vnc
```

**Important**: The X display and auth file differ between the GDM login screen and a logged-in user session:

| State            | Display | Xauthority                      |
| ---------------- | ------- | ------------------------------- |
| GDM login screen | `:0`    | `/run/user/124/gdm/Xauthority`  |
| User logged in   | `:1`    | `/run/user/1000/gdm/Xauthority` |

Find the correct values:

```bash
# List X servers and their auth files
ps aux | grep Xorg

# Example output shows two X servers:
# tty1 (GDM):  -auth /run/user/124/gdm/Xauthority
# tty2 (user): -auth /run/user/1000/gdm/Xauthority
```

### Start VNC Server

```bash
# If GNOME Remote Desktop is running, stop it first (it binds port 5900)
sudo systemctl stop gnome-remote-desktop
# Or: sudo fuser -k 5900/tcp

# For logged-in user session (display :1)
sudo x11vnc -auth /run/user/1000/gdm/Xauthority -display :1 -forever -nopw -rfbport 5900 &

# For GDM login screen (display :0)
sudo x11vnc -auth /run/user/124/gdm/Xauthority -display :0 -forever -nopw -rfbport 5900 &
```

### Connect from macOS

```bash
# Open built-in Screen Sharing
open vnc://frog-0:5900

# Or via Tailscale hostname
open vnc://bvr-01:5900
```

Works with Screens 5, macOS Screen Sharing, or any VNC client.

### Firewall

```bash
sudo ufw allow 5900/tcp  # VNC
```

## Troubleshooting

### Mid-360 Not Responding

```bash
# Check Ethernet link
ip link show eth0

# Check IP assignment
ip addr show eth0

# Scan for device
nmap -sn 192.168.1.0/24

# Check Livox ports
nc -zv 192.168.1.10 56000
```

### High Latency to Depot

```bash
# Check LTE signal
mmcli -m 0 --signal-get

# Test latency
ping depot.example.com

# Check for packet loss
mtr depot.example.com
```
