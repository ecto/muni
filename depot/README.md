# Depot

Depot is the base station infrastructure for the Muni robot fleet. It provides:

- **Console** for unified fleet operations, teleop, and infrastructure monitoring
- **Dispatch** for mission planning, zone management, and task scheduling
- **Real-time metrics** via InfluxDB (battery, motors, GPS, mode)
- **Fleet dashboards** via Grafana (fleet overview + per-rover detail)
- **Session storage** via SFTP (rovers upload recordings)
- **Map processing** via Mapper service (orchestrates splatting jobs)
- **Gaussian splatting** via Splat Worker (GPU-accelerated 3D reconstruction)
- **Map serving** via Map API (browse and download 3D maps)
- **RTK corrections** via NTRIP caster (centimeter-accurate GPS)
- **Automatic retention** (30-day cleanup by default)

## Architecture

```
Rovers                          Depot
┌─────────┐                     ┌─────────────────────────────────────┐
│ rover-01│──HTTP register────▶│  Discovery (:4860)                  │
│ rover-02│──UDP metrics──────▶│  InfluxDB (:8086, :8089)            │
│ rover-0N│──rclone SFTP──────▶│  SFTP (:2222)                       │
│         │◀─WebSocket─────────│  Console (:80) ◀─── Operators       │
│         │◀─WebSocket tasks───│  Dispatch (:4890)                   │
│         │◀─RTCM corrections──│  NTRIP (:2101) [optional]           │
└─────────┘                     │  Grafana (:3000)                    │
                                │  PostgreSQL (:5432) [internal]      │
                                └─────────────────────────────────────┘
```

## Console

The Console is a unified React application that provides:

- **Dashboard**: Fleet overview, service health, alerts
- **Base Station**: GPS module status, RTK survey progress, NTRIP clients
- **Services**: Infrastructure health monitoring
- **Fleet**: Rover list, status, and quick access to teleop
- **Teleop**: 3D visualization, video feed, gamepad control
- **Dispatch**: Zone management, mission planning, task monitoring
- **Sessions**: Recorded telemetry browser and playback
- **Maps**: 3D Gaussian splat viewer

The Console replaces the previous separate Portal and Operator applications.

## Quick Start

### Prerequisites

- Docker and Docker Compose
- 100+ GB storage for session files
- Network accessible from rovers

### Development (Quick)

```bash
cd depot
docker compose up -d
```

This starts all services with default (insecure) credentials.

### Production Setup

For production deployments, use the provisioning script:

```bash
./scripts/provision.sh lakewood  # or your depot name
docker compose up -d
```

This generates secure, unique credentials for all services.

See **[SETUP.md](SETUP.md)** for the complete setup guide including:

- Credential management
- Tailscale configuration
- Rover fleet onboarding
- Customer depot handoff

### Access

The Console at http://localhost provides access to all functionality:

| Path                | Description           |
| ------------------- | --------------------- |
| `/`                 | Dashboard (overview)  |
| `/base-station`     | GPS/RTK status        |
| `/services`         | Infrastructure health |
| `/fleet`            | Rover list            |
| `/fleet/:id`        | Rover detail          |
| `/fleet/:id/teleop` | Teleop interface      |
| `/dispatch`         | Mission planning      |
| `/sessions`         | Session browser       |
| `/maps`             | 3D map viewer         |

External services (direct access):

| Service   | URL                   | Default Credentials                   |
| --------- | --------------------- | ------------------------------------- |
| Console   | http://localhost      | See [Authentication](#authentication) |
| Grafana   | http://localhost:3000 | admin / munipassword                  |
| InfluxDB  | http://localhost:8086 | admin / munipassword                  |
| SFTP      | localhost:2222        | bvr / SSH key auth                    |
| Discovery | http://localhost:4860 | None (internal)                       |
| Dispatch  | ws://localhost:4890   | None (internal)                       |
| Map API   | http://localhost:4870 | None (internal)                       |

### GPU Support (for Gaussian Splatting)

To enable the splat-worker for GPU-accelerated 3D reconstruction:

```bash
# Requires NVIDIA GPU and nvidia-docker2
docker compose --profile gpu up -d
```

### RTK Base Station

