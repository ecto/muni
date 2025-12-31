#!/bin/bash
#
# Depot Provisioning Script
# Generates secure credentials for a new depot installation.
#
# Usage:
#   ./scripts/provision.sh [depot-name]
#
# Example:
#   ./scripts/provision.sh lakewood
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPOT_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$DEPOT_DIR/.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Generate a secure random string
generate_password() {
    local length=${1:-24}
    openssl rand -base64 48 | tr -dc 'a-zA-Z0-9' | head -c "$length"
}

generate_token() {
    openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 48
}

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                    Muni Depot Provisioning                     ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Get depot name
DEPOT_NAME="${1:-}"
if [ -z "$DEPOT_NAME" ]; then
    read -p "Enter depot name (e.g., lakewood, minneapolis): " DEPOT_NAME
fi

if [ -z "$DEPOT_NAME" ]; then
    error "Depot name is required"
fi

# Validate depot name (lowercase, alphanumeric, hyphens)
if ! [[ "$DEPOT_NAME" =~ ^[a-z0-9-]+$ ]]; then
    error "Depot name must be lowercase alphanumeric with hyphens only"
fi

info "Provisioning depot: $DEPOT_NAME"
echo ""

# Check if .env already exists
if [ -f "$ENV_FILE" ]; then
    warn ".env file already exists at $ENV_FILE"
    read -p "Overwrite? (y/N): " OVERWRITE
    if [ "$OVERWRITE" != "y" ] && [ "$OVERWRITE" != "Y" ]; then
        info "Keeping existing .env file"
        exit 0
    fi
    # Backup existing
    cp "$ENV_FILE" "$ENV_FILE.backup.$(date +%Y%m%d%H%M%S)"
    success "Backed up existing .env"
fi

# Generate credentials
info "Generating credentials..."

CONSOLE_PASSWORD=$(generate_password 16)
INFLUXDB_PASSWORD=$(generate_password 24)
INFLUXDB_TOKEN=$(generate_token)
GRAFANA_PASSWORD=$(generate_password 16)
NTRIP_PASSWORD=$(generate_password 12)

# Create .env file
cat > "$ENV_FILE" << EOF
# ============================================================================
# Muni Depot Configuration
# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Depot: $DEPOT_NAME
# ============================================================================

# Depot Identity
DEPOT_NAME=$DEPOT_NAME

# ----------------------------------------------------------------------------
# Console (Web UI)
# ----------------------------------------------------------------------------
# Password for web UI access (username: admin)
# Leave empty to disable password auth (use Tailscale instead)
CONSOLE_PASSWORD=$CONSOLE_PASSWORD
CONSOLE_USERNAME=admin

# ----------------------------------------------------------------------------
# InfluxDB (Metrics Database)
# ----------------------------------------------------------------------------
INFLUXDB_ADMIN_USER=admin
INFLUXDB_ADMIN_PASSWORD=$INFLUXDB_PASSWORD
INFLUXDB_ORG=muni
INFLUXDB_BUCKET=muni
INFLUXDB_ADMIN_TOKEN=$INFLUXDB_TOKEN

# ----------------------------------------------------------------------------
# Grafana (Dashboards)
# ----------------------------------------------------------------------------
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=$GRAFANA_PASSWORD

# ----------------------------------------------------------------------------
# Storage
# ----------------------------------------------------------------------------
# Where to store session recordings (adjust for your system)
SESSIONS_PATH=/data/muni-sessions
# How long to keep sessions (days)
RETENTION_DAYS=30

# ----------------------------------------------------------------------------
# RTK Base Station (Optional)
# ----------------------------------------------------------------------------
# Password for NTRIP caster (rovers use this to get corrections)
NTRIP_PASSWORD=$NTRIP_PASSWORD
# Serial port for GPS module (adjust for your system)
GPS_SERIAL_PORT=/dev/ttyACM0
GPS_BAUD_RATE=115200
EOF

success "Created $ENV_FILE"
echo ""

# Create sessions directory
SESSIONS_DIR="/data/muni-sessions"
if [ ! -d "$SESSIONS_DIR" ] && [ -w "$(dirname "$SESSIONS_DIR")" ]; then
    mkdir -p "$SESSIONS_DIR"
    success "Created sessions directory: $SESSIONS_DIR"
elif [ ! -d "$SESSIONS_DIR" ]; then
    warn "Cannot create $SESSIONS_DIR (need sudo or adjust SESSIONS_PATH in .env)"
fi

# Generate SSH host keys for SFTP if they don't exist
SSH_KEYS_DIR="$DEPOT_DIR/sftp/ssh_host_keys"
if [ ! -f "$SSH_KEYS_DIR/ssh_host_ed25519_key" ]; then
    info "Generating SFTP host keys..."
    mkdir -p "$SSH_KEYS_DIR"
    ssh-keygen -t ed25519 -f "$SSH_KEYS_DIR/ssh_host_ed25519_key" -N "" -C "depot-$DEPOT_NAME" >/dev/null 2>&1
    ssh-keygen -t rsa -b 4096 -f "$SSH_KEYS_DIR/ssh_host_rsa_key" -N "" -C "depot-$DEPOT_NAME" >/dev/null 2>&1
    success "Generated SFTP host keys"
fi

# Create authorized_keys directory
AUTH_KEYS_DIR="$DEPOT_DIR/sftp/authorized_keys"
mkdir -p "$AUTH_KEYS_DIR"

# Summary
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                      Provisioning Complete                     ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "Depot Name:        $DEPOT_NAME"
echo ""
echo "┌─────────────────────────────────────────────────────────────────┐"
echo "│ SAVE THESE CREDENTIALS SECURELY                                 │"
echo "├─────────────────────────────────────────────────────────────────┤"
printf "│ %-20s %-42s │\n" "Console:" "admin / $CONSOLE_PASSWORD"
printf "│ %-20s %-42s │\n" "Grafana:" "admin / $GRAFANA_PASSWORD"
printf "│ %-20s %-42s │\n" "InfluxDB:" "admin / $INFLUXDB_PASSWORD"
printf "│ %-20s %-42s │\n" "NTRIP:" "$NTRIP_PASSWORD"
echo "├─────────────────────────────────────────────────────────────────┤"
echo "│ InfluxDB Token (for API access):                                │"
echo "│ $INFLUXDB_TOKEN │"
echo "└─────────────────────────────────────────────────────────────────┘"
echo ""
echo "Configuration saved to: $ENV_FILE"
echo ""
echo "Next steps:"
echo "  1. Review and adjust .env as needed"
echo "  2. Start services: docker compose up -d"
echo "  3. Access Console: http://localhost"
echo "  4. Set up Tailscale (optional): tailscale serve https / http://localhost:80"
echo ""
echo "For rover setup, see: depot/SETUP.md#rover-fleet-onboarding"
echo ""
