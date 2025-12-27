# Depot

Depot is the base station infrastructure for the Muni robot fleet. It provides:

- **Web operator** for browser-based teleop with 360° video
- **Real-time metrics** via InfluxDB (battery, motors, GPS, mode)
- **Fleet dashboards** via Grafana (fleet overview + per-rover detail)
- **Session storage** via SFTP (rovers upload recordings)
- **Map processing** via Mapper service (Gaussian splatting pipeline)
- **Map serving** via Map API (browse and download 3D maps)
- **Automatic retention** (30-day cleanup by default)

## Architecture

```
Rovers                          Depot
┌─────────┐                     ┌─────────────────────────────┐
│ rover-01│──HTTP register────▶│  Discovery (:4860)          │
│ rover-02│──UDP metrics──────▶│  InfluxDB (:8086, :8089)    │
│ rover-0N│──rclone SFTP──────▶│  SFTP (:2222)               │
│         │◀─WebSocket─────────│  Operator (:8080)           │
└─────────┘                     │  Grafana (:3000)            │
                                └─────────────────────────────┘
```

## Quick Start

### Prerequisites

- Docker and Docker Compose
- 100+ GB storage for session files
- Network accessible from rovers

### Quick Start

```bash
cd depot
docker compose up -d
```

This starts all services with default development credentials.

### Access

| Service   | URL                   | Default Credentials  |
| --------- | --------------------- | -------------------- |
| Operator  | http://localhost:8080 | None (public)        |
| Grafana   | http://localhost:3000 | admin / munipassword |
| InfluxDB  | http://localhost:8086 | admin / munipassword |
| SFTP      | localhost:2222        | muni / SSH key auth  |
| Discovery | http://localhost:4860 | None (internal)      |
| Map API   | http://localhost:4870 | None (internal)      |

## Development

For developing the operator web app with hot-reload (no container rebuilds):

```bash
# Terminal 1: Start backend services (discovery, influxdb, grafana)
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# Terminal 2: Run operator with hot-reload
cd operator
npm install
npm run dev
```

The operator dev server runs on http://localhost:5173 with:

- Hot module replacement (instant updates)
- Auto-refresh on file changes
- Source maps for debugging

### Simulating Rovers

You can register mock rovers for testing:

```bash
# Register a rover
curl -X POST http://localhost:4860/register \
  -H "Content-Type: application/json" \
  -d '{"id":"bvr-01","name":"Beaver-01","address":"ws://localhost:4850"}'

# Send heartbeat (keeps rover online)
curl -X POST http://localhost:4860/heartbeat/bvr-01 \
  -H "Content-Type: application/json" \
  -d '{"battery_voltage":48.5,"mode":1,"pose":{"x":10,"y":5,"theta":0.5}}'
```

### Production Setup

For production, use `./scripts/setup.sh` to generate secure credentials,
or create a `.env` file with custom values (see Configuration below).

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
# InfluxDB
INFLUXDB_ADMIN_USER=admin
INFLUXDB_ADMIN_PASSWORD=<secure-password>
INFLUXDB_ORG=muni
INFLUXDB_BUCKET=muni
INFLUXDB_ADMIN_TOKEN=<secure-token>

# Grafana
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=<secure-password>

# Storage
SESSIONS_PATH=/data/muni-sessions
RETENTION_DAYS=30
```

### Adding Rover SSH Keys

Rovers authenticate to SFTP using SSH keys. Add each rover's public key:

```bash
# Copy rover's public key
cp /path/to/bvr-01.pub sftp/authorized_keys/

# Restart SFTP to pick up new keys
docker compose restart sftp
```

On the rover side, configure rclone with the corresponding private key.

## Rover Configuration

Configuration varies by morphology. See the morphology-specific docs:

- **BVR**: See [bvr/docs/](../bvr/docs/) for BVR-specific configuration

### Metrics Push (General)

Rovers push metrics via UDP to InfluxDB. Example config:

```toml
[metrics]
enabled = true
endpoint = "depot.local:8089"  # or IP address
interval_hz = 1
```

### Session Sync (General)

Configure rclone on each rover for session upload:

```ini
[base]
type = sftp
host = depot.local
port = 2222
user = muni
key_file = /etc/muni/id_ed25519
```

## Dashboards

### Fleet Overview

The Fleet Overview dashboard shows:

- Rover status table (online/offline, last seen)
- Battery levels bar chart
- Motor temperatures (max per rover)
- GPS map (if available)
- Alert indicators (offline, low battery, high temp)

### Rover Detail

Select a rover from the dropdown to see:

- Current mode, battery, current
- Battery voltage over time
- Motor temperatures over time
- Velocity (commanded vs actual)
- Motor currents
- Mode timeline

## Maintenance

### Session Cleanup

Sessions older than `RETENTION_DAYS` are automatically cleaned up. Run manually:

```bash
./scripts/cleanup.sh
```

Or set up a cron job:

```bash
# Run daily at 3am
0 3 * * * /opt/depot/scripts/cleanup.sh >> /var/log/depot-cleanup.log 2>&1
```

### Backup

Important data to back up:

- `.env` (credentials)
- `sftp/authorized_keys/` (rover SSH keys)
- Docker volumes (influxdb-data, grafana-data)

```bash
# Backup volumes
docker run --rm -v depot_influxdb-data:/data -v $(pwd):/backup alpine \
    tar czf /backup/influxdb-backup.tar.gz /data
