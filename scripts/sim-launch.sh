#!/bin/bash
# Launch sim-bridge and N bvrd instances for multi-rover simulation.
#
# Usage:
#   scripts/sim-launch.sh                          # default scenario
#   scripts/sim-launch.sh path/to/scenario.toml    # custom scenario
#
# Requires:
#   cargo build -p sim-bridge && cargo build --bin bvrd
set -euo pipefail

SCENARIO="${1:-bvr/firmware/config/sim/scenario.toml}"
CONFIG_DIR="bvr/firmware/config/sim"
PIDS=()

cleanup() {
    echo ""
    echo "Shutting down..."
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null
    echo "Done."
}
trap cleanup EXIT INT TERM

echo "=== sim-bridge multi-rover simulation ==="
echo "Scenario: $SCENARIO"
echo ""

# Start sim-bridge
echo "Starting sim-bridge..."
cargo run -p sim-bridge -- --scenario "$SCENARIO" &
PIDS+=($!)
sleep 2

# Start bvrd instances
for config in "$CONFIG_DIR"/frog-*.toml; do
    ROVER=$(basename "$config" .toml)

    # Extract CAN port from scenario (match rover ID to can_port)
    CAN_PORT=$(grep -A1 "id = \"$ROVER\"" "$SCENARIO" | grep can_port | head -1 | sed 's/.*= *//')

    if [ -z "$CAN_PORT" ]; then
        echo "  WARNING: No can_port found for $ROVER in scenario, skipping"
        continue
    fi

    # Extract RTC port offset from teleop port in config
    RTC_PORT=$(grep 'port = ' "$config" | head -1 | sed 's/.*= *//')
    # Use teleop port + 12 as RTC port (4840->4852, 4841->4853, etc)
    RTC_PORT=$((RTC_PORT + 12))

    echo "  Starting $ROVER (CAN port $CAN_PORT, RTC port $RTC_PORT)..."
    cargo run --bin bvrd -- \
        --remote-can "tcp://127.0.0.1:$CAN_PORT" \
        --config "$config" \
        --rtc-port "$RTC_PORT" \
        --no-camera \
        --no-recording \
        --ui-port 0 &
    PIDS+=($!)
    sleep 1
done

echo ""
echo "All rovers started."
echo "  sim-bridge status: http://localhost:4900/status"
echo "  Console: http://localhost (if depot is running)"
echo ""
echo "Press Ctrl+C to stop."
wait