To enable RTK corrections (requires ZED-F9P connected via USB):

```bash
# Start with RTK profile
docker compose --profile rtk up -d
```

See [RTK documentation](../docs/hardware/rtk.md) for hardware setup.

## Dispatch (Mission Planning)

The Dispatch service enables automated mission planning and task execution:

### Concepts

- **Zones**: Geographic areas defined by waypoints (routes) or GPS polygons
- **Missions**: Scheduled work assignments linking zones to rovers
- **Tasks**: Individual execution instances of missions

### Zone Types

**Waypoint Routes**: Ordered list of coordinates the rover follows sequentially
```json
{
  "name": "Sidewalk A",
  "zone_type": "waypoints",
  "waypoints": [
    {"x": 0.0, "y": 0.0},
    {"x": 10.0, "y": 0.0},
    {"x": 10.0, "y": 5.0}
  ],
  "is_loop": true
}
```

**GPS Polygons**: Area boundaries for coverage planning (future: auto-generate waypoints)
```json
{
  "name": "Parking Lot B",
  "zone_type": "polygon",
  "polygon": [
    {"lat": 41.481956, "lon": -81.8053},
    {"lat": 41.482100, "lon": -81.8053},
    {"lat": 41.482100, "lon": -81.8050}
  ]
}
```

### Task Lifecycle

```
pending -> assigned -> active -> completed
                    \         \-> failed
                     \-> cancelled
```

1. **Pending**: Task created, waiting for rover availability
2. **Assigned**: Sent to rover via WebSocket
3. **Active**: Rover executing waypoints, reporting progress
4. **Completed**: All waypoints visited successfully
5. **Failed**: Error during execution (rover reports reason)
6. **Cancelled**: Operator cancelled via console

### API Endpoints

| Method | Path | Description |
| ------ | ---- | ----------- |
| GET | `/api/zones` | List all zones |
| POST | `/api/zones` | Create zone |
| GET | `/api/zones/:id` | Get zone details |
| PUT | `/api/zones/:id` | Update zone |
| DELETE | `/api/zones/:id` | Delete zone |
| GET | `/api/missions` | List all missions |
| POST | `/api/missions` | Create mission |
| GET | `/api/missions/:id` | Get mission details |
| DELETE | `/api/missions/:id` | Delete mission |
| GET | `/api/tasks` | List tasks (filterable) |
| POST | `/api/tasks/:id/cancel` | Cancel a task |

### WebSocket Protocol

**Rover connection** (`/ws`):
```json
// Rover -> Dispatch: Register
{"type": "register", "rover_id": "bvr-01"}

// Dispatch -> Rover: Task assignment
{"type": "task", "task_id": "uuid", "mission_id": "uuid", "zone": {...}}

// Rover -> Dispatch: Progress update
{"type": "progress", "task_id": "uuid", "progress": 75, "waypoint": 3, "lap": 0}

// Rover -> Dispatch: Complete
{"type": "complete", "task_id": "uuid", "laps": 1}

// Dispatch -> Rover: Cancel
{"type": "cancel", "task_id": "uuid"}
```

**Console connection** (`/ws/console`): Receives real-time task state updates.

### Rover Configuration

Enable dispatch in `bvr.toml`:

```toml
[dispatch]
enabled = true
endpoint = "ws://depot.local:4890/ws"
```

Or via CLI:
```bash
bvrd --dispatch-endpoint ws://192.168.1.100:4890/ws
```

Disable with `--no-dispatch` flag.

## Authentication

The Console supports two authentication methods:

### Password Authentication

Set `CONSOLE_PASSWORD` to enable password protection:

```bash
# Via environment variable
CONSOLE_PASSWORD=your-secure-password docker compose up -d

# Or in .env file
echo "CONSOLE_PASSWORD=your-secure-password" >> .env
docker compose up -d
```

Default username is `admin`. Override with `CONSOLE_USERNAME`:

```bash
CONSOLE_USERNAME=operator CONSOLE_PASSWORD=secret docker compose up -d
```

### Tailscale Authentication

