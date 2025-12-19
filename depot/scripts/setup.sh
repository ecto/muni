#!/bin/bash
#
# setup.sh - First-time setup for Depot
#
# This script:
#   1. Creates required directories
#   2. Generates secure tokens if not set
#   3. Validates configuration
#   4. Starts the Docker Compose stack

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPOT_DIR="${SCRIPT_DIR}/.."

cd "$DEPOT_DIR"

echo "=== Depot Setup ==="
echo

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed. Please install Docker first."
    echo "  https://docs.docker.com/engine/install/"
    exit 1
fi

# Check for Docker Compose
if ! docker compose version &> /dev/null; then
    echo "Error: Docker Compose is not available."
    echo "  Please install Docker Compose plugin or update Docker."
    exit 1
fi

# Create .env from example if needed
if [ ! -f .env ]; then
    if [ -f .env.example ]; then
        echo "Creating .env from .env.example..."
        cp .env.example .env

        # Generate secure tokens
        echo "Generating secure tokens..."
        INFLUX_TOKEN=$(openssl rand -hex 32)
        INFLUX_PASS=$(openssl rand -base64 16 | tr -d '=')
        GRAFANA_PASS=$(openssl rand -base64 16 | tr -d '=')

        # Update .env with generated values
        sed -i.bak "s/changeme-generate-secure-token/${INFLUX_TOKEN}/" .env
        sed -i.bak "s/changeme-influxdb-password/${INFLUX_PASS}/" .env
        sed -i.bak "s/changeme-grafana-password/${GRAFANA_PASS}/" .env
        rm -f .env.bak

        echo "Generated credentials (save these!):"
        echo "  InfluxDB password: ${INFLUX_PASS}"
        echo "  Grafana password:  ${GRAFANA_PASS}"
        echo
    else
        echo "Error: No .env or .env.example found."
        exit 1
    fi
else
    echo "Using existing .env file"
fi

# Load environment
# shellcheck source=/dev/null
source .env

# Create sessions directory
SESSIONS_PATH="${SESSIONS_PATH:-/data/bvr-sessions}"
echo "Creating sessions directory: ${SESSIONS_PATH}"
sudo mkdir -p "$SESSIONS_PATH"
sudo chown 1000:1000 "$SESSIONS_PATH"

# Create authorized_keys directory for SFTP
mkdir -p sftp/authorized_keys

# Check if any rover keys are configured
if [ -z "$(ls -A sftp/authorized_keys 2>/dev/null)" ]; then
    echo
    echo "Warning: No rover SSH keys configured in sftp/authorized_keys/"
    echo "  Rovers won't be able to upload sessions until you add their public keys."
    echo "  Example: cp /path/to/rover-key.pub sftp/authorized_keys/bvr-01.pub"
    echo
fi

# Start the stack
echo "Starting Depot services..."
docker compose up -d

# Wait for services to be healthy
echo "Waiting for services to start..."
sleep 5

# Check service status
echo
echo "=== Service Status ==="
docker compose ps

echo
echo "=== Setup Complete ==="
echo
echo "Services:"
echo "  Grafana:   http://localhost:3000 (admin / ${GRAFANA_ADMIN_PASSWORD:-check .env})"
echo "  InfluxDB:  http://localhost:8086"
echo "  SFTP:      localhost:2222 (user: bvr, key auth only)"
echo
echo "Next steps:"
echo "  1. Add rover SSH public keys to sftp/authorized_keys/"
echo "  2. Configure rovers to sync to this depot"
echo "  3. Access Grafana to view fleet dashboards"
echo

