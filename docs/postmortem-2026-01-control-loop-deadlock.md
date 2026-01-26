# Postmortem: Control Loop Deadlock During LiDAR Streaming

**Date**: January 23-24, 2026
**Duration**: ~8 hours of debugging
**Severity**: Critical (motor control loop completely frozen)
**Status**: Resolved

## Executive Summary

The bvrd control loop would hang indefinitely approximately 10-16 seconds after a WebRTC client connected with LiDAR streaming enabled. The root cause was calling `recorder.log_lidar_scan()` (which uses the rerun SDK internally) while holding a `std::sync::Mutex`. The rerun SDK uses rayon for parallel processing, and this interaction caused a deadlock that froze the 100 Hz motor control loop.

## Timeline

### January 23, 2026

**~14:00** - Initial report: Control loop hangs after WebRTC connection with LiDAR streaming enabled. Loop runs fine without LiDAR, or with LiDAR but no WebRTC connection.

**~14:30** - First hypothesis: tokio runtime starvation. The control loop was using `tokio::time::sleep().await` which depends on the tokio scheduler. WebRTC tasks might be starving the control loop.

**~15:00** - Attempted fix #1: Replaced `tokio::sync::Mutex` with `std::sync::Mutex` for the shared state. Rationale: tokio mutexes can cause issues when held across await points.

**Result**: No improvement. Loop still hangs.

**~15:30** - Attempted fix #2: Replaced `tokio::time::sleep().await` with `std::thread::sleep()` in the control loop. Rationale: Remove dependency on tokio scheduler for timing.

**Result**: No improvement. Loop still hangs.

**~16:00** - Attempted fix #3: Moved entire control loop to a dedicated `std::thread::spawn()` instead of running in tokio task. Rationale: Complete isolation from async runtime.

**Result**: No improvement. Loop still hangs at exactly the same point (~iteration 1600).

**~16:30** - Added heartbeat monitoring thread to detect hangs. Separate thread prints "STUCK" if iteration counter doesn't advance for 5 seconds.

**Result**: Confirmed loop is genuinely stuck, not just slow. Heartbeat shows iteration frozen at 1615.

**~17:00** - Added PHASE checkpoint logging throughout control loop to identify exact hang location:
- PHASE 0a: after sleep
- PHASE 0b: CAN recv done
- PHASE 1: CAN process done
- PHASE 2: estop done
- PHASE 3: dispatch done
- PHASE 4: nav done
- PHASE 5: motor done
- PHASE 5a-5r: granular telemetry/recording phases
- PHASE 6: telemetry done

**~17:30** - First checkpoint deployment. Logs showed loop progressing through all phases normally until hang.

**Problem**: journald rate limiting suppressed 3909+ messages, hiding the exact hang point.

**~18:00** - Reduced checkpoint frequency (every 10th iteration) and added verbose mode after iteration 900. Still couldn't pinpoint exact location due to log suppression.

**~18:30** - Pivoted to GDB debugging. Attached to running process during hang:
```bash
ssh frog-0 "sudo gdb -p $(pgrep bvrd) -batch -ex 'thread apply all bt'"
```

**Key finding**: Thread 17 (control loop) blocked on `std::sys::sync::mutex::futex::Mutex::lock_contended()`. This was the smoking gun - the loop was blocked trying to acquire a mutex.

**~19:00** - Analyzed thread dump:
- Thread 17 (control loop): blocked on mutex
- Threads 10-13: rayon worker threads (from rerun SDK)
- Thread 1: tokio runtime

The presence of rayon threads was suspicious - we don't use rayon directly.

### January 24, 2026

**~03:30** - Root cause identified: `recorder.log_lidar_scan()` was being called while holding the `shared` mutex (line 2081). The rerun SDK internally uses rayon for parallel processing. When rayon workers interact with the mutex in certain ways, it causes a deadlock.

