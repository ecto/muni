# Depot Setup Guide

Step-by-step instructions for deploying a new depot (base station).

## Prerequisites

- [ ] Server hardware assembled (see [docs/hardware/depot.md](../docs/hardware/depot.md))
- [ ] Linux installed (Raspberry Pi OS, Ubuntu, or similar)
- [ ] Docker and Docker Compose installed
- [ ] Network connectivity (Ethernet to router/switch)
- [ ] RTK antenna mounted with clear sky view (if using RTK)

## Quick Start (5 minutes)

For a quick development setup:

```bash
cd depot
docker compose up -d
```

This starts all services with default (insecure) credentials. For production, follow the full setup below.

---

## Full Setup Checklist

### 1. Clone Repository

```bash
git clone https://github.com/muni-ai/muni.git
cd muni/depot
```

### 2. Generate Credentials

Run the provisioning script to generate unique, secure credentials:

```bash
./scripts/provision.sh
```

This creates a `.env` file with:

- Console password (for web UI access)
- InfluxDB admin password and API token
- Grafana admin password
- NTRIP password (for RTK corrections)

**Review the generated `.env`:**

```bash
cat .env
```

**Save these credentials securely** (password manager, vault, etc.)

### 3. Configure Depot Identity

Edit `.env` to set your depot name:

```bash
# Depot identity (used in hostnames, logs)
DEPOT_NAME=lakewood
```

### 4. Network Configuration

#### Option A: Tailscale (Recommended for Muni-operated depots)

```bash
# Install Tailscale
curl -fsSL https://tailscale.com/install.sh | sh

# Join your tailnet with a descriptive hostname
sudo tailscale up --hostname=depot-lakewood

# Expose Console via Tailscale Serve (automatic HTTPS + auth)
sudo tailscale serve https / http://localhost:80

# Verify
tailscale status
```

With Tailscale Serve:

- Access via `https://depot-lakewood.<tailnet>.ts.net`
- Authentication is automatic (Tailscale identity)
- No password needed for your team

#### Option B: Password-Only (Customer depots or no Tailscale)

Ensure `CONSOLE_PASSWORD` is set in `.env` (the provision script does this).

Access via local network: `http://<depot-ip>/`

#### Option C: Public Access via Tailscale Funnel

For remote access without VPN:

```bash
# Expose publicly (still requires Tailscale login)
sudo tailscale funnel https / http://localhost:80
```

### 5. Start Services

```bash
# Start core services
docker compose up -d

# With RTK base station (requires ZED-F9P connected)
docker compose --profile rtk up -d

# With GPU splatting (requires NVIDIA GPU)
docker compose --profile gpu up -d

# All profiles
docker compose --profile rtk --profile gpu up -d
```

### 6. Verify Services

```bash
# Check all containers are running
docker compose ps

# Check logs for errors
docker compose logs --tail 50

# Test Console
curl -I http://localhost/health

# Test Discovery
curl http://localhost:4860/rovers

# Test InfluxDB
curl http://localhost:8086/health
```

Access the Console at `http://localhost` (or your Tailscale URL).

### 7. Configure Firewall

If using UFW (Ubuntu/Debian):

```bash
# Allow internal services (if not using Tailscale for everything)
sudo ufw allow 80/tcp    # Console
sudo ufw allow 2101/tcp  # NTRIP (RTK corrections)
sudo ufw allow 2222/tcp  # SFTP (rover session uploads)
sudo ufw allow 8089/udp  # InfluxDB metrics (rover telemetry)

# Optional: direct access to dashboards
sudo ufw allow 3000/tcp  # Grafana
sudo ufw allow 8086/tcp  # InfluxDB UI
```

With Tailscale, you can skip most of these (traffic goes through the tunnel).

---

## Rover Fleet Onboarding

### 1. Generate Fleet SSH Key

Each depot needs SSH keys for rover SFTP uploads:

```bash
# Generate a key for this depot's fleet
ssh-keygen -t ed25519 -f sftp/ssh_host_keys/fleet-key -N "" -C "depot-lakewood-fleet"

# Copy public key to authorized_keys
cp sftp/ssh_host_keys/fleet-key.pub sftp/authorized_keys/

# Restart SFTP to pick up new keys
docker compose restart sftp
```

### 2. Configure Each Rover

On each rover, configure rclone for session sync:

```bash
# /etc/muni/rclone.conf
[depot]
type = sftp
host = depot-lakewood  # or IP address
port = 2222
user = bvr
key_file = /etc/muni/fleet-key
```

Copy the private key (`fleet-key`) to each rover at `/etc/muni/fleet-key`.

### 3. Configure Rover Metrics

In each rover's `bvr.toml`:

