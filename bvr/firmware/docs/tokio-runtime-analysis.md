# Tokio Runtime Analysis for bvrd

**Date:** 2026-01-23
**Author:** Claude (systems analysis)
**Status:** Root cause identified, fixes proposed

## Executive Summary

The bvrd daemon suffers from **runtime starvation** caused by a combination of:
1. **Blocking mutex locks** in an async control loop
2. **Unbounded async channel sends** that block when the receiver is slow
3. **Synchronous I/O** (CAN bus, Rerun recording) on tokio worker threads
4. **Resource contention** between CPU-intensive SLAM and the async runtime

The result is that after initial operation, the tokio runtime becomes "gummed up" — async tasks stop making progress, channels fill up, and the UDP teleop server stops processing commands even though packets arrive at the kernel level.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           bvrd Process                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    Tokio Runtime (multi-threaded)                 │   │
│  │                                                                    │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │   │
│  │   │ Main Loop   │  │ UDP Teleop  │  │ WebRTC      │               │   │
│  │   │ (100Hz)     │  │ Server      │  │ Handler     │               │   │
│  │   │             │  │             │  │             │               │   │
│  │   │ • CAN recv  │  │ • select!   │  │ • Signaling │               │   │
│  │   │ • Mutex     │  │   - recv    │  │ • Channels  │               │   │
│  │   │   locks     │  │   - tick    │  │ • Video     │               │   │
│  │   │ • Recording │  │ • send()    │  │             │               │   │
│  │   │ • Commands  │  │   .await    │  │             │               │   │
│  │   └─────────────┘  └─────────────┘  └─────────────┘               │   │
│  │          ▲                │                │                       │   │
│  │          │                │                │                       │   │
│  │          │     cmd_tx     │     cmd_tx     │                       │   │
│  │          │    (clone)     │    (clone)     │                       │   │
│  │          │                ▼                ▼                       │   │
│  │          │         ┌─────────────────────────┐                    │   │
│  │          └─────────│   Command Channel (32)   │                    │   │
│  │      try_recv()    └─────────────────────────┘                    │   │
│  │                                                                    │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    Dedicated OS Threads                           │   │
│  │                                                                    │   │
│  │   ┌─────────────┐  ┌─────────────┐                                │   │
│  │   │ SLAM Thread │  │ Camera      │                                │   │
│  │   │ (97% CPU!)  │  │ (GStreamer) │                                │   │
│  │   └─────────────┘  └─────────────┘                                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Critical Issues Identified

### Issue 1: Blocking Mutex in Async Context (CRITICAL)

**Location:** `bins/bvrd/src/main.rs` lines 1478, 1490, 1512, 1530, 1576, etc.

**Problem:** The main control loop uses `std::sync::Mutex` with `.lock().unwrap()` extensively:

```rust
// Inside async main() with tokio::time::sleep().await
loop {
    tokio::time::sleep(control_period - elapsed).await;  // Yields to runtime

    // PROBLEM: Blocking lock on tokio worker thread
    let mut state = shared.lock().unwrap();  // Can block indefinitely

    // ... more blocking locks throughout the loop
}
```

**Why this is bad:**
- `std::sync::Mutex::lock()` is a **blocking syscall**
- When called on a tokio worker thread, it **blocks the entire worker**
- Other async tasks waiting on that worker **cannot make progress**
- With 4 default workers and multiple lock acquisitions, workers can all be blocked

**Evidence:** The control loop stops iterating (no battery logs), but the process still runs at 127% CPU.

### Issue 2: Async Channel Send Blocking (CRITICAL)

**Location:** `crates/teleop/src/lib.rs` line 380

**Problem:** The UDP teleop server sends commands with `.await`:

```rust
// In select! loop
result = socket.recv_from(&mut buf) => {
    if let Some(cmd) = Self::parse_command(&buf[..len]) {
        // PROBLEM: This blocks if channel is full (capacity 32)
        self.command_tx.send(cmd).await;
    }
}
```

