# Depot

Depot is the base station infrastructure for the BVR rover fleet. It provides:

- **Real-time metrics** via InfluxDB (battery, motors, GPS, mode)
- **Fleet dashboards** via Grafana (fleet overview + per-rover detail)
- **Session storage** via SFTP (rovers upload .rrd recordings)
- **Automatic retention** (30-day cleanup by default)

## Architecture

```
Rovers                          Depot
┌─────────┐                     ┌─────────────────────────────┐
│  bvr-01 │──UDP metrics──────▶│  InfluxDB (:8086, :8089)    │
│  bvr-02 │──rclone SFTP──────▶│  SFTP (:2222)               │
│  bvr-0N │                     │  Grafana (:3000)            │
└─────────┘                     └─────────────────────────────┘
```

## Quick Start

### Prerequisites

- Docker and Docker Compose
- 100+ GB storage for session files
- Network accessible from rovers

### Setup

```bash
cd depot
./scripts/setup.sh
```

This will:

1. Create `.env` with secure random credentials
2. Create the sessions directory
3. Start the Docker Compose stack

### Access

After setup, access the services:

| Service  | URL                   | Credentials         |
| -------- | --------------------- | ------------------- |
| Grafana  | http://localhost:3000 | admin / (from .env) |
| InfluxDB | http://localhost:8086 | admin / (from .env) |
| SFTP     | localhost:2222        | bvr / SSH key auth  |

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
# InfluxDB
INFLUXDB_ADMIN_USER=admin
INFLUXDB_ADMIN_PASSWORD=<secure-password>
INFLUXDB_ORG=muni
INFLUXDB_BUCKET=bvr
INFLUXDB_ADMIN_TOKEN=<secure-token>

# Grafana
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=<secure-password>

# Storage
SESSIONS_PATH=/data/bvr-sessions
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

### Metrics Push

In the rover's `bvr.toml`:

```toml
[metrics]
enabled = true
endpoint = "depot.local:8089"  # or IP address
interval_hz = 1
```

Or via CLI:

```bash
bvrd --metrics-endpoint depot.local:8089
```

### Session Sync

Configure rclone on each rover (`/etc/bvr/rclone.conf`):

```ini
[base]
type = sftp
host = depot.local
port = 2222
user = bvr
key_file = /etc/bvr/id_ed25519
```

And in `bvr.toml`:

```toml
[sync]
enabled = true
destination = "base:sessions"
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
   sftp -P 2222 -i /etc/bvr/id_ed25519 bvr@depot.local
   ```

### High disk usage

1. Check session storage:

   ```bash
   du -sh /data/bvr-sessions/*
   ```

2. Run cleanup manually:

   ```bash
   ./scripts/cleanup.sh
   ```

3. Reduce `RETENTION_DAYS` in `.env`

## Network Ports

| Port | Protocol | Service  | Purpose               |
| ---- | -------- | -------- | --------------------- |
| 2222 | TCP      | SFTP     | Session file uploads  |
| 3000 | TCP      | Grafana  | Web dashboards        |
| 8086 | TCP      | InfluxDB | HTTP API + Web UI     |
| 8089 | UDP      | InfluxDB | Line protocol metrics |

## Security Considerations

- SFTP uses key-based authentication only (no passwords)
- Grafana and InfluxDB use password auth (set strong passwords in .env)
- Consider placing behind a reverse proxy with TLS for production
- Use a VPN (WireGuard/Tailscale) for rovers connecting over public internet

