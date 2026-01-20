# Teleop Debugging Summary

**Date:** 2025-01-19/20
**Rover:** frog-0 (Jetson Orin NX)
**Console:** depot/console React app

---

## Original Problem

When using the depot console to teleoperate frog-0, controls are **extremely jerky**. Holding forward causes the rover to start/stop erratically. The motors work (verified via VESC Tool), but commands arrive in bursts with large gaps (700ms+), causing the watchdog to timeout repeatedly.

---

## Root Causes Identified & Fixed

### 1. Camera/GStreamer Spinloop (FIXED)
**Symptom:** Jetson freezes completely within seconds of starting bvrd with camera enabled.
**Root Cause:** `nvarguscamerasrc` GStreamer pipeline causes 100% CPU spinloop on the Jetson.
**Fix:** Run with `--no-camera` flag until camera pipeline is debugged separately.
**Status:** ✅ Workaround in place

### 2. CAN Bus Driver Overwhelm (FIXED)
**Symptom:** When VESCs are powered off or CAN bus has errors, bvrd would spam the SocketCAN driver at 400 frames/sec, eventually causing system instability.
**Root Cause:** No backoff when CAN errors occur.
**Fix:** Added CAN bus health monitoring with exponential backoff in `bvrd/src/main.rs`:
- Track consecutive CAN errors
- After 10 consecutive errors, back off to 1 retry per second
- Log state transitions (healthy → unhealthy → recovered)
**File:** `bvr/firmware/bins/bvrd/src/main.rs` (lines ~1000-1100)
**Status:** ✅ Fixed

### 3. CAN Backoff Log Spam (FIXED)
**Symptom:** "CAN bus unhealthy" warning logged every 16ms instead of once.
**Root Cause:** Check used `>=` threshold which stayed true; needed a flag to log once.
**Fix:** Added `can_backoff_active` flag to ensure warning logs only once.
**File:** `bvr/firmware/bins/bvrd/src/main.rs`
**Status:** ✅ Fixed

### 4. Keyboard Input Tracking Bug (FIXED)
**Symptom:** Keys appeared to "release" while still held, causing jerky movement.
**Root Cause:** Module-level and useEffect both registered the same `handleKeyUp` function. When useEffect cleanup ran, it removed the shared function entirely.
**Fix:** Separated into `globalKeyUp` (module-level, never removed) and component-level handler.
**File:** `depot/console/src/hooks/useKeyboard.ts`
**Status:** ✅ Fixed

### 5. Discovery Address Issue (FIXED)
**Symptom:** Console couldn't connect to frog-0 via discovery service.
**Root Cause:** bvrd was registering with local network IP (`10.104.17.0`) instead of Tailscale-resolvable hostname.
**Fix:** Added `advertise_host` config option to discovery client. Set to `"frog-0"` in config.
**Files:**
- `bvr/firmware/crates/metrics/src/discovery.rs` - Added `advertise_host: Option<String>` field
- `bvr/firmware/bins/bvrd/src/main.rs` - Wire up config
- `/etc/bvr/bvr.toml` on frog-0 - Set `advertise_host = "frog-0"`
**Status:** ✅ Fixed

### 6. Browser Main Thread Blocking (FIXED)
**Symptom:** Commands sent in bursts with 100-600ms gaps despite 33ms setInterval.
**Root Cause:** JavaScript `setInterval` runs on the main thread, which gets blocked by React re-renders, garbage collection, and other UI work.
**Diagnosis:** Added timestamp to Twist commands; measured 26ms network latency but 400ms+ gaps between sends.
**Fix:** Moved command sending to a **Web Worker** which runs on a separate thread.
**Files:**
- `depot/console/src/workers/commandWorker.ts` - New Web Worker
- `depot/console/src/hooks/useRoverConnection.ts` - Updated to use worker
- `depot/console/src/lib/protocol.ts` - Added timestamp to Twist command (26 bytes now)
- `bvr/firmware/crates/teleop/src/ws.rs` - Added timestamp parsing and gap logging
**Status:** ✅ Fixed (gap_ms now 19-40ms when working correctly)

---

## Remaining Issues