When accessing the Console through [Tailscale Serve](https://tailscale.com/kb/1312/serve) or [Tailscale Funnel](https://tailscale.com/kb/1223/funnel), authentication is automatic using your Tailscale identity.

```bash
# Expose Console via Tailscale (HTTPS, authenticated)
tailscale serve https / http://localhost:80

# Or expose publicly via Funnel (still requires Tailscale login)
tailscale funnel https / http://localhost:80
```

The Console automatically detects Tailscale headers and bypasses password auth when Tailscale identity is present.

### No Authentication (Development)

By default, if `CONSOLE_PASSWORD` is not set, authentication is disabled:

```bash
# No auth - for local development only
docker compose up -d
```

⚠️ **Warning**: Do not expose the Console to the internet without authentication enabled.

## Development

For developing the Console with hot-reload:

```bash
# Terminal 1: Start backend services
docker compose up -d discovery influxdb grafana

# Terminal 2: Run console with hot-reload
cd console
npm install
npm run dev
```

The console dev server runs on http://localhost:5173 with:

- Hot module replacement (instant updates)
- Auto-refresh on file changes
- Source maps for debugging
- Proxy to backend services

### Directory Structure

```
depot/
├── console/          # React web application (Console)
├── discovery/        # Rover registration service (Rust)
├── dispatch/         # Mission planning & task dispatch (Rust)
├── map-api/          # Map serving API (Rust)
├── mapper/           # Map processing orchestrator (Rust)
├── splat-worker/     # GPU splatting worker (Python)
├── grafana/          # Grafana provisioning
├── sftp/             # SFTP server config
└── scripts/          # Maintenance scripts
```

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

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
# Console Authentication
CONSOLE_PASSWORD=<secure-password>
CONSOLE_USERNAME=admin

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

# RTK (optional)
NTRIP_PASSWORD=<secure-password>
```

### Adding Rover SSH Keys

Rovers authenticate to SFTP using SSH keys. Add each rover's public key:

```bash
# Copy rover's public key
cp /path/to/bvr-01.pub sftp/authorized_keys/

# Restart SFTP to pick up new keys
docker compose restart sftp
```

## Rover Configuration

Configuration varies by morphology. See the morphology-specific docs:

- **BVR**: See [bvr/docs/](../bvr/docs/) for BVR-specific configuration

### Metrics Push

Rovers push metrics via UDP to InfluxDB:

```toml
[metrics]
enabled = true
endpoint = "depot.local:8089"
interval_hz = 1
```

### Session Sync

Configure rclone on each rover for session upload:

```ini
[base]
type = sftp
host = depot.local
port = 2222
user = bvr
key_file = /etc/muni/id_ed25519
```

## Maintenance

### Session Cleanup

Sessions older than `RETENTION_DAYS` are automatically cleaned up. Run manually:

```bash
./scripts/cleanup.sh
```

### Viewing Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f console
docker compose logs -f discovery
docker compose logs -f influxdb
```

## Network Ports

| Port | Protocol | Service    | Purpose                   |
| ---- | -------- | ---------- | ------------------------- |
| 80   | TCP      | Console    | Web interface             |
| 2101 | TCP      | NTRIP      | RTK corrections broadcast |
| 2222 | TCP      | SFTP       | Session file uploads      |
| 3000 | TCP      | Grafana    | Metrics dashboards        |
| 4860 | TCP      | Discovery  | Rover registration        |
| 4870 | TCP      | Map API    | Map serving               |
| 4890 | TCP      | Dispatch   | Mission/task WebSocket    |
| 5432 | TCP      | PostgreSQL | Database (internal)       |
| 8086 | TCP      | InfluxDB   | HTTP API + Web UI         |
| 8089 | UDP      | InfluxDB   | Line protocol metrics     |

## Security Considerations

- **Console**: Supports password auth or Tailscale identity (set `CONSOLE_PASSWORD` for production)
- **SFTP**: Uses key-based authentication only (no passwords)
- **Grafana/InfluxDB**: Use password auth (set strong passwords in .env)
- **TLS**: Use Tailscale Serve/Funnel for automatic HTTPS, or place behind a reverse proxy
- **Network**: Use Tailscale for rovers connecting over public internet (provides auth + encryption)
- **Internal APIs**: Discovery, Map API, and GPS Status are proxied through Console and not exposed directly