**~03:45** - Fix implemented: Removed `recorder.log_lidar_scan()` from the critical section. LiDAR data is now:
1. Cloned outside the mutex lock
2. Sent to SLAM background task (non-blocking)
3. Used for costmap updates only

Recording of LiDAR scans disabled for now (TODO: move to async recording task).

**~04:00** - Fix deployed and verified. Control loop ran continuously past iteration 11,000+ with LiDAR streaming enabled. Previously hung at ~1,600.

**~04:05** - Cleanup: Removed all PHASE debug checkpoints from main.rs. Kept heartbeat monitoring thread for future debugging.

## Root Cause Analysis

### The Bug

```rust
// BEFORE (problematic):
let mut state = shared.lock().unwrap();  // Acquire mutex
// ... motor commands, telemetry building ...

if let Some(ref scan) = &*lidar_rx.borrow_and_update() {
    let _ = recorder.log_lidar_scan(scan);  // <-- DEADLOCK HERE
    // rerun SDK uses rayon internally
    // rayon workers somehow interact with mutex
}
```

### Why It Deadlocked

1. Control loop acquires `shared` mutex
2. Control loop calls `recorder.log_lidar_scan()`
3. Rerun SDK spawns work on rayon thread pool
4. Rayon workers (or rerun internals) attempt to acquire same mutex or block in a way that prevents the control loop from releasing it
5. Control loop cannot proceed, mutex never released
6. Deadlock