### 1. Multiple WebSocket Connections
**Symptom:** Logs show 3-4 different `cmd_count` values at the same timestamp, indicating multiple clients.
**Evidence:**
```
WebSocket client connected addr=100.108.233.98:53069
WebSocket client connected addr=10.104.12.233:41326
WebSocket client connected addr=10.104.12.233:41328
WebSocket client connected addr=100.108.233.98:53344
```
**Likely Causes:**
- Multiple browser tabs open to console
- Multiple devices (phone, laptop) connected
- Console creating duplicate connections on re-renders
- React StrictMode double-mounting (dev only)

**Impact:** Commands from multiple clients compete, causing jerky behavior even with correct timing.

**Potential Fix:**
- Add connection management to only allow one active teleop session per rover
- Or ensure console properly closes old connections before creating new ones

### 2. Occasional Large Gaps (1-3 seconds)
**Symptom:** Even with Web Worker, occasional gaps of 1000-3000ms appear.
**Likely Causes:**
- Network issues (Tailscale tunnel, WiFi)
- TCP buffering/congestion
- Browser tab throttling when not focused
- Multiple connections interfering

### 3. Clock Skew (Cosmetic)
**Symptom:** `latency_ms` shows 60-90 seconds instead of realistic values.
**Cause:** Browser clock differs from frog-0 clock by ~60-90 seconds.
**Impact:** None functional; just confuses latency metrics.
**Fix:** Could use NTP sync or relative timing instead of absolute timestamps.

### 4. Jetson Freeze After ~13 Minutes (UNRESOLVED)
**Symptom:** frog-0 freezes after running bvrd for extended periods, even with `--no-camera`.
**Likely Causes:**
- SLAM processing (enabled, using LiDAR)
- Rerun recording memory accumulation
- Tokio runtime issue
- Memory leak somewhere

**Not yet investigated.**

---

## Current Configuration

### bvrd on frog-0
```bash
sudo /usr/local/bin/bvrd --no-camera --no-recording --config /etc/bvr/bvr.toml
```

### Key Config (`/etc/bvr/bvr.toml`)
```toml
[discovery]
enabled = true
endpoint = "depot:4860"
rover_id = "frog-0"
rover_name = "Frog-0"
ws_port = 4850
ws_video_port = 4851
heartbeat_secs = 2
advertise_host = "frog-0"  # Use Tailscale hostname
```

### Console
- Dev server: `npm run dev` in `depot/console`
- Commands sent via Web Worker at 30Hz
- Input state polled at 60Hz from main thread

---

## Files Modified

| File | Changes |
|------|---------|
| `bvr/firmware/bins/bvrd/src/main.rs` | CAN backoff, discovery config, log filter for teleop |
| `bvr/firmware/crates/metrics/src/discovery.rs` | Added `advertise_host` option |
| `bvr/firmware/crates/teleop/src/ws.rs` | Added timestamp parsing, gap/latency logging |
| `depot/console/src/hooks/useRoverConnection.ts` | Refactored to use Web Worker |
| `depot/console/src/hooks/useKeyboard.ts` | Fixed key tracking bug |
| `depot/console/src/workers/commandWorker.ts` | New file - Web Worker for commands |
| `depot/console/src/lib/protocol.ts` | Added timestamp to Twist command |

---

## Diagnostic Commands

### Check bvrd logs for timing issues:
```bash
ssh frog-0 "grep -E '(gap|latency|client connected)' /tmp/bvrd.log | tail -50"
```

### Check for multiple connections:
```bash
ssh frog-0 "grep 'client connected' /tmp/bvrd.log"
```

### Monitor real-time:
```bash
ssh frog-0 "tail -f /tmp/bvrd.log" | grep -E '(gap|latency)'
```

### Check discovery service:
```bash
curl -s http://depot:4860/rovers | jq
```

---

## Next Steps

1. **Fix multiple connections issue** - Either at console level (ensure single connection) or bvrd level (reject/replace old connections)

2. **Investigate 13-minute freeze** - Run with `RUST_BACKTRACE=1`, check memory usage over time, try disabling SLAM

3. **Fix camera pipeline** - Debug GStreamer/nvarguscamerasrc spinloop separately

4. **Consider UDP instead of WebSocket** - UDP has no TCP buffering/head-of-line blocking issues, better for real-time control
