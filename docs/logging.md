# BVR Logging and Telemetry Infrastructure

This document describes BVR's logging, telemetry recording, and data sync architecture.

## Overview

BVR uses a dual-layer approach to data capture:

1. **Event Logging** (`tracing`): Text-based logs for operational events, errors, and debugging
2. **Telemetry Recording** (`rerun`): Time-series sensor data for playback and analysis

```
bvrd
├── tracing → stdout + /var/log/bvr/bvrd.log (rolling daily)
│   └── Events: state transitions, errors, warnings
│
└── rerun → /var/log/bvr/sessions/*.rrd
    └── Telemetry: pose, velocity, motors, GPS, tools
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Rover (Jetson)                             │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  bvrd                                                        │   │
│  │                                                              │   │
│  │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │   │
│  │  │   tracing    │    │   Recorder   │    │   Teleop     │  │   │
│  │  │              │    │   (rerun)    │    │   (UDP)      │  │   │
│  │  └──────┬───────┘    └──────┬───────┘    └──────────────┘  │   │
│  │         │                   │                               │   │
│  │         ▼                   ▼                               │   │
│  │  /var/log/bvr/       /var/log/bvr/sessions/                │   │
│  │    bvrd.log            {rover}_{timestamp}.rrd              │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                │                                    │
│  ┌─────────────────────────────┴───────────────────────────────┐   │
│  │  bvr-sync.timer (every 15 min)                               │   │
│  │    └── rclone sync → base station                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 │ rclone (SFTP/S3/WebDAV)
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Base Station                                 │
│                                                                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │  Session Store  │  │    InfluxDB     │  │      Grafana        │ │
│  │  (.rrd files)   │  │   (metrics)     │  │   (dashboards)      │ │
│  └────────┬────────┘  └────────┬────────┘  └─────────────────────┘ │
│           │                    │                                    │
│           ▼                    ▼                                    │
│     Rerun Viewer         Fleet Dashboard                            │
└─────────────────────────────────────────────────────────────────────┘
```

## Layer 1: Event Logging (tracing)

### Configuration

In `config/bvr.toml`:

```toml
[logging]
level = "info"           # trace, debug, info, warn, error
dir = "/var/log/bvr"     # Log directory
rotation = "daily"       # daily, hourly, never
retain_days = 7          # Days to keep logs
```

### CLI Overrides

```bash
# Set log level via environment
RUST_LOG=debug bvrd

# Or via CLI argument
bvrd --log-level debug
```

### Output Destinations

| Destination             | Level           | Format                  |
| ----------------------- | --------------- | ----------------------- |
| stdout                  | From config/env | Colored, human-readable |
| `/var/log/bvr/bvrd.log` | Same            | Plain text, timestamped |

### Log Rotation

- Daily rotation by default (configurable)
- Old logs: `bvrd.log.2024-12-17` etc.
- Synced to base station by `bvr-sync`

### Usage in Code

```rust
use tracing::{info, warn, error, debug, trace};

// Structured fields
info!(mode = ?new_mode, "State transition");

// Error context
error!(?err, "CAN send failed");

// High-frequency (use trace, not debug)
trace!(rpm = wheel_rpms[0], "Wheel command");
```

## Layer 2: Telemetry Recording (Rerun)

### Configuration

In `config/bvr.toml`:

```toml
[recording]
enabled = true
session_dir = "/var/log/bvr/sessions"
max_storage_bytes = 10_737_418_240  # 10 GB
include_camera = false               # High bandwidth

[identity]
rover_id = "bvr-01"
```

### What's Recorded