This is a **known rayon limitation** documented in [rayon-rs/rayon#592](https://github.com/rayon-rs/rayon/issues/592): "Using rayon under a Mutex can lead to deadlocks". Rayon's work-stealing behavior can cause implicit lock recursion when a mutex is held while calling into rayon-using code.

**The rule**: Never call into libraries that use rayon (or any work-stealing thread pool) while holding a mutex.

### Why It Only Happened With LiDAR + WebRTC

- Without LiDAR: `log_lidar_scan()` never called
- Without WebRTC: Different code path, possibly different timing that avoided the race
- With both: Increased load on rerun SDK triggered the deadlock

## The Fix

```rust
// AFTER (fixed):
// Clone scan OUTSIDE the mutex lock
let lidar_scan_clone = if lidar_rx.has_changed().unwrap_or(false) {
    lidar_rx.borrow_and_update().clone()
} else {
    None
};

// Process LiDAR without holding shared mutex
if let Some(ref scan) = lidar_scan_clone {
    // Send to SLAM (non-blocking channel)
    if let Some(ref tx) = slam_scan_tx {
        let _ = tx.try_send((scan.clone(), pose_estimator.pose()));
    }

    // Update costmap (doesn't use rerun)
    if let Some(ref mut nav) = navigation_controller {
        nav.update_costmap(&scan, &robot_tf);
    }

    // NOTE: recorder.log_lidar_scan() removed - was causing deadlock
    // TODO: Move to async recording task
}
```

## What We Tried (And Why It Didn't Work)

| Attempt | Rationale | Result | Why It Failed |
|---------|-----------|--------|---------------|
| tokio::Mutex → std::Mutex | Avoid async mutex issues | No change | Problem wasn't async mutex |
| tokio::sleep → std::thread::sleep | Remove tokio scheduler dependency | No change | Problem wasn't scheduling |
| Move loop to std::thread | Complete isolation from async | No change | Deadlock was in std::sync::Mutex |
| PHASE checkpoints | Find exact hang location | Logs suppressed | journald rate limiting |
| Reduce checkpoint frequency | Avoid suppression | Still unclear | Hang location between checkpoints |
| GDB thread dump | See what threads are doing | **SUCCESS** | Showed mutex contention |

## Lessons Learned

### 1. GDB is the Right Tool for Deadlocks

Checkpoint logging is useful for understanding control flow, but for deadlocks, you need to see thread state. `gdb -batch -ex 'thread apply all bt'` immediately showed the blocked mutex.

### 2. Be Suspicious of Third-Party Library Threading

The rerun SDK's internal use of rayon was not obvious. When debugging mutex issues, check if any called code might spawn threads or use thread pools internally.

### 3. Minimize Critical Section Duration

The original code held the mutex while:
- Building telemetry structs
- Sending to channels
- Recording to rerun
- Updating costmap

Only motor commands and shared state updates truly needed the lock.

### 4. journald Rate Limiting Hides Bugs

High-frequency logging gets suppressed. For debugging timing-critical code:
- Use GDB for thread state
- Write to a file instead of journald
- Use a separate monitoring thread (like our heartbeat)

### 5. Heartbeat Threads Are Valuable

The heartbeat thread immediately confirmed the loop was stuck vs. just slow. This pattern should be standard for safety-critical loops:

```rust
let heartbeat_counter = Arc::new(AtomicU64::new(0));
let counter_clone = heartbeat_counter.clone();

std::thread::spawn(move || {
    let mut last = 0;
    loop {
        std::thread::sleep(Duration::from_secs(5));
        let current = counter_clone.load(Ordering::Relaxed);
        if current == last {
            eprintln!("HEARTBEAT: Loop STUCK at iteration {}", current);
        } else {
            eprintln!("HEARTBEAT: Loop running, iterations: {} (+{})", current, current - last);
        }
        last = current;
    }
});
```

### 6. This Is a Known Rayon Issue

The deadlock pattern we hit is documented in [rayon-rs/rayon#592](https://github.com/rayon-rs/rayon/issues/592). When debugging similar issues, check if any dependency uses rayon (search `Cargo.lock` for `rayon`).

## Reproduction Attempts

We attempted to create a minimal reproduction on macOS x86_64 but could not trigger the deadlock. The issue appears to be specific to Linux aarch64, possibly due to:

1. **Futex implementation**: Linux uses futexes for mutex implementation
2. **ARM64 memory model**: Weaker memory ordering than x86_64
3. **Thread scheduling**: Linux CFS scheduler behavior
4. **Timing-dependent**: Required specific load from WebRTC + LiDAR

The reproduction code is in `/tmp/rerun-deadlock-repro/` for future investigation.

## Action Items

- [x] Fix the deadlock by removing rerun call from critical section
- [x] Clean up debug checkpoints
- [x] Keep heartbeat monitoring for future debugging
- [x] Document root cause (rayon#592)
- [x] Submit docs PR to rerun warning about mutex + rayon pattern (https://github.com/rerun-io/rerun/pull/12579)
- [ ] Move LiDAR recording to async task (recording currently disabled)
- [ ] Audit other recorder calls for similar issues
- [ ] Consider lock-free communication patterns for telemetry
- [ ] Add integration test that runs WebRTC + LiDAR for extended period

## Appendix: GDB Thread Dump (Key Sections)

```
Thread 17 (Thread 0x7f8877fff640 (LWP 77488) "tokio-runtime-w"):
#0  syscall () at ../sysdeps/unix/sysv/linux/aarch64/syscall.S:38
#1  std::sys::sync::mutex::futex::Mutex::lock_contended ()
#2  std::sys::sync::mutex::futex::Mutex::lock ()
#3  std::sync::mutex::Mutex<T>::lock ()
#4  bvrd::main::{{closure}}::{{closure}} ()
    at bins/bvrd/src/main.rs:2337

Thread 10 (Thread 0x7f88e57fe640 (LWP 77481) "tokio-runtime-w"):
#0  syscall () at ../sysdeps/unix/sysv/linux/aarch64/syscall.S:38
#1  parking_lot_core::parking_lot::park ()
#2  rayon_core::sleep::Sleep::sleep ()
#3  rayon_core::registry::WorkerThread::wait_until_cold ()

Thread 11-13: Similar rayon worker stacks
```

The control loop (Thread 17) was blocked on `Mutex::lock_contended` at line 2337, which was inside the LiDAR processing section that called into the rerun SDK.
