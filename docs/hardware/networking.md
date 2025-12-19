# Rover Networking

Network architecture for BVR rovers and depot communication.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Rover Network                                   │
│                                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                     │
│  │ Livox       │    │ Jetson      │    │ LTE Modem   │                     │
│  │ Mid-360     │    │ Orin NX     │    │ (Sierra)    │                     │
│  │             │    │             │    │             │                     │
│  │ 192.168.1.10│    │ 192.168.1.1 │    │ DHCP (WAN)  │                     │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                     │
│         │                  │                  │ USB                         │
│         │ Ethernet         │ Ethernet         │                             │
│         │                  │                  │                             │
│         └──────────────────┴──────────────────┘                             │
│                            │                                                 │
│                      Direct / Switch                                         │
└─────────────────────────────────────────────────────────────────────────────┘
                             │
                             │ LTE / Internet
                             ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Depot Network                                   │
│                                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │ Depot       │    │ InfluxDB    │    │ SFTP        │    │ RTK Base    │ │
│  │ Server      │    │ (metrics)   │    │ (files)     │    │ Station     │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

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

| Component | Model | Price | Notes |
| --- | --- | --- | --- |
| Switch | Netgear GS305 | ~$20 | 5-port gigabit, unmanaged |
| Alternative | TP-Link TL-SG105 | ~$15 | 5-port gigabit |

Power consumption: ~2W

### Benefits

- Plug in laptop for on-rover debugging
- Add future sensors (additional cameras, etc.)
- Network tap for traffic analysis

## IP Addressing Scheme

For fleet operations, each rover gets a unique ID reflected in its IP.

| Device | IP Pattern | Example (Rover 1) |
| --- | --- | --- |
| Jetson | 192.168.1.1 | 192.168.1.1 |
| Mid-360 | 192.168.1.10 | 192.168.1.10 |
| Future sensor | 192.168.1.20+ | 192.168.1.20 |

Note: The Mid-360 IP is configurable via Livox Viewer or SDK. For fleet
deployment, consider giving each unit a unique IP or use the default and
rely on the isolated per-rover network.

## LTE Connectivity

The rover connects to depot/operator via LTE modem.

### Sierra MC7455 Configuration

```bash
# Check modem status
mmcli -m 0

# Connect
sudo nmcli con add type gsm ifname cdc-wdm0 con-name lte apn "your_apn"
```

### VPN (Recommended for Production)

Use Tailscale or WireGuard for secure rover-to-depot communication:

```bash
# Install Tailscale
curl -fsSL https://tailscale.com/install.sh | sh

# Connect to tailnet
sudo tailscale up --authkey=YOUR_KEY
```

Benefits:
- NAT traversal (no port forwarding needed)
- Encrypted tunnel
- Stable IPs across cellular connections

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

| Service | Port | Protocol | Purpose |
| --- | --- | --- | --- |
| InfluxDB | 8086 | HTTP | Metrics storage |
| InfluxDB UDP | 8089 | UDP | Metrics ingestion |
| Grafana | 3000 | HTTP | Dashboards |
| SFTP | 2222 | SSH | Session file sync |
| NTRIP | 2101 | TCP | RTK corrections (bvr1) |

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