| Entity Path                         | Data Type   | Rate      | Description           |
| ----------------------------------- | ----------- | --------- | --------------------- |
| `robot/pose`                        | Transform3D | 100 Hz    | Position and heading  |
| `robot/trajectory`                  | Points2D    | 100 Hz    | 2D trajectory overlay |
| `robot/heading`                     | Scalar      | 100 Hz    | Heading time-series   |
| `velocity/linear/commanded`         | Scalar      | 100 Hz    | Commanded linear vel  |
| `velocity/linear/actual`            | Scalar      | 100 Hz    | Actual linear vel     |
| `velocity/angular/commanded`        | Scalar      | 100 Hz    | Commanded angular vel |
| `velocity/angular/actual`           | Scalar      | 100 Hz    | Actual angular vel    |
| `motors/{fl,fr,rl,rr}/current`      | Scalar      | 100 Hz    | Per-motor current     |
| `motors/{fl,fr,rl,rr}/temp`         | Scalar      | 100 Hz    | Per-motor temperature |
| `motors/total_current`              | Scalar      | 100 Hz    | Sum of motor currents |
| `power/battery_voltage`             | Scalar      | 100 Hz    | Battery voltage       |
| `power/system_current`              | Scalar      | 100 Hz    | System current        |
| `power/power_watts`                 | Scalar      | 100 Hz    | Instantaneous power   |
| `odometry/{dx,dy,dtheta}`           | Scalar      | 100 Hz    | Wheel odometry deltas |
| `gps/position`                      | Points2D    | ~1 Hz     | GPS coordinates       |
| `gps/{latitude,longitude,accuracy}` | Scalar      | ~1 Hz     | GPS details           |
| `state/mode`                        | TextLog     | On change | Mode transitions      |
| `events`                            | TextLog     | Sparse    | Warnings, errors      |
| `tools/{name}/position`             | Scalar      | 100 Hz    | Tool position         |
| `tools/{name}/current`              | Scalar      | 100 Hz    | Tool current          |
| `camera/front`                      | Image       | Optional  | Camera frames         |

### Session Files

- Location: `/var/log/bvr/sessions/`
- Naming: `{rover_id}_{unix_timestamp}.rrd`
- Example: `bvr-01_1702900800.rrd`

### Storage Management

- Max storage configurable (default 10 GB)
- Oldest sessions automatically deleted when limit reached
- Sync to base station preserves data before deletion

### Viewing Sessions

```bash
# Install Rerun viewer
pip install rerun-sdk
# or
cargo install rerun-cli

# View a session
rerun /var/log/bvr/sessions/bvr-01_1702900800.rrd
```

## Layer 3: Automatic Sync

### Overview

The `bvr-sync` timer automatically syncs recordings and logs to a base station.

### Configuration

In `config/bvr.toml`:

```toml
[sync]
enabled = true
destination = "base:bvr-sessions"  # rclone remote:path
interval_secs = 900                 # 15 minutes
min_age_secs = 60                   # Don't sync active files
```

### rclone Setup

1. Copy example config:

   ```bash
   sudo cp /etc/bvr/rclone.conf.example /etc/bvr/rclone.conf
   ```

2. Edit for your base station:

   ```bash
   sudo nano /etc/bvr/rclone.conf
   ```

3. Test connection:
   ```bash
   rclone ls base:bvr-sessions --config /etc/bvr/rclone.conf
   ```

### Systemd Timer

```bash
# Check timer status
systemctl status bvr-sync.timer

# View next run time
systemctl list-timers bvr-sync.timer

# Manually trigger sync
sudo systemctl start bvr-sync.service

# View sync logs
journalctl -u bvr-sync.service -f
```

### Sync Destinations

See `config/rclone.conf.example` for options:

| Type             | Example           | Use Case                |
| ---------------- | ----------------- | ----------------------- |
| SFTP             | `user@host:/data` | Local NAS, base station |
| S3               | `s3:bucket/path`  | Cloud storage, MinIO    |
| WebDAV           | Nextcloud         | Existing infrastructure |
| Tailscale + SFTP | `100.x.y.z:/data` | LTE rovers              |

### Directory Structure on Base Station

```
bvr-sessions/
├── bvr-01/
│   ├── sessions/
│   │   ├── bvr-01_1702900800.rrd
│   │   ├── bvr-01_1702904400.rrd
│   │   └── ...
│   └── logs/
│       ├── bvrd.log.2024-12-17
│       ├── bvrd.log.2024-12-16
│       └── ...
├── bvr-02/
│   └── ...
└── ...
```

## Base Station Setup

### Prerequisites

