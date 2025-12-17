#!/bin/bash
#
# Deploy bvrd to a rover via Tailscale
#
# Usage:
#   ./deploy.sh <hostname>          # Deploy to rover
#   ./deploy.sh jetson              # Using Tailscale magic DNS
#   ./deploy.sh jetson --restart    # Deploy and restart service
#   ./deploy.sh jetson --config     # Also sync config file
#
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Defaults
TARGET="aarch64-unknown-linux-gnu"
BINARY="bvrd"
REMOTE_USER="${REMOTE_USER:-cam}"
REMOTE_PATH="/usr/local/bin"
CONFIG_PATH="/etc/bvr"
RESTART=false
SYNC_CONFIG=false

usage() {
    echo "Usage: $0 <hostname> [options]"
    echo ""
    echo "Options:"
    echo "  --restart     Restart bvrd service after deploy"
    echo "  --config      Also sync config/bvr.toml"
    echo "  --user USER   SSH user (default: cam, or \$REMOTE_USER)"
    echo "  --cli         Also deploy the cli tool"
    echo "  --help        Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 jetson                    # Deploy to 'jetson' via Tailscale"
    echo "  $0 jetson --restart          # Deploy and restart service"
    echo "  $0 100.82.116.26 --config    # Deploy with config"
    exit 1
}

# Parse arguments
DEPLOY_CLI=false
HOSTNAME=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --restart)
            RESTART=true
            shift
            ;;
        --config)
            SYNC_CONFIG=true
            shift
            ;;
        --cli)
            DEPLOY_CLI=true
            shift
            ;;
        --user)
            REMOTE_USER="$2"
            shift 2
            ;;
        --help|-h)
            usage
            ;;
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
        *)
            if [[ -z "$HOSTNAME" ]]; then
                HOSTNAME="$1"
            else
                echo -e "${RED}Unexpected argument: $1${NC}"
                usage
            fi
            shift
            ;;
    esac
done

if [[ -z "$HOSTNAME" ]]; then
    echo -e "${RED}Error: hostname required${NC}"
    usage
fi

REMOTE="${REMOTE_USER}@${HOSTNAME}"

# Ensure we're in the firmware directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  BVR Deploy${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  Target:  ${GREEN}${HOSTNAME}${NC}"
echo -e "  User:    ${REMOTE_USER}"
echo -e "  Restart: ${RESTART}"
echo -e "  Config:  ${SYNC_CONFIG}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check for cross-compilation toolchain
if ! command -v cross &> /dev/null; then
    echo -e "${YELLOW}Note: 'cross' not found, using cargo directly${NC}"
    echo -e "${YELLOW}      Install cross for easier cross-compilation: cargo install cross${NC}"
    BUILD_CMD="cargo"
else
    BUILD_CMD="cross"
fi

# Build
echo -e "${BLUE}▸ Building for ${TARGET}...${NC}"
$BUILD_CMD build --release --target "$TARGET" --bin "$BINARY"

if [[ "$DEPLOY_CLI" == true ]]; then
    $BUILD_CMD build --release --target "$TARGET" --bin bvr
fi

BINARY_PATH="target/${TARGET}/release/${BINARY}"
CLI_PATH="target/${TARGET}/release/bvr"

if [[ ! -f "$BINARY_PATH" ]]; then
    echo -e "${RED}Build failed: $BINARY_PATH not found${NC}"
    exit 1
fi

# Get binary info
SIZE=$(du -h "$BINARY_PATH" | cut -f1)
echo -e "${GREEN}✓ Built ${BINARY} (${SIZE})${NC}"

# Test SSH connection
echo -e "${BLUE}▸ Checking connection to ${HOSTNAME}...${NC}"
if ! ssh -o ConnectTimeout=5 "$REMOTE" "echo ok" &>/dev/null; then
    echo -e "${RED}Cannot connect to ${REMOTE}${NC}"
    echo -e "${YELLOW}Is Tailscale running? Try: tailscale status${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Connected${NC}"

# Get current version on rover (if bvrd exists and responds to --version)
echo -e "${BLUE}▸ Checking current version...${NC}"
CURRENT_VERSION=$(ssh "$REMOTE" "$REMOTE_PATH/$BINARY --version 2>/dev/null" || echo "not installed")
echo -e "  Current: ${YELLOW}${CURRENT_VERSION}${NC}"

# Upload binary
echo -e "${BLUE}▸ Uploading ${BINARY}...${NC}"
scp -q "$BINARY_PATH" "${REMOTE}:/tmp/${BINARY}.new"
echo -e "${GREEN}✓ Uploaded${NC}"

# Upload CLI if requested
if [[ "$DEPLOY_CLI" == true ]] && [[ -f "$CLI_PATH" ]]; then
    echo -e "${BLUE}▸ Uploading bvr...${NC}"
    scp -q "$CLI_PATH" "${REMOTE}:/tmp/bvr.new"
    echo -e "${GREEN}✓ Uploaded bvr${NC}"
fi

# Upload config if requested
if [[ "$SYNC_CONFIG" == true ]]; then
    echo -e "${BLUE}▸ Uploading config...${NC}"
    scp -q "config/bvr.toml" "${REMOTE}:/tmp/bvr.toml.new"
    echo -e "${GREEN}✓ Uploaded config${NC}"
fi

# Install on remote
echo -e "${BLUE}▸ Installing...${NC}"
ssh "$REMOTE" bash -s <<EOF
set -e

# Stop service if we're restarting
if [[ "$RESTART" == true ]] && systemctl is-active --quiet bvrd 2>/dev/null; then
    echo "  Stopping bvrd..."
    sudo systemctl stop bvrd
fi

# Atomic move of binary
sudo mv /tmp/${BINARY}.new ${REMOTE_PATH}/${BINARY}
sudo chmod +x ${REMOTE_PATH}/${BINARY}

# CLI if present
if [[ -f /tmp/bvr.new ]]; then
    sudo mv /tmp/bvr.new ${REMOTE_PATH}/bvr
    sudo chmod +x ${REMOTE_PATH}/bvr
fi

# Config if present
if [[ -f /tmp/bvr.toml.new ]]; then
    sudo mkdir -p ${CONFIG_PATH}
    sudo mv /tmp/bvr.toml.new ${CONFIG_PATH}/bvr.toml
fi

# Restart service if requested
if [[ "$RESTART" == true ]]; then
    if systemctl list-unit-files | grep -q bvrd; then
        echo "  Starting bvrd..."
        sudo systemctl start bvrd
        sleep 1
        if systemctl is-active --quiet bvrd; then
            echo "  Service running"
        else
            echo "  WARNING: Service failed to start"
            sudo journalctl -u bvrd -n 10 --no-pager
        fi
    else
        echo "  Note: bvrd.service not installed, skipping restart"
    fi
fi
EOF

echo -e "${GREEN}✓ Installed${NC}"

# Verify
echo -e "${BLUE}▸ Verifying...${NC}"
NEW_VERSION=$(ssh "$REMOTE" "$REMOTE_PATH/$BINARY --version 2>/dev/null" || echo "unknown")
echo -e "  Version: ${GREEN}${NEW_VERSION}${NC}"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ✓ Deploy complete${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
