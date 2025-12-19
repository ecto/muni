#!/bin/bash
#
# cleanup.sh - Remove old BVR session files and logs
#
# This script deletes .rrd session files and logs older than RETENTION_DAYS.
# Designed to run as a daily cron job.
#
# Usage:
#   ./cleanup.sh                  # Uses default 30 days
#   RETENTION_DAYS=7 ./cleanup.sh # Override retention
#
# Cron example (run daily at 3am):
#   0 3 * * * /opt/depot/scripts/cleanup.sh >> /var/log/depot-cleanup.log 2>&1

set -euo pipefail

# Load environment from .env if available
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/../.env"
if [ -f "$ENV_FILE" ]; then
    # shellcheck source=/dev/null
    source "$ENV_FILE"
fi

# Configuration
SESSIONS_PATH="${SESSIONS_PATH:-/data/muni-sessions}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"

echo "$(date -Iseconds) Starting cleanup (retention: ${RETENTION_DAYS} days)"

if [ ! -d "$SESSIONS_PATH" ]; then
    echo "Error: Sessions directory not found: $SESSIONS_PATH"
    exit 1
fi

# Count files before cleanup
BEFORE_COUNT=$(find "$SESSIONS_PATH" -type f \( -name "*.rrd" -o -name "*.log.*" \) | wc -l)

# Delete old session files (.rrd)
echo "Cleaning .rrd files older than ${RETENTION_DAYS} days..."
find "$SESSIONS_PATH" -type f -name "*.rrd" -mtime "+${RETENTION_DAYS}" -delete -print | while read -r f; do
    echo "  Deleted: $f"
done

# Delete old log files (.log.*)
echo "Cleaning rotated log files older than ${RETENTION_DAYS} days..."
find "$SESSIONS_PATH" -type f -name "*.log.*" -mtime "+${RETENTION_DAYS}" -delete -print | while read -r f; do
    echo "  Deleted: $f"
done

# Count files after cleanup
AFTER_COUNT=$(find "$SESSIONS_PATH" -type f \( -name "*.rrd" -o -name "*.log.*" \) | wc -l)
DELETED=$((BEFORE_COUNT - AFTER_COUNT))

# Report disk usage
DISK_USAGE=$(du -sh "$SESSIONS_PATH" 2>/dev/null | cut -f1)

echo "$(date -Iseconds) Cleanup complete: deleted $DELETED files, $AFTER_COUNT remaining, using $DISK_USAGE"