- Docker and Docker Compose
- 100+ GB storage for session files
- Network accessible from rovers (Tailscale recommended)

### Quick Start

```bash
# Create docker-compose.yml (see below)
docker compose up -d
```

### Docker Compose

```yaml
version: "3.8"

services:
  # Time-series database for metrics
  influxdb:
    image: influxdb:2.7
    ports:
      - "8086:8086"
    volumes:
      - influxdb-data:/var/lib/influxdb2
    environment:
      - DOCKER_INFLUXDB_INIT_MODE=setup
      - DOCKER_INFLUXDB_INIT_USERNAME=admin
      - DOCKER_INFLUXDB_INIT_PASSWORD=changeme
      - DOCKER_INFLUXDB_INIT_ORG=muni
      - DOCKER_INFLUXDB_INIT_BUCKET=bvr

  # Dashboards
  grafana:
    image: grafana/grafana:10.2.0
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=changeme
    depends_on:
      - influxdb

  # S3-compatible object storage for sessions
  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio-data:/data
    environment:
      - MINIO_ROOT_USER=admin
      - MINIO_ROOT_PASSWORD=changeme
    command: server /data --console-address ":9001"

volumes:
  influxdb-data:
  grafana-data:
  minio-data:
```

### Grafana Dashboard Setup

1. Open http://localhost:3000 (admin/changeme)
2. Add InfluxDB data source:
   - URL: http://influxdb:8086
   - Organization: muni
   - Token: (from InfluxDB setup)
   - Default bucket: bvr
3. Import or create dashboards for:
   - Fleet overview (map, status indicators)
   - Per-rover detail (battery, motors, velocity)
   - Alerts (offline rovers, faults)

### MinIO Setup for Session Storage

1. Open http://localhost:9001 (admin/changeme)
2. Create bucket: `bvr-sessions`
3. Create access key for rclone
4. Configure rovers with S3 rclone remote

## Deployment

### Install on Rover

```bash
# Full deploy including sync infrastructure
./deploy.sh frog-0 --all

# Or just sync infrastructure
./deploy.sh frog-0 --sync
```

### Configure rclone on Rover

```bash
ssh frog-0

# Configure rclone for your base station
sudo cp /etc/bvr/rclone.conf.example /etc/bvr/rclone.conf
sudo nano /etc/bvr/rclone.conf

# Test
sudo /opt/bvr/bin/bvr-sync.sh
```

### Verify

```bash
# Check recording is active
journalctl -u bvrd | grep "Recording session"

# Check sync timer
systemctl status bvr-sync.timer

# List local sessions
ls -la /var/log/bvr/sessions/
```

## Troubleshooting

### No .rrd Files Created

1. Check recording is enabled in config
2. Check bvrd logs: `journalctl -u bvrd | grep -i record`
3. Verify directory exists: `ls -la /var/log/bvr/sessions/`
4. Check disk space: `df -h /var/log/bvr`

### Sync Not Working

1. Check timer is active: `systemctl status bvr-sync.timer`
2. Run manual sync: `sudo /opt/bvr/bin/bvr-sync.sh`
3. Test rclone directly:
   ```bash
   rclone ls base: --config /etc/bvr/rclone.conf
   ```
4. Check network: `ping base-station.local`

### Viewer Won't Open .rrd File

1. Check file isn't corrupted: `file session.rrd`
2. Try different viewer version: `pip install --upgrade rerun-sdk`
3. Check Rerun version compatibility (recorded with 0.22.x)

### High Disk Usage

1. Check session rotation is working
2. Reduce `max_storage_bytes` in config
3. Verify sync is uploading files
4. Disable camera recording if enabled

## Future Enhancements

### Planned

- [ ] Real-time metrics push to InfluxDB (for live dashboard)
- [ ] Camera frame recording (configurable quality/rate)
- [ ] Fleet-wide alerting (PagerDuty/Slack)
- [ ] Session annotations (mark events for review)

### Under Consideration

- Ring buffer for trace-level data (dump on fault)
- Compressed log shipping
- Edge analytics (anomaly detection on rover)
- Multi-camera support