```

### Viewing Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f influxdb
docker compose logs -f grafana
docker compose logs -f sftp
```

## Troubleshooting

### No metrics appearing in Grafana

1. Check rover is sending metrics:

   ```bash
   # On depot, listen for UDP packets
   tcpdump -i any port 8089 -A
   ```

2. Verify InfluxDB UDP listener is working:

   ```bash
   docker compose logs influxdb | grep -i udp
   ```

3. Check Grafana data source is configured correctly

### Rovers can't connect via SFTP

1. Check SFTP container is running:

   ```bash
   docker compose ps sftp
   ```

2. Verify rover's public key is in `sftp/authorized_keys/`

3. Test connection from rover:
   ```bash
   sftp -P 2222 -i /path/to/key muni@depot.local
   ```

### High disk usage

1. Check session storage:

   ```bash
   du -sh /data/muni-sessions/*
   ```

2. Run cleanup manually:

   ```bash
   ./scripts/cleanup.sh
   ```

3. Reduce `RETENTION_DAYS` in `.env`

## Network Ports

| Port | Protocol | Service   | Purpose               |
| ---- | -------- | --------- | --------------------- |
| 2222 | TCP      | SFTP      | Session file uploads  |
| 3000 | TCP      | Grafana   | Web dashboards        |
| 4860 | TCP      | Discovery | Rover registration    |
| 4870 | TCP      | Map API   | Map serving           |
| 8080 | TCP      | Operator  | Web teleop interface  |
| 8086 | TCP      | InfluxDB  | HTTP API + Web UI     |
| 8089 | UDP      | InfluxDB  | Line protocol metrics |

## RTK Base Station

For centimeter-accurate georeferenced mapping, the depot can host an RTK GPS
base station that broadcasts corrections to rovers.

### Hardware

| Component  | Model            | Notes                   |
| ---------- | ---------------- | ----------------------- |
| GPS Module | SparkFun ZED-F9P | USB to depot server     |
| Antenna    | Tallysman TW4721 | Roof-mounted, clear sky |
| Cable      | LMR-400, 25ft    | Low-loss for roof run   |

Total: ~$360 (module + antenna + cable)

### Architecture

```
Roof
  │
  ▼ GNSS Antenna
  │
  │ SMA/LMR-400
  ▼
┌─────────────┐
│ ZED-F9P     │──USB──▶ Depot Server
│ (Base Mode) │
└─────────────┘
       │
       ▼ RTCM3 corrections
┌─────────────┐
│ NTRIP       │◀──TCP:2101── Rovers
│ Caster      │
└─────────────┘
```

### Docker Setup

Add to `docker-compose.yml`:

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
```

### Rover Configuration

On each rover, configure NTRIP client. Example for BVR (`bvr.toml`):

```toml
[gps]
ntrip_enabled = true
ntrip_server = "depot.local"
ntrip_port = 2101
ntrip_mountpoint = "ROVER"
```

### Survey Procedure

The base station must be surveyed to determine its precise location:

1. **Configure for survey-in mode** (24 hours recommended)
2. **Wait for position to converge** (10cm accuracy target)
3. **Save fixed coordinates** to config

See morphology-specific docs for detailed RTK setup instructions.

### Network Ports

Add to the ports table:

| Port | Protocol | Service | Purpose                   |
| ---- | -------- | ------- | ------------------------- |
| 2101 | TCP      | NTRIP   | RTK corrections broadcast |

## Security Considerations

- SFTP uses key-based authentication only (no passwords)
- Grafana and InfluxDB use password auth (set strong passwords in .env)
- Consider placing behind a reverse proxy with TLS for production
- Use a VPN (WireGuard/Tailscale) for rovers connecting over public internet