```toml
[metrics]
enabled = true
endpoint = "depot-lakewood:8089"  # InfluxDB UDP
interval_hz = 1

[discovery]
enabled = true
endpoint = "http://depot-lakewood:4860"
```

### 4. Configure RTK (Optional)

If using RTK corrections:

```toml
[gps]
ntrip_host = "depot-lakewood"
ntrip_port = 2101
ntrip_mountpoint = "RTCM3"
ntrip_password = "<from .env>"
```

---

## RTK Base Station Setup

If using the RTK profile:

### 1. Connect GPS Module

Connect the ZED-F9P via USB. Verify it appears:

```bash
ls -la /dev/ttyACM*  # Linux
ls -la /dev/cu.usbmodem*  # macOS
```

### 2. Configure Device Path

Edit `docker-compose.yml` if your device path differs:

```yaml
gps-status:
  devices:
    - /dev/ttyACM0:/dev/ttyACM0 # Adjust as needed
```

### 3. Start RTK Services

```bash
docker compose --profile rtk up -d
```

### 4. Verify GPS Status

Check the Console at `/base-station` or:

```bash
curl http://localhost:4880/status
```

### 5. Survey-In

The base station needs to determine its precise position. This happens automatically:

- Takes 5-15 minutes with good sky view
- Console shows survey progress
- Once complete, RTCM corrections are broadcast

---

## Customer Depot Differences

For depots operated by customers (not Muni):

### Credentials

- Generate unique credentials (run `provision.sh`)
- Store customer credentials in your support system
- Consider shorter/simpler passwords if customer will type them

### Access Model

| Access Level         | Who                 | How               |
| -------------------- | ------------------- | ----------------- |
| Console (teleop)     | Customer operators  | Password auth     |
| Grafana (dashboards) | Customer (optional) | Separate password |
| SSH (support)        | Muni support        | Tailscale or VPN  |
| InfluxDB API         | Muni analytics      | API token         |

### Remote Support Options

1. **Tailscale Subnet Router**: Add depot to your tailnet for full access
2. **SSH Tunnel**: Customer opens reverse tunnel for support sessions
3. **Tailscale Funnel**: Expose specific services publicly (with auth)

### Handoff Checklist

- [ ] Hardware installed and powered
- [ ] Services running (`docker compose ps`)
- [ ] Customer can access Console
- [ ] At least one rover registered and visible
- [ ] Customer trained on basic operation
- [ ] Support contact information provided
- [ ] Credentials stored in support system

---

## Maintenance

### Updates

```bash
cd ~/muni
git pull
cd depot
docker compose build
docker compose up -d
```

### Backup Credentials

The `.env` file contains all secrets. Back it up securely:

```bash
# Encrypt and store
gpg -c .env  # Creates .env.gpg
# Store .env.gpg in secure location
```

### Log Rotation

Docker handles log rotation. Check disk usage:

```bash
docker system df
```

### Session Cleanup

Sessions older than `RETENTION_DAYS` (default 30) are auto-cleaned.
Manual cleanup:

```bash
./scripts/cleanup.sh
```

---

## Troubleshooting

### Console Not Loading

```bash
# Check container status
docker compose ps console

# Check logs
docker compose logs console

# Verify nginx config
docker compose exec console nginx -t
```

### Rovers Not Appearing

```bash
# Check discovery service
docker compose logs discovery

# Test registration manually
curl -X POST http://localhost:4860/register \
  -H "Content-Type: application/json" \
  -d '{"id":"test","name":"Test","address":"ws://localhost:4850"}'
```

### GPS Not Connecting

```bash
# Check if device exists
ls -la /dev/ttyACM*

# Check gps-status logs
docker compose --profile rtk logs gps-status

# Test serial connection
screen /dev/ttyACM0 115200
```

### SFTP Connection Refused

```bash
# Check SFTP container
docker compose logs sftp

# Test connection
sftp -P 2222 -i path/to/key bvr@localhost
```

---

## Quick Reference

| Service       | URL                       | Default Port |
| ------------- | ------------------------- | ------------ |
| Console       | http://localhost/         | 80           |
| Grafana       | http://localhost/grafana/ | (proxied)    |
| InfluxDB      | http://localhost:8086     | 8086         |
| Discovery API | http://localhost:4860     | 4860         |
| SFTP          | sftp://localhost:2222     | 2222         |
| NTRIP         | ntrip://localhost:2101    | 2101         |

| File                    | Purpose                    |
| ----------------------- | -------------------------- |
| `.env`                  | All credentials and config |
| `sftp/authorized_keys/` | Rover SSH public keys      |
| `sftp/ssh_host_keys/`   | SFTP server keys           |
| `grafana/dashboards/`   | Dashboard definitions      |