**Why this is bad:**
- `tokio::sync::mpsc::Sender::send().await` blocks until space is available
- If the main loop is slow/blocked, the channel fills up
- Once full, the UDP server task blocks waiting to send
- New UDP packets queue in the kernel buffer (we saw rx_queue grow to 2-3KB)
- **Cascading failure:** slow receiver → full channel → blocked sender → packets queue

### Issue 3: Synchronous CAN I/O on Async Runtime (HIGH)

**Location:** `bins/bvrd/src/main.rs` lines 1468-1474

**Problem:** CAN bus reads are synchronous with a 1ms timeout:

```rust
// On tokio worker thread
while frames.len() < MAX_CAN_FRAMES_PER_ITERATION {
    match can_interface.recv() {  // Blocking syscall with 1ms timeout
        Ok(Some(frame)) => frames.push(frame),
        Ok(None) => break,
        Err(_) => break,
    }
}
```

**Why this is bad:**
- Each `recv()` can block for up to 1ms
- 100 frames/iteration × 1ms timeout = potential 100ms blocking per loop iteration
- This happens on a tokio worker thread, blocking other async tasks

### Issue 4: Rerun Recording I/O (MEDIUM)

**Location:** `crates/recording/src/lib.rs`

**Problem:** Rerun `stream.log()` calls are synchronous and may perform I/O:

```rust
// Called from main control loop (on tokio worker)
stream.log("robot/pose", &rerun::Transform3D::from_translation_rotation(...))?;
```

**Why this is bad:**
- Rerun SDK may buffer or flush to disk synchronously
- Multiple log calls per loop iteration compound the issue
- Disk I/O latency can be 1-10ms per call

### Issue 5: SLAM Thread CPU Starvation (MEDIUM)

**Location:** `bins/bvrd/src/main.rs` lines 1256-1294

**Observation:** SLAM thread consumes 97.8% CPU.

**Problem:** While SLAM is correctly on a dedicated `std::thread`, it may be:
- Consuming CPU cycles that tokio workers need
- Causing cache thrashing with tokio workers
- Creating memory pressure affecting async allocations

---

## Root Cause Chain

```
1. SLAM thread consumes 97% CPU
         │
         ▼
2. Tokio workers get less CPU time, main loop runs slower
         │
         ▼
3. Main loop holds std::sync::Mutex while slow
         │
         ▼
4. Other tokio tasks waiting for mutex block their workers
         │
         ▼
5. Command channel fills up (capacity 32)
         │
         ▼
6. UDP teleop send().await blocks waiting for channel space
         │
         ▼
7. UDP select! loop stops polling socket.recv_from()
         │
         ▼
8. Kernel buffers incoming UDP packets (rx_queue grows)
         │
         ▼
9. Commands never reach main loop → watchdog timeout
```

---

## Recommended Fixes

### Fix 1: Replace std::sync::Mutex with tokio::sync::Mutex (CRITICAL)

```rust
// Before
let shared = Arc::new(std::sync::Mutex::new(SharedState { ... }));
// Usage: let state = shared.lock().unwrap();

// After
let shared = Arc::new(tokio::sync::Mutex::new(SharedState { ... }));
// Usage: let state = shared.lock().await;
```

**Benefit:** `tokio::sync::Mutex::lock().await` yields to the runtime instead of blocking the worker thread.

**Caveat:** Requires making lock-holding code async-aware. Cannot hold across `.await` points without care.

### Fix 2: Use try_send() Instead of send().await (CRITICAL)

```rust
// Before
self.command_tx.send(cmd).await;

// After - Option A: Drop old commands when channel full
if self.command_tx.try_send(cmd).is_err() {
    warn!("Command channel full, dropping command");
}

// After - Option B: Use bounded send with timeout
match tokio::time::timeout(
    Duration::from_millis(10),
    self.command_tx.send(cmd)
).await {
    Ok(Ok(())) => {},
    Ok(Err(_)) => warn!("Channel closed"),
    Err(_) => warn!("Channel send timeout"),
}
```

