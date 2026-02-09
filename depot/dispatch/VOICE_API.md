# Voice Command API

REST API for sending high-level commands to rovers from external systems (e.g., speech-to-text servers). The API lives on the dispatch service, which already maintains persistent WebSocket connections to every online rover — so your STT server only needs one URL.

**Base URL:** `http://depot:4890` (or wherever the dispatch service is running)

## Endpoints

### List connected rovers

```
GET /rovers
```

Returns an array of currently connected rovers.

```json
[
  { "id": "frog-0", "connected": true, "currentTask": null },
  { "id": "frog-1", "connected": true, "currentTask": "a1b2c3d4-..." }
]
```

### Send a command

```
POST /rovers/{rover_id}/command
Content-Type: application/json
```

Returns `200 OK` with `{"status": "ok"}` on success. Returns `404` if the rover isn't connected, `400` for invalid payloads, `401` for bad API key, `429` if rate-limited.

#### Emergency stop

Immediately stops all motors. Always accepted regardless of current mode.

```json
{ "type": "estop" }
```

#### Release e-stop

Returns the rover to Idle mode after an e-stop.

```json
{ "type": "estop_release" }
```

#### Navigate to coordinates

Sets a goal in the rover's local frame (meters) and auto-transitions to Autonomous mode. The rover will plan a path and drive there.

```json
{ "type": "set_goal", "x": 5.0, "y": 3.0 }
```

#### Change mode

Switch the rover's operating mode.

```json
{ "type": "set_mode", "mode": "idle" }
```

Valid modes: `idle`, `autonomous`, `sleep`, `dance`

## Authentication

Set the `VOICE_API_KEY` environment variable on the dispatch service to require an API key. When set, all requests must include:

```
X-API-Key: your-secret-key
```

When `VOICE_API_KEY` is not set, the endpoint is open (relies on network-level isolation).

## Rate Limiting

Max 10 commands per second per rover. Exceeding this returns `429 Too Many Requests`. This is plenty for voice-speed commands.

## Command Safety

Only high-level commands are accepted through this API:

| Allowed | Blocked |
|---------|---------|
| `estop` | `twist` (continuous velocity) |
| `estop_release` | `tool` (attachment control) |
| `set_mode` | `heartbeat` |
| `set_goal` | `lidar_toggle` |

Continuous velocity commands (`twist`) are intentionally blocked — voice is not a real-time control source, and sending velocity without a heartbeat stream would trigger the watchdog timeout. Use `set_goal` for navigation instead.

## Example: curl

```bash
# Check who's online
curl http://depot:4890/rovers

# Send a rover to coordinates (5, 3)
curl -X POST http://depot:4890/rovers/frog-0/command \
  -H "Content-Type: application/json" \
  -d '{"type": "set_goal", "x": 5.0, "y": 3.0}'

# Emergency stop
curl -X POST http://depot:4890/rovers/frog-0/command \
  -H "Content-Type: application/json" \
  -d '{"type": "estop"}'

# With API key
curl -X POST http://depot:4890/rovers/frog-0/command \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-secret-key" \
  -d '{"type": "set_mode", "mode": "idle"}'
```

## Example: Python

```python
import requests

DISPATCH = "http://depot:4890"
API_KEY = None  # set if VOICE_API_KEY is configured

def send_command(rover_id: str, command: dict):
    headers = {"Content-Type": "application/json"}
    if API_KEY:
        headers["X-API-Key"] = API_KEY
    r = requests.post(f"{DISPATCH}/rovers/{rover_id}/command", json=command, headers=headers)
    r.raise_for_status()
    return r.json()

# List rovers
rovers = requests.get(f"{DISPATCH}/rovers").json()
print(rovers)

# Navigate
send_command("frog-0", {"type": "set_goal", "x": 5.0, "y": 3.0})

# E-stop
send_command("frog-0", {"type": "estop"})
```

## How It Works

```
Your STT Server ──HTTP POST──> Dispatch Service ──WebSocket──> bvrd (rover)
                                    |                              |
                              validates payload              feeds into cmd_tx
                              checks API key                 (same channel as
                              rate-limits                     teleop gamepad)
                              forwards via WS                     |
                                                            state machine
                                                            rate limiter
                                                            collision guard
                                                               motors
```

Commands enter the exact same pipeline as the teleop gamepad and console click-to-navigate. All safety systems (e-stop, rate limiting, collision avoidance, watchdog) apply.

## Missions

For multi-waypoint missions, use the existing dispatch mission API instead:

```bash
# Start a pre-configured mission
POST /missions/{id}/start

# Stop a running mission
POST /missions/{id}/stop
```

See the dispatch service source for the full mission/zone/task CRUD API.
