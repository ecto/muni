#!/bin/bash
#
# bvr-sync.sh - Sync BVR session recordings to base station
#
# This script syncs .rrd session files and rotated logs to the configured
# base station using rclone. It's designed to be run by systemd timer.
#
# Environment variables:
#   BVR_CONFIG     - Path to bvr.toml (default: /etc/bvr/bvr.toml)
#   RCLONE_CONFIG  - Path to rclone.conf (default: /etc/bvr/rclone.conf)

set -euo pipefail

# Configuration
BVR_CONFIG="${BVR_CONFIG:-/etc/bvr/bvr.toml}"
RCLONE_CONFIG="${RCLONE_CONFIG:-/etc/bvr/rclone.conf}"
SESSION_DIR="${SESSION_DIR:-/var/log/bvr/sessions}"
LOG_DIR="${LOG_DIR:-/var/log/bvr}"
MIN_AGE="${MIN_AGE:-1m}"  # Don't sync files newer than this

# Read rover ID from config (fallback to hostname)
ROVER_ID=$(grep -oP 'rover_id\s*=\s*"\K[^"]+' "$BVR_CONFIG" 2>/dev/null || hostname)

# Read destination from config
DESTINATION=$(grep -oP 'destination\s*=\s*"\K[^"]+' "$BVR_CONFIG" 2>/dev/null || echo "")

if [ -z "$DESTINATION" ]; then
    echo "Error: No sync destination configured in $BVR_CONFIG"
    exit 1
fi

# Check if rclone is available
if ! command -v rclone &> /dev/null; then
    echo "Error: rclone not found. Install with: apt install rclone"
    exit 1
fi

# Check if rclone config exists
if [ ! -f "$RCLONE_CONFIG" ]; then
    echo "Warning: rclone config not found at $RCLONE_CONFIG"
    echo "Run 'rclone config' to set up the base station remote"
    exit 1
fi

echo "$(date -Iseconds) Starting BVR sync for $ROVER_ID"

# Sync session recordings (full session directories with telemetry, LiDAR, camera)
if [ -d "$SESSION_DIR" ]; then
    echo "Syncing sessions from $SESSION_DIR to $DESTINATION/$ROVER_ID/sessions/"
    rclone sync "$SESSION_DIR" "$DESTINATION/$ROVER_ID/sessions/" \
        --config "$RCLONE_CONFIG" \
        --min-age "$MIN_AGE" \
        --include "*.rrd" \
        --include "*.pcd" \
        --include "*.jpg" \
        --include "*.csv" \
        --include "metadata.json" \
        --transfers 4 \
        --checkers 4 \
        --retries 3 \
        --low-level-retries 10 \
        --stats-one-line \
        -v || echo "Warning: Session sync failed (will retry later)"
else
    echo "Session directory $SESSION_DIR does not exist"
fi

# Sync rotated log files (not the active bvrd.log)
if [ -d "$LOG_DIR" ]; then
    echo "Syncing logs from $LOG_DIR to $DESTINATION/$ROVER_ID/logs/"
    rclone sync "$LOG_DIR" "$DESTINATION/$ROVER_ID/logs/" \
        --config "$RCLONE_CONFIG" \
        --min-age "$MIN_AGE" \
        --include "*.log.*" \
        --exclude "bvrd.log" \
        --transfers 2 \
        --checkers 2 \
        --retries 3 \
        --stats-one-line \
        -v || echo "Warning: Log sync failed (will retry later)"
fi

echo "$(date -Iseconds) BVR sync complete"