**Benefit:** Prevents UDP server from blocking indefinitely. Commands may be dropped under load, but the system stays responsive.

### Fix 3: Move CAN I/O to Dedicated Thread (HIGH)

```rust
// Create dedicated CAN reader thread
let (can_tx, mut can_rx) = mpsc::channel::<Vec<Frame>>(4);

std::thread::Builder::new()
    .name("can-reader".to_string())
    .spawn(move || {
        loop {
            let mut frames = Vec::with_capacity(100);
            while frames.len() < 100 {
                match can_bus.recv() {
                    Ok(Some(frame)) => frames.push(frame),
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            if !frames.is_empty() {
                let _ = can_tx.blocking_send(frames);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    })?;

// In main loop - non-blocking receive
while let Ok(frames) = can_rx.try_recv() {
    // Process frames
}
```

**Benefit:** CAN blocking I/O happens on dedicated thread, not tokio workers.

### Fix 4: Move Recording to Background Task (MEDIUM)

```rust
// Create recording channel
let (record_tx, mut record_rx) = mpsc::channel::<RecordingEvent>(64);

// Spawn dedicated recording task
tokio::task::spawn_blocking(move || {
    while let Some(event) = record_rx.blocking_recv() {
        match event {
            RecordingEvent::Pose(pose) => recorder.log_pose(&pose),
            RecordingEvent::Velocity(cmd, actual) => recorder.log_velocity(&cmd, &actual),
            // ...
        }
    }
});

// In main loop - non-blocking send
let _ = record_tx.try_send(RecordingEvent::Pose(pose));
```

**Benefit:** Rerun I/O happens on blocking thread pool, not tokio workers.

### Fix 5: Throttle SLAM CPU Usage (MEDIUM)

```rust
// In SLAM thread loop
while let Ok((scan, odom_pose)) = slam_thread_rx.recv() {
    slam.process_scan(&scan);

    // Yield CPU periodically to other threads
    std::thread::sleep(Duration::from_millis(1));
}
```

Or use CPU affinity to pin SLAM to specific cores.

### Fix 6: Increase Channel Capacity (LOW - BANDAID)

```rust
// Before
let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(32);

// After - larger buffer provides more headroom
let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(256);
```

**Benefit:** More buffer for bursty traffic.
**Caveat:** Doesn't fix root cause, just delays symptoms.

---

## Implementation Priority

| Priority | Fix | Effort | Impact |
|----------|-----|--------|--------|
| P0 | Fix 2: try_send() | Low | High - Prevents UDP server blocking |
| P0 | Fix 1: tokio::sync::Mutex | Medium | High - Prevents worker blocking |
| P1 | Fix 3: CAN dedicated thread | Medium | Medium - Reduces worker blocking |
| P1 | Fix 4: Recording background | Medium | Medium - Reduces I/O on workers |
| P2 | Fix 5: SLAM throttling | Low | Low - Reduces CPU contention |
| P3 | Fix 6: Channel capacity | Low | Low - Bandaid |

---

## Verification Plan

After implementing fixes, verify:

1. **Battery logs continue** - Main loop iterations consistent at 100Hz
2. **Commands process reliably** - SetMode via UDP works after 5+ minutes uptime
3. **CPU usage normalized** - bvrd total CPU < 50% (excluding SLAM)
4. **No rx_queue growth** - `cat /proc/net/udp | grep 12e8` shows rx_queue = 0
5. **Watchdog only fires legitimately** - Only when operator actually disconnects

---

## References

- [Tokio: Bridging with Sync Code](https://tokio.rs/tokio/topics/bridging)
- [Tokio: Shared State](https://tokio.rs/tokio/tutorial/shared-state)
- [Alice Ryhl: Async: What is blocking?](https://ryhl.io/blog/async-what-is-blocking/)
- [Rerun SDK Threading Model](https://www.rerun.io/docs/reference/sdk/threading)
