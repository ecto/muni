//! Control loop — runs at 100 Hz on a dedicated `std::thread`.
//!
//! Receives a fully-initialized [`DaemonContext`] from [`crate::init`] and
//! drives the main sense-plan-act loop until shutdown.

use crate::init::DaemonContext;
use crate::state_types::*;

use control_loop::NavigationState;
use dispatch::DispatchEvent;
use hal::EStopEvent;
use lidar::LidarCommand;
use metrics::MetricsSnapshot;
use policy::PolicyObservation;
use slam::SlamState;
use state::Event;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use teleop::Telemetry;
use tokio::sync::watch;
use tools::{protocol, ToolOutput, ToolType};
use transforms::Transform2D;
use tracing::{debug, error, info, warn};
use types::{Command, Mode, PowerStatus, SlamStatus, Twist};

/// Run the 100 Hz control loop on the calling thread (blocks forever).
pub(crate) fn run(ctx: DaemonContext) {
    // Destructure all fields from the context into local bindings.
    let DaemonContext {
        can_interface,
        is_sim,
        shared,
        cmd_tx,
        mut cmd_rx,
        telemetry_tx,
        metrics_tx,
        mixer,
        mut rate_limiter,
        collision_monitor,
        mut watchdog,
        mut odometry,
        mut pose_estimator,
        mut gps_rx,
        lidar_rx,
        imu_rx,
        lidar_cmd_tx,
        lidar_enabled,
        lidar_status_rx,
        slam_scan_tx,
        slam_state_rx,
        mut navigation_controller,
        mut obstacle_tracker,
        costmap_tx,
        obstacles_tx,
        path_tx,
        loaded_policy,
        autonomous_goal,
        autonomous_max_linear,
        autonomous_max_angular,
        norm_config,
        dispatch_client,
        mut dispatch_rx,
        current_dispatch_task,
        mut estop_rx,
        estop_require_confirmation,
        mut recorder,
        recording_enabled,
        rtc_metrics,
        peer_count,
        sleep_timeout_secs,
        mount_pitch_rad,
        discovery_enabled,
        dispatch_enabled,
        tokio_handle,
        mut loop_count,
        heartbeat_counter,
        mut heartbeat_count,
        mut last_heartbeat_log,
        mut tool_command,
        mut idle_since,
        mut last_mode,
        mut sys_metrics,
        mut last_sys_refresh,
        mut can_errors_active,
        mut can_backoff_active,
        mut can_error_count,
        mut consecutive_can_errors,
        mut last_policy_action,
        mut last_policy_action_time,
        dispatch_semaphore,
        mut attitude_filter,
        mut imu_preintegrator,
        mut last_tick,
        control_period,
        mut hardware_estop_latched,
        mut dt_min_ms,
        mut dt_max_ms,
        mut last_gps_update,
        mut last_lidar_update,
        mut last_slam_update,
        mut last_subsystem_check,
        mut dispatch_connected_live,
        heartbeat_stalled,
        seq_tracker,
        mut eth_discovery,
        depth_geometries,
        depth_config: _depth_config,
        mut depth_visual_odometry,
        depth_raw_rxs,
        depth_enabled,
    } = ctx;

    // Pre-compute IMU mount transform (used for both attitude and yaw extraction)
    let (sin_mount_pitch, cos_mount_pitch) = (mount_pitch_rad as f64).sin_cos();
    let mut vel_tracker = teleop::VelocityTracker::new();

    // Clone dispatch task handle for event processing
    let dispatch_task_clone = current_dispatch_task.clone();

    // Clone lidar/slam receivers for the loop
    let mut lidar_rx_local = lidar_rx.clone();
    let mut slam_state_rx_local: Option<watch::Receiver<SlamState>> = slam_state_rx.clone();

    // Control loop timing — overwritten every iteration, initial value unused
    #[allow(unused_assignments)]
    let mut dt_last_ms: f64 = 0.0;

    // Auto-sleep setup
    let sleep_timeout = Duration::from_secs(sleep_timeout_secs);

    // Dance mode choreography timer (reset when entering Dance)
    let mut dance_start = Instant::now();

    // CAN bus health tracking constants
    const CAN_ERROR_BACKOFF_THRESHOLD: u32 = 10;
    const CAN_RETRY_INTERVAL: u64 = 100;

    // Policy execution timeout tracking for autonomous mode safety
    const POLICY_WARN_THRESHOLD: Duration = Duration::from_millis(20);
    const POLICY_ERROR_THRESHOLD: Duration = Duration::from_millis(50);
    const POLICY_TIMEOUT: Duration = Duration::from_millis(30);
    const POLICY_ACTION_STALENESS: Duration = Duration::from_millis(200);

    // -----------------------------------------------------------------------
    // Heartbeat thread for deadlock detection — runs on std::thread, not tokio.
    // This helps diagnose if main loop is stuck vs tokio starvation.
    // -----------------------------------------------------------------------
    let heartbeat_counter_clone = heartbeat_counter.clone();
    let heartbeat_stalled_clone = heartbeat_stalled.clone();
    std::thread::spawn(move || {
        let mut last_count = 0u64;
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let current = heartbeat_counter_clone.load(Ordering::Relaxed);
            if current == last_count {
                warn!(
                    iteration = current,
                    "Heartbeat: main loop STUCK"
                );
                heartbeat_stalled_clone.store(true, Ordering::Relaxed);
            } else {
                debug!(
                    iteration = current,
                    delta = current - last_count,
                    "Heartbeat: main loop running"
                );
                heartbeat_stalled_clone.store(false, Ordering::Relaxed);
            }
            last_count = current;
        }
    });

    // -----------------------------------------------------------------------
    // Main 100 Hz control loop
    // -----------------------------------------------------------------------

    // Checkpoint logging at end of each 100-iteration block
    let _last_checkpoint = 0u64;

    loop {
        loop_count += 1;
        heartbeat_counter.store(loop_count, Ordering::Relaxed);

        // Wait for next tick - use std::thread::sleep to avoid tokio scheduler dependency.
        // This prevents WebRTC/async task starvation from affecting motor control timing.
        // With the multi-threaded runtime (8 workers), other workers handle async tasks.
        let elapsed = last_tick.elapsed();
        if elapsed < control_period {
            std::thread::sleep(control_period - elapsed);
        }

        let dt = last_tick.elapsed().as_secs_f64();
        last_tick = Instant::now();

        // Track control loop timing metrics
        let dt_ms = dt * 1000.0;
        dt_last_ms = dt_ms;
        if dt_ms > dt_max_ms {
            dt_max_ms = dt_ms;
        }
        if dt_ms < dt_min_ms {
            dt_min_ms = dt_ms;
        }

        // Debug: warn if control loop is running slow
        if dt > 0.1 {
            warn!(dt_ms = format!("{:.0}", dt * 1000.0), "Control loop slow iteration");
        }

        // Tick simulation if in sim mode
        can_interface.tick(dt);

        // Read CAN frames (limited per iteration to prevent cascade delays)
        // VESCs send ~1kHz status each, so 4 VESCs = 4000 frames/sec.
        // At 100Hz loop, expect ~40 frames/iteration. Cap at 100 to bound latency.
        // Process all frames with a single mutex acquisition to minimize contention.
        const MAX_CAN_FRAMES_PER_ITERATION: usize = 100;

        // Collect frames first (no mutex needed for read)
        let mut frames: Vec<can::Frame> = Vec::with_capacity(MAX_CAN_FRAMES_PER_ITERATION);
        while frames.len() < MAX_CAN_FRAMES_PER_ITERATION {
            match can_interface.recv() {
                Ok(Some(frame)) => frames.push(frame),
                Ok(None) => break, // No more frames
                Err(_) => break,   // Error reading
            }
        }

        // Process all collected frames with a single mutex acquisition
        if !frames.is_empty() {
            let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
            for frame in &frames {
                state.drivetrain.process_frame(frame);
                state.tool_registry.process_frame(frame);
            }
        }

        // Check for newly discovered Ethernet tools and auto-register them
        if let Some(ref eth) = eth_discovery {
            let new_mcus = eth.online_mcus();
            if !new_mcus.is_empty() {
                let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                for (serial, tool_type, addr) in new_mcus {
                    // Check if already registered
                    if state.tool_registry.iter().any(|t| t.info().serial == serial) {
                        continue;
                    }
                    match tool_type {
                        ToolType::Lights => {
                            info!(serial, %addr, "Registering Ethernet headlights");
                            state.tool_registry.register(Box::new(
                                tools::EthHeadlights::new(serial, addr),
                            ));
                        }
                        _ => {
                            debug!(serial, ?tool_type, "Ignoring unknown Ethernet tool type");
                        }
                    }
                }
            }
        }

        // Process hardware e-stop events (highest priority - before any other commands)
        while let Ok(event) = estop_rx.try_recv() {
            match event {
                EStopEvent::Pressed => {
                    warn!("Hardware e-stop button PRESSED");
                    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                    state.state_machine.transition(Event::EStop);
                    state.commanded_twist = Twist::default();
                    rate_limiter.reset();
                    hardware_estop_latched = true;

                    // Send e-stop LED pattern
                    let led_cmd = state.state_machine.led_command();
                    if let Err(e) = can_interface.send(&led_cmd.to_frame()) {
                        debug!(?e, "Failed to send e-stop LED command");
                    }
                }
                EStopEvent::Released => {
                    if estop_require_confirmation {
                        // Latch semantics: button release doesn't auto-clear e-stop
                        // Operator must send explicit EStopRelease command via teleop
                        info!(
                            "Hardware e-stop button released (awaiting operator confirmation to resume)"
                        );
                    } else {
                        // Auto-release: button release clears e-stop immediately
                        info!("Hardware e-stop button released (auto-release enabled)");
                        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                        state.state_machine.transition(Event::EStopRelease);
                        hardware_estop_latched = false;

                        // Send idle LED pattern
                        let led_cmd = state.state_machine.led_command();
                        if let Err(e) = can_interface.send(&led_cmd.to_frame()) {
                            debug!(?e, "Failed to send e-stop release LED command");
                        }
                    }
                }
            }
        }

        // Log battery voltage every 10 seconds for diagnostics
        if loop_count % 1000 == 0 {
            let state = shared.lock().unwrap_or_else(|e| e.into_inner());
            let voltage = state.drivetrain.battery_voltage();
            if voltage > 0.0 {
                info!(voltage = format!("{:.1}V", voltage), "Battery status");
            } else {
                warn!("Battery voltage not received - check VESC CAN status settings");
            }
        }

        // Process incoming commands (non-blocking, "latest Twist wins")
        // Drain all commands first, keeping only the latest Twist
        let mut latest_twist: Option<Twist> = None;
        let mut other_commands: Vec<Command> = Vec::new();

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Twist(twist) => {
                    // Always overwrite - only the latest Twist matters
                    latest_twist = Some(twist);
                }
                other => {
                    // Queue non-Twist commands for ordered processing
                    other_commands.push(other);
                }
            }
        }

        // Process non-Twist commands (order matters for these)
        if !other_commands.is_empty() {
            let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
            for cmd in other_commands {
                match cmd {
                    Command::EStop => {
                        warn!("E-Stop command received");
                        state.state_machine.transition(Event::EStop);
                        rate_limiter.reset();
                    }
                    Command::EStopRelease => {
                        if hardware_estop_latched {
                            // This is the operator confirmation to clear hardware e-stop
                            info!("E-Stop release confirmed by operator (clearing hardware latch)");
                            hardware_estop_latched = false;
                        } else {
                            info!("E-Stop release command received");
                        }
                        state.state_machine.transition(Event::EStopRelease);
                    }
                    Command::SetMode(mode) => {
                        info!(
                            requested_mode = ?mode,
                            current_mode = ?state.state_machine.mode(),
                            "SetMode command processing"
                        );
                        let current = state.state_machine.mode();
                        let event = match mode {
                            Mode::Disabled => Event::Disable, // old consoles send byte 0 -> maps to Idle now
                            Mode::Idle => {
                                if current == Mode::Sleep {
                                    // Reset auto-sleep timer so we don't immediately re-sleep
                                    idle_since = Instant::now();
                                    Event::Wake
                                } else if current == Mode::Fault {
                                    info!("Clearing fault via SetMode(Idle)");
                                    state.fault_code = FaultCode::None;
                                    Event::FaultClear
                                } else {
                                    Event::Disable // Disable is a no-op when already Idle
                                }
                            }
                            Mode::Teleop => {
                                // Feed watchdog when entering Teleop to prevent immediate timeout
                                watchdog.feed();
                                Event::TeleopCommand
                            }
                            Mode::Autonomous => {
                                // Feed watchdog when entering Autonomous to prevent immediate timeout
                                watchdog.feed();
                                Event::AutonomousRequest
                            }
                            Mode::Dance => {
                                watchdog.feed();
                                Event::DanceRequest
                            }
                            Mode::EStop => Event::EStop,
                            Mode::Sleep => Event::Sleep,
                            _ => continue,
                        };
                        // Cancel navigation when leaving Autonomous
                        if current == Mode::Autonomous && !matches!(event, Event::AutonomousRequest) {
                            if let Some(ref mut nav) = navigation_controller {
                                info!("Cancelling navigation (leaving Autonomous)");
                                nav.cancel();
                            }
                        }
                        state.state_machine.transition(event);
                    }
                    Command::Heartbeat => {
                        watchdog.feed();
                        heartbeat_count += 1;
                        // Log heartbeat rate every 10 seconds when in teleop
                        // Note: we already hold `state` from above, use it directly
                        if last_heartbeat_log.elapsed() > Duration::from_secs(10) {
                            if state.state_machine.mode() == Mode::Teleop {
                                let rate = heartbeat_count as f64 / last_heartbeat_log.elapsed().as_secs_f64();
                                info!(count = heartbeat_count, rate = format!("{:.1}/s", rate), "Heartbeats received (10s window)");
                            }
                            heartbeat_count = 0;
                            last_heartbeat_log = Instant::now();
                        }
                    }
                    Command::Tool(tc) => {
                        watchdog.feed();
                        tool_command = tc;
                    }
                    Command::Twist(_) => unreachable!(), // Already handled above
                    Command::LidarToggle(_) => {} // Handled per-connection in WebRTC
                    Command::SetGoal { x, y } => {
                        if let Some(ref mut nav) = navigation_controller {
                            info!(x, y, "SetGoal command received");
                            nav.set_goal(planner::Waypoint::new(x, y));
                            // Auto-transition to Autonomous if not already
                            let current_mode = state.state_machine.mode();
                            if current_mode != Mode::Autonomous {
                                watchdog.feed();
                                state.state_machine.transition(Event::AutonomousRequest);
                            }
                        } else {
                            warn!("SetGoal received but navigation controller disabled");
                        }
                    }
                }
            }
        }

        // Apply latest Twist command only if in Teleop mode (requires explicit control takeover)
        if let Some(twist) = latest_twist {
            let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
            if state.state_machine.mode() == Mode::Teleop {
                watchdog.feed();
                state.commanded_twist = twist;
            }
        }


        // Process dispatch events (task assignments/cancellations)
        if let Some(ref mut rx) = dispatch_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    DispatchEvent::TaskAssigned(assignment) => {
                        info!(
                            task_id = %assignment.task_id,
                            mission_id = %assignment.mission_id,
                            waypoints = assignment.zone.waypoints.len(),
                            is_loop = assignment.zone.is_loop,
                            "Task assigned from dispatch"
                        );

                        // Set task on navigation controller if using classical nav
                        if let Some(ref mut nav) = navigation_controller {
                            nav.set_task(assignment.clone());
                        }

                        // Also set on legacy dispatch task for policy fallback
                        let task = DispatchedTask::from_assignment(assignment);
                        *dispatch_task_clone.lock().unwrap_or_else(|e| e.into_inner()) = Some(task);

                        // Transition to autonomous mode to execute the task
                        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                        match state.state_machine.mode() {
                            Mode::Idle => {
                                state.state_machine.transition(Event::AutonomousRequest);
                                info!("Transitioned to Autonomous mode for dispatch task");
                            }
                            Mode::Teleop => {
                                // Don't interrupt teleop - operator has control
                                info!("Task queued - currently in Teleop mode");
                            }
                            Mode::Autonomous => {
                                // Already autonomous, will pick up new task
                                info!("New task received while autonomous");
                            }
                            _ => {}
                        }
                    }
                    DispatchEvent::TaskCancelled(task_id) => {
                        info!(task_id = %task_id, "Task cancelled by dispatch");

                        // Cancel on navigation controller
                        if let Some(ref mut nav) = navigation_controller {
                            if nav.current_task().map(|t| t.task_id == task_id).unwrap_or(false) {
                                nav.cancel();
                            }
                        }

                        // Also handle legacy dispatch task
                        let mut task_guard = dispatch_task_clone.lock().unwrap_or_else(|e| e.into_inner());
                        if let Some(ref task) = *task_guard {
                            if task.task_id == task_id {
                                *task_guard = None;
                                // Return to idle if we were executing this task
                                let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                                if state.state_machine.mode() == Mode::Autonomous {
                                    state.state_machine.transition(Event::CommandTimeout);
                                    info!("Returned to Idle after task cancellation");
                                }
                            }
                        }
                    }
                    DispatchEvent::CommandReceived(cmd) => {
                        info!(?cmd, "Voice command received via dispatch");
                        let _ = cmd_tx.try_send(cmd);
                    }
                    DispatchEvent::Connected(true) => {
                        info!("Connected to dispatch service");
                        dispatch_connected_live = true;
                    }
                    DispatchEvent::Connected(false) => {
                        warn!("Disconnected from dispatch service");
                        dispatch_connected_live = false;
                    }
                }
            }
        }

        // Check watchdog - different handling for teleop vs autonomous
        {
            let is_timed_out;
            let current_mode;
            {
                let state = shared.lock().unwrap_or_else(|e| e.into_inner());
                is_timed_out = watchdog.is_timed_out() && state.state_machine.is_driving();
                current_mode = state.state_machine.mode();
            }

            if is_timed_out {
                match current_mode {
                    Mode::Teleop => {
                        // Teleop timeout: operator stopped sending commands, return to Idle
                        let elapsed_ms = watchdog.elapsed_since_last_command()
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        warn!(
                            elapsed_ms,
                            heartbeat_count,
                            "Command watchdog timeout in teleop mode (no commands received)"
                        );
                        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                        state.state_machine.transition(Event::CommandTimeout);
                        state.commanded_twist = Twist::default();
                    }
                    Mode::Autonomous => {
                        // Autonomous timeout: control loop or policy hung - this is critical
                        // Since watchdog is fed on successful policy inference, timeout means
                        // either policy is hanging or control loop stopped executing
                        error!("Watchdog timeout in autonomous mode - control loop or policy hung");

                        // Report failure if executing dispatch task
                        if let Some(ref task) = *current_dispatch_task.lock().unwrap_or_else(|e| e.into_inner()) {
                            if let Some(ref client) = dispatch_client {
                                let task_id = task.task_id;
                                let client_clone = client.clone();
                                let sem = dispatch_semaphore.clone();
                                tokio_handle.spawn(async move {
                                    // Acquire permit (bounded concurrency)
                                    let _permit = sem.acquire().await;
                                    let _ = client_clone
                                        .report_failed(task_id, "Watchdog timeout - control loop hung")
                                        .await;
                                });
                            }
                        }

                        // Transition to Fault (more severe than Idle) for autonomous failures
                        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                        state.fault_code = FaultCode::WatchdogTimeout;
                        state.state_machine.transition(Event::Fault);
                        state.commanded_twist = Twist::default();
                    }
                    _ => {
                        // Shouldn't happen (is_driving only true for Teleop/Autonomous)
                        warn!("Unexpected watchdog timeout in mode {:?}", current_mode);
                        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                        state.state_machine.transition(Event::CommandTimeout);
                        state.commanded_twist = Twist::default();
                    }
                }
                rate_limiter.reset();
            }
        }

        // Auto-sleep after idle/disabled timeout (suppressed while a console is connected)
        if sleep_timeout_secs > 0 {
            let mode = shared.lock().unwrap_or_else(|e| e.into_inner()).state_machine.mode();
            let peers = peer_count.load(Ordering::Relaxed);
            if matches!(mode, Mode::Idle)
                && idle_since.elapsed() > sleep_timeout
                && peers == 0
            {
                info!(
                    elapsed_secs = idle_since.elapsed().as_secs(),
                    mode = ?mode,
                    "Auto-sleep timeout reached"
                );
                let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                state.state_machine.transition(Event::Sleep);
            }
        }

        // Get current mode and commanded twist for control decisions
        let (current_mode, commanded_twist) = {
            let state = shared.lock().unwrap_or_else(|e| e.into_inner());
            (state.state_machine.mode(), state.commanded_twist)
        };

        // Track policy intention for telemetry visualization
        let mut policy_intention: Option<(f32, f32)> = None;

        // Compute motor outputs based on mode
        let (mut target_twist, boost_active) = match current_mode {
            Mode::Autonomous => {
                // Autonomous mode: use classical navigation (A* + pursuit) or policy
                let current_pose = if is_sim {
                    can_interface.pose()
                } else {
                    pose_estimator.pose()
                };

                // Classical navigation stack (preferred when enabled)
                if let Some(ref mut nav) = navigation_controller {
                    // Update navigation controller
                    let nav_output = nav.update(&current_pose, dt);

                    // Debug: log navigation state on first few iterations or state changes
                    if loop_count < 5 || loop_count % 100 == 0 {
                        debug!(
                            state = ?nav_output.state,
                            dist = format!("{:.2}", nav_output.distance_to_goal),
                            twist_linear = format!("{:.3}", nav_output.twist.linear),
                            twist_angular = format!("{:.3}", nav_output.twist.angular),
                            pose_x = format!("{:.2}", current_pose.x),
                            pose_y = format!("{:.2}", current_pose.y),
                            pose_theta = format!("{:.2}", current_pose.theta),
                            "Navigation update"
                        );
                    }

                    // Handle navigation state transitions
                    match nav_output.state {
                        NavigationState::GoalReached => {
                            // Advance to next waypoint or complete task
                            let task_complete = nav.advance_waypoint();

                            if let Some((task_id, progress, waypoint, lap)) = nav.task_progress() {
                                // Report progress to dispatch service
                                if let Some(ref client) = dispatch_client {
                                    let client_clone = client.clone();
                                    let sem = dispatch_semaphore.clone();
                                    tokio_handle.spawn(async move {
                                        let _permit = sem.acquire().await;
                                        let _ = client_clone.report_progress(task_id, progress, waypoint, lap).await;
                                    });
                                }
                            }

                            if task_complete {
                                // Task complete - report and return to idle
                                if let Some(task) = nav.current_task() {
                                    let task_id = task.task_id;
                                    let laps = task.lap;
                                    info!(task_id = %task_id, laps, "Navigation task complete");

                                    if let Some(ref client) = dispatch_client {
                                        let client_clone = client.clone();
                                        let sem = dispatch_semaphore.clone();
                                        tokio_handle.spawn(async move {
                                            let _permit = sem.acquire().await;
                                            let _ = client_clone.report_complete(task_id, laps).await;
                                        });
                                    }
                                }

                                let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                                state.state_machine.transition(Event::CommandTimeout);
                            }
                        }
                        NavigationState::Failed => {
                            // Navigation failed - report and return to idle
                            if let Some(task) = nav.current_task() {
                                let task_id = task.task_id;
                                let error = nav_output.error.clone().unwrap_or_else(|| "Navigation failed".to_string());
                                error!(task_id = %task_id, error = %error, "Navigation failed");

                                if let Some(ref client) = dispatch_client {
                                    let client_clone = client.clone();
                                    let sem = dispatch_semaphore.clone();
                                    tokio_handle.spawn(async move {
                                        let _permit = sem.acquire().await;
                                        let _ = client_clone.report_failed(task_id, &error).await;
                                    });
                                }
                            }

                            nav.cancel();
                            let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                            state.state_machine.transition(Event::CommandTimeout);
                        }
                        NavigationState::ObstacleStopped => {
                            debug!("Stopped for obstacle");
                        }
                        NavigationState::Recovering(_) => {
                            debug!("Executing recovery behavior");
                        }
                        _ => {}
                    }

                    // Feed watchdog unconditionally — the control loop reaching this
                    // point proves it's healthy. Navigation has its own timeouts
                    // (blocked_timeout, recovery cycles) for handling stuck states.
                    // Previously only active states fed the watchdog, which caused
                    // spurious Fault transitions when navigation was between goals
                    // and the console was briefly disconnected (no heartbeats).
                    watchdog.feed();

                    // Publish navigation path for console visualization
                    {
                        use control_loop::NavigationState;
                        use teleop::path_stream::{nav_state, PathSnapshot};
                        let nav_state_wire = match nav.state() {
                            NavigationState::Idle => nav_state::IDLE,
                            NavigationState::Planning => nav_state::PLANNING,
                            NavigationState::Following => nav_state::FOLLOWING,
                            NavigationState::ObstacleStopped => nav_state::OBSTACLE_STOPPED,
                            NavigationState::Replanning => nav_state::REPLANNING,
                            NavigationState::Recovering(_) => nav_state::RECOVERING,
                            NavigationState::GoalReached => nav_state::GOAL_REACHED,
                            NavigationState::Failed => nav_state::FAILED,
                        };
                        let (waypoints, current_wp) = if let Some(path) = nav.planned_path() {
                            let wps: Vec<(f32, f32)> = path.waypoints.iter()
                                .map(|w| (w.x as f32, w.y as f32))
                                .collect();
                            let idx = nav.current_task()
                                .map(|t| t.current_waypoint as u16)
                                .unwrap_or(0);
                            (wps, idx)
                        } else {
                            (vec![], 0)
                        };
                        let mppi_trajectory: Vec<(f32, f32)> = nav_output.mppi_trajectory
                            .iter()
                            .map(|p| (p[0] as f32, p[1] as f32))
                            .collect();
                        let _ = path_tx.send(Some(PathSnapshot {
                            state: nav_state_wire,
                            waypoints,
                            current_waypoint: current_wp,
                            distance_to_goal: nav_output.distance_to_goal as f32,
                            mppi_trajectory,
                        }));
                    }

                    (nav_output.twist, false)
                } else {
                    // Fallback: Policy-based navigation
                    let goal = {
                        let task_guard = current_dispatch_task.lock().unwrap_or_else(|e| e.into_inner());
                        if let Some(ref task) = *task_guard {
                            task.current_goal()
                        } else {
                            autonomous_goal
                        }
                    };

                    if let (Some(policy), Some(goal)) = (&loaded_policy, goal) {
                        let linear_vel = commanded_twist.linear;
                        let angular_vel = commanded_twist.angular;

                        let obs = PolicyObservation::from_raw(
                            current_pose.x, current_pose.y, current_pose.theta,
                            linear_vel, angular_vel,
                            goal[0], goal[1],
                            &norm_config,
                        );

                        let policy_start = Instant::now();
                        let policy_clone = policy.clone();
                        let obs_clone = obs.clone();

                        // Use block_on for policy inference from dedicated control thread
                        // This blocks the thread but that's fine since we're on a dedicated thread
                        let handle = tokio_handle.clone();
                        let inference_result = handle.block_on(async move {
                            tokio::time::timeout(
                                POLICY_TIMEOUT,
                                tokio::task::spawn_blocking(move || policy_clone.infer(&obs_clone)),
                            ).await
                        });

                        let inference_time = policy_start.elapsed();

                        // Stale action check: only reuse last action if it's
                        // recent enough. Otherwise zero the command for safety.
                        let fresh_fallback = || -> Option<policy::PolicyAction> {
                            last_policy_action.filter(|_| {
                                last_policy_action_time
                                    .map_or(false, |t| t.elapsed() < POLICY_ACTION_STALENESS)
                            })
                        };

                        let action = match inference_result {
                            Ok(Ok(Ok(action))) => {
                                if inference_time > POLICY_ERROR_THRESHOLD {
                                    error!(duration_ms = inference_time.as_millis(), "Policy critically slow");
                                } else if inference_time > POLICY_WARN_THRESHOLD {
                                    warn!(duration_ms = inference_time.as_millis(), "Policy slow");
                                }
                                last_policy_action = Some(action);
                                last_policy_action_time = Some(Instant::now());
                                Some(action)
                            }
                            Ok(Ok(Err(e))) => { warn!(?e, "Policy error"); fresh_fallback() }
                            Ok(Err(e)) => { error!("Policy panic: {}", e); fresh_fallback() }
                            Err(_) => { warn!("Policy timeout"); fresh_fallback() }
                        };

                        if let Some(action) = action {
                            watchdog.feed();
                            let twist = action.to_twist(autonomous_max_linear, autonomous_max_angular);

                            // Compute intention point: project velocity forward 1.5s from current pose
                            let lookahead = 1.5_f32;
                            let intent_x = current_pose.x as f32 + twist.linear as f32 * (current_pose.theta as f32).cos() * lookahead;
                            let intent_y = current_pose.y as f32 + twist.linear as f32 * (current_pose.theta as f32).sin() * lookahead;
                            policy_intention = Some((intent_x, intent_y));

                            // Check waypoint reached (policy mode)
                            let dx = goal[0] - current_pose.x;
                            let dy = goal[1] - current_pose.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < 0.5 {
                                let mut task_guard = current_dispatch_task.lock().unwrap_or_else(|e| e.into_inner());
                                if let Some(ref mut task) = *task_guard {
                                    info!(waypoint = task.current_waypoint, "Waypoint reached");

                                    if let Some(ref client) = dispatch_client {
                                        let progress = task.progress_percent();
                                        let task_id = task.task_id;
                                        let waypoint = task.current_waypoint as i32;
                                        let lap = task.lap;
                                        let client_clone = client.clone();
                                        let sem = dispatch_semaphore.clone();
                                        tokio_handle.spawn(async move {
                                            let _permit = sem.acquire().await;
                                            let _ = client_clone.report_progress(task_id, progress, waypoint, lap).await;
                                        });
                                    }

                                    let task_complete = task.advance();
                                    if task_complete {
                                        let task_id = task.task_id;
                                        let laps = task.lap;
                                        info!(task_id = %task_id, "Task complete");

                                        if let Some(ref client) = dispatch_client {
                                            let client_clone = client.clone();
                                            let sem = dispatch_semaphore.clone();
                                            tokio_handle.spawn(async move {
                                                let _permit = sem.acquire().await;
                                                let _ = client_clone.report_complete(task_id, laps).await;
                                            });
                                        }

                                        drop(task_guard);
                                        *current_dispatch_task.lock().unwrap_or_else(|e| e.into_inner()) = None;

                                        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                                        state.state_machine.transition(Event::CommandTimeout);
                                    }
                                }
                            }

                            (twist, false)
                        } else {
                            warn!("Policy failed, no fallback");
                            (Twist::default(), false)
                        }
                    } else {
                        debug!("No policy or goal");
                        (Twist::default(), false)
                    }
                }
            }
            Mode::Teleop => {
                // Teleop mode: use commanded twist directly
                (commanded_twist, commanded_twist.boost)
            }
            Mode::Dance => {
                let t = dance_start.elapsed().as_secs_f64();
                let phase = t % 12.0; // 12-second loop
                let twist = if phase < 3.0 {
                    // Spin right
                    Twist { linear: 0.0, angular: -1.5, boost: false }
                } else if phase < 6.0 {
                    // Spin left
                    Twist { linear: 0.0, angular: 1.5, boost: false }
                } else if phase < 8.0 {
                    // Wiggle (fast oscillation)
                    Twist { linear: 0.0, angular: 3.0 * (t * 8.0).sin(), boost: false }
                } else {
                    // Drive in a circle
                    Twist { linear: 0.5, angular: 1.0, boost: false }
                };
                watchdog.feed();
                (twist, false)
            }
            _ => {
                // Not driving: zero velocity
                (Twist::default(), false)
            }
        };

        // Collision monitor: scale linear velocity near obstacles (all modes)
        if let Some(ref mut nav) = navigation_controller {
            let pose = if is_sim {
                can_interface.pose()
            } else {
                pose_estimator.pose()
            };
            let costmap = nav.costmap();
            let monitor_output = collision_monitor.filter(
                target_twist,
                pose.x, pose.y, pose.theta,
                |x, y| costmap.get_cost(x, y),
            );
            target_twist = monitor_output.twist;

            // Notify navigation controller about collision status
            if current_mode == Mode::Autonomous {
                let is_stopped = monitor_output.is_limited && monitor_output.twist.linear.abs() < 1e-3;
                nav.notify_collision_status(is_stopped);
            }
        }

        // Lock state for motor commands and telemetry
        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());

        // Rate limit for safety (acceleration limits only)
        let mut twist = rate_limiter.limit(target_twist);

        // Boost angular for skid steering (requires more torque than forward motion)
        twist.angular *= 2.5;

        let wheel_vels = mixer.mix(twist);

        // Convert wheel velocities (rad/s) to duty cycle (-1.0 to 1.0)
        // Using duty cycle control for smoother low-speed operation with hall sensors
        // Max wheel velocity at 5 m/s with 0.08m radius = 62.5 rad/s
        const MAX_WHEEL_VEL: f64 = 62.5;
        const NORMAL_DUTY: f64 = 0.5;  // Normal mode: ~50% power (~3 m/s)
        const BOOST_DUTY: f64 = 0.95;  // Boost mode: full blast
        let max_duty = if boost_active { BOOST_DUTY } else { NORMAL_DUTY };
        let wheel_duties: [f64; 4] = [
            (wheel_vels.front_left / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (wheel_vels.front_right / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (wheel_vels.rear_left / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
            (wheel_vels.rear_right / MAX_WHEEL_VEL * max_duty).clamp(-max_duty, max_duty),
        ];

        // Log wheel commands when turning (left != right)
        if (wheel_duties[0] - wheel_duties[1]).abs() > 0.01 {
            info!(
                fl = format!("{:.2}", wheel_duties[0]),
                fr = format!("{:.2}", wheel_duties[1]),
                rl = format!("{:.2}", wheel_duties[2]),
                rr = format!("{:.2}", wheel_duties[3]),
                "Wheel duties (turning)"
            );
        }

        // Send to VESCs using duty cycle control (smoother than RPM at low speeds)
        // Skip VESC commands when CAN bus is unhealthy to avoid overwhelming the driver
        // (which can cause kernel-level issues on Jetson)
        let should_send_vesc = consecutive_can_errors < CAN_ERROR_BACKOFF_THRESHOLD
            || loop_count % CAN_RETRY_INTERVAL == 0;

        let mut send_failed = false;
        if should_send_vesc {
            let vesc_cmds = state.drivetrain.build_duty_commands(wheel_duties);
            for frame in vesc_cmds {
                if let Err(_e) = can_interface.send(&frame) {
                    send_failed = true;
                    can_error_count += 1;
                }
            }
        }

        // Track consecutive errors for backoff
        if send_failed {
            consecutive_can_errors = consecutive_can_errors.saturating_add(1);
        } else if should_send_vesc {
            // Only reset on successful send (not when skipping)
            consecutive_can_errors = 0;
        }

        // Rate-limited CAN error logging - only log state transitions
        if send_failed && !can_errors_active {
            error!(
                "CAN bus errors started - drivetrain commands failing (VESCs powered?)"
            );
            can_errors_active = true;
        } else if consecutive_can_errors >= CAN_ERROR_BACKOFF_THRESHOLD && !can_backoff_active {
            warn!(
                "CAN bus unhealthy - backing off motor commands (retrying every {}ms)",
                CAN_RETRY_INTERVAL * 10
            );
            can_backoff_active = true;
        } else if !send_failed && should_send_vesc && can_errors_active {
            info!(errors = can_error_count, "CAN bus recovered - drivetrain commands succeeding");
            can_errors_active = false;
            can_backoff_active = false;
            can_error_count = 0;
        }

        // Update active tool
        if let Some(tool) = state.tool_registry.active_mut() {
            let output = tool.update(&tool_command);

            // Send tool command via CAN or UDP
            let slot = tool.info().slot;
            match output {
                ToolOutput::Udp { addr, data } => {
                    // Ethernet tool — send via tokio UDP
                    let handle = tokio_handle.clone();
                    handle.spawn(async move {
                        if let Ok(socket) = tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                            let _ = socket.send_to(&data, addr).await;
                        }
                    });
                }
                _ => {
                    let frame = match output {
                        ToolOutput::SetAxis(axis) => Some(protocol::build_command(slot, axis, 0.0)),
                        ToolOutput::SetMotor(motor) => Some(protocol::build_command(slot, 0.0, motor)),
                        ToolOutput::SetBoth { axis, motor } => {
                            Some(protocol::build_command(slot, axis, motor))
                        }
                        ToolOutput::Raw(f) => Some(f),
                        ToolOutput::None => None,
                        ToolOutput::Udp { .. } => unreachable!(),
                    };
                    if let Some(f) = frame {
                        if should_send_vesc {
                            let _ = can_interface.send(&f);
                        }
                    }
                }
            }
        }

        // Build telemetry
        let vesc_states = state.drivetrain.states();
        let motor_temps: [f32; 4] = [
            vesc_states[0].status4.temp_motor,
            vesc_states[1].status4.temp_motor,
            vesc_states[2].status4.temp_motor,
            vesc_states[3].status4.temp_motor,
        ];
        let motor_currents: [f32; 4] = [
            vesc_states[0].status.current,
            vesc_states[1].status.current,
            vesc_states[2].status.current,
            vesc_states[3].status.current,
        ];
        // Wheel status: 0=offline, 1=degraded, 2=online
        // - Offline: FET temp <= 5C (not receiving data, default is 0.0)
        // - Degraded: FET temp > 5C but motor temp <= 5C (VESC online, motor disconnected)
        // - Online: FET temp > 5C and motor temp > 5C (fully operational)
        let wheel_status: [u8; 4] = std::array::from_fn(|i| {
            let fet_temp = vesc_states[i].status4.temp_fet;
            let motor_temp = vesc_states[i].status4.temp_motor;
            if fet_temp <= 5.0 {
                types::wheel_status::OFFLINE
            } else if motor_temp <= 5.0 {
                types::wheel_status::DEGRADED
            } else {
                types::wheel_status::ONLINE
            }
        });

        // Update wheel odometry from VESC tachometers
        // VESC "Invert Motor Direction" inverts both duty AND tachometer readings,
        // so right-side values already use the correct sign convention (positive = forward).
        let tach: [i32; 4] = [
            vesc_states[0].status5.tachometer,  // FL
            vesc_states[1].status5.tachometer,  // FR
            vesc_states[2].status5.tachometer,  // RL
            vesc_states[3].status5.tachometer,  // RR
        ];
        let (dx, dy, dtheta) = odometry.update(tach);
        let now_us = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);
        vel_tracker.update((dx * dx + dy * dy).sqrt(), dtheta, now_us);

        // Extract body-frame yaw rate from IMU (if available)
        let gyro_z_body: Option<f32> = imu_rx.borrow().as_ref().map(|imu| {
            (-imu.gyro_x as f64 * sin_mount_pitch + imu.gyro_z as f64 * cos_mount_pitch) as f32
        });

        // Feed body-frame yaw rate into IMU pre-integrator for SLAM
        if let Some(gz) = gyro_z_body {
            imu_preintegrator.integrate(gz as f64, dt as f64);
        }

        // Update EKF prediction with odometry + IMU yaw
        pose_estimator.predict(dx, dy, dtheta, gyro_z_body, dt);

        // Check for SLAM pose updates (non-blocking)
        // SLAM runs in background task, we just check for new state.
        // When new SLAM data arrives, feed it as a measurement update into the EKF.
        let slam_state: Option<SlamState> = if let Some(ref mut rx) = slam_state_rx_local {
            if rx.has_changed().unwrap_or(false) {
                let state = rx.borrow_and_update().clone();
                // Feed SLAM correction into EKF as measurement update
                pose_estimator.update_slam_from_array(&state.pose, &state.pose_covariance);
                last_slam_update = Some(Instant::now());
                Some(state)
            } else {
                Some(rx.borrow().clone())
            }
        } else {
            None
        };

        // Check for GPS updates
        if gps_rx.has_changed().unwrap_or(false) {
            let gps_state = gps_rx.borrow_and_update().clone();
            pose_estimator.update_gps(&gps_state);
            if let Some(ref coord) = gps_state.coord {
                last_gps_update = Some(Instant::now());
                debug!(
                    lat = coord.lat,
                    lon = coord.lon,
                    sats = gps_state.satellites,
                    fix = gps_state.fix_quality,
                    "GPS update"
                );
            }
        }

        // Check for LiDAR updates
        // Clone the scan OUTSIDE the shared lock to avoid blocking issues with rerun SDK
        let lidar_scan_clone = if lidar_rx_local.has_changed().unwrap_or(false) {
            let scan = lidar_rx_local.borrow_and_update().clone();
            if scan.is_some() {
                last_lidar_update = Some(Instant::now());
            }
            scan
        } else {
            None
        };

        // Process LiDAR scan (navigation costmap only - recorder moved to async task)
        if let Some(ref scan) = lidar_scan_clone {
            // Send scan to SLAM background task (non-blocking)
            if let Some(ref tx) = slam_scan_tx {
                // Consume pre-integrated IMU delta accumulated since last scan
                let imu_delta = imu_preintegrator.consume();
                let _ = tx.try_send((scan.clone(), pose_estimator.pose(), imu_delta));
            }

            // Update costmap for navigation controller
            let costmap_start = Instant::now();
            if let Some(ref mut nav) = navigation_controller {
                let robot_tf = Transform2D::from_pose(&pose_estimator.pose());
                nav.update_costmap(&scan, &robot_tf);
                // Publish costmap snapshot for console streaming
                let _ = costmap_tx.send(Some(nav.costmap_snapshot()));
                // Track obstacles for stable IDs and velocity
                let raw_obstacles = nav.extract_obstacles();
                let tracked = obstacle_tracker.update(&raw_obstacles, dt as f32);
                let _ = obstacles_tx.send(Some(tracked));
            }
            let costmap_ms = costmap_start.elapsed().as_millis();

            if costmap_ms > 50 {
                warn!(costmap_ms = costmap_ms, points = scan.points.len(), "Costmap processing slow");
            }

            // NOTE: recorder.log_lidar_scan() removed - was causing deadlock with rerun/rayon
            // TODO: Move to async recording task
        }

        // -----------------------------------------------------------------
        // Depth perception pipeline (camera-based obstacle detection + VO)
        // -----------------------------------------------------------------
        // Drain raw frames from each camera, run back-projection, merge into
        // costmap, and optionally run visual odometry for EKF prediction.
        if depth_enabled {
            let all_ground: Vec<nalgebra::Vector3<f64>> = Vec::new();
            let all_obstacles: Vec<nalgebra::Vector3<f64>> = Vec::new();

            for (cam_id, rx) in &depth_raw_rxs {
                // Drain to latest frame (non-blocking)
                let mut latest: Option<camera::RawFrame> = None;
                while let Ok(frame) = rx.try_recv() {
                    latest = Some(frame);
                }

                if let Some(frame) = latest {
                    // Find matching geometry for this camera
                    if let Some((_id, geom)) = depth_geometries.iter().find(|(id, _)| id == cam_id) {
                        // TODO: When onnx feature is enabled, run DepthEstimator here.
                        // For now, the depth pipeline is wired but inference requires
                        // the onnx feature + model file on disk. The backproject +
                        // costmap + VO path below is ready for when DepthMap is produced.
                        let _ = (frame, geom); // suppress unused warnings
                    }
                }
            }

            // If we got points from any camera, update costmap and run VO
            if !all_ground.is_empty() || !all_obstacles.is_empty() {
                // Update costmap
                if let Some(ref mut nav) = navigation_controller {
                    let p = pose_estimator.pose();
                    let sensor_pos = nalgebra::Vector2::new(p.x, p.y);
                    nav.update_costmap_from_points(sensor_pos, &all_ground, &all_obstacles);
                    let _ = costmap_tx.send(Some(nav.costmap_snapshot()));
                    let tracked = obstacle_tracker.update(&nav.extract_obstacles(), dt as f32);
                    let _ = obstacles_tx.send(Some(tracked));
                }

                // Visual odometry: depth-derived ego-motion → EKF
                if let Some(ref mut vo) = depth_visual_odometry {
                    let classified = depth::ClassifiedPoints {
                        ground: all_ground,
                        obstacles: all_obstacles,
                    };
                    let (vo_dx, vo_dy, vo_dtheta) = vo.update(&classified);

                    // Feed VO into EKF (same interface as wheel odometry)
                    if vo_dx.abs() > 1e-6 || vo_dy.abs() > 1e-6 || vo_dtheta.abs() > 1e-6 {
                        pose_estimator.predict(vo_dx, vo_dy, vo_dtheta, gyro_z_body, dt);
                        debug!(
                            vo_dx = format!("{vo_dx:.4}"),
                            vo_dy = format!("{vo_dy:.4}"),
                            vo_dtheta = format!("{vo_dtheta:.4}"),
                            "depth.vo"
                        );
                    }
                }
            }
        }

        let (active_tool, tool_status) = if let Some(tool) = state.tool_registry.active() {
            let status = tool.status();
            (
                Some(tool.info().name.to_string()),
                Some(teleop::ToolStatus {
                    name: status.name.to_string(),
                    position: status.position,
                    active: status.active,
                    current: status.current,
                }),
            )
        } else {
            (None, None)
        };

        // Get pose from EKF (unified: fuses odom + GPS + IMU + SLAM)
        // In sim mode, use simulation ground truth instead
        drop(state);
        let pose = if is_sim {
            can_interface.pose()
        } else {
            pose_estimator.pose()
        };

        // Build SLAM status if enabled
        let slam_status = slam_state.as_ref().map(|state| SlamStatus {
            pose,
            confidence: slam_state.as_ref().map(|s| s.match_score as f32).unwrap_or(0.0),
            keyframe_count: state.keyframe_count,
            loop_closure_count: state.loop_closure_count,
            mapping_active: true,
        });

        // Update health status based on current subsystem states
        let gps_state = gps_rx.borrow();
        let has_gps_fix = gps_state.coord.is_some() && gps_state.fix_quality > 0;
        drop(gps_state);

        let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());

        // Update health from available state
        state.health.can_healthy = state.drivetrain.battery_voltage() > 0.0;
        state.health.gps_fix = has_gps_fix;
        state.health.recording_active = recording_enabled;
        // Note: camera_active is set when camera starts successfully

        // Dispatch/discovery: use live connection state instead of config-only flag
        state.health.dispatch_connected = if dispatch_enabled { dispatch_connected_live } else { false };
        state.health.discovery_connected = discovery_enabled;

        // Subsystem liveness checks (every 1s to avoid per-tick overhead)
        if last_subsystem_check.elapsed() >= Duration::from_secs(1) {
            // GPS: stale if no update in 5s
            state.health.gps_fix = has_gps_fix
                && last_gps_update.map_or(false, |t| t.elapsed() < Duration::from_secs(5));

            // LiDAR: stale if no scan in 3s
            state.health.lidar_active = lidar_enabled
                && last_lidar_update.map_or(false, |t| t.elapsed() < Duration::from_secs(3));

            // SLAM: stale if no update in 10s
            state.health.slam_running = slam_scan_tx.is_some()
                && last_slam_update.map_or(false, |t| t.elapsed() < Duration::from_secs(10));

            last_subsystem_check = Instant::now();
        }

        // Wheel/VESC status
        state.health.wheel_status = wheel_status;

        // Increment telemetry sequence
        static TELEM_SEQ: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);
        let telem_seq = TELEM_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Get system metrics for telemetry (reuses cached values from sys_metrics)
        let sys = sys_metrics.metrics();

        // Compute roll/pitch from IMU using Kalman filter (fuses gyro + accelerometer)
        // The IMU is mounted tilted forward, so we transform readings to body frame first.
        let (roll, pitch) = if let Some(imu) = imu_rx.borrow().as_ref() {
            // Transform both accel and gyro from sensor frame to body frame.
            // Sensor is pitched forward by mounting_pitch_deg degrees, so we rotate back.
            let (sin_p, cos_p) = mount_pitch_rad.sin_cos();

            // Livox Mid360 IMU outputs acceleration in g's, convert to m/s^2
            const GRAVITY: f32 = 9.81;
            let accel_x_ms2 = imu.accel_x * GRAVITY;
            let accel_y_ms2 = imu.accel_y * GRAVITY;
            let accel_z_ms2 = imu.accel_z * GRAVITY;

            let accel = [
                accel_x_ms2 * cos_p + accel_z_ms2 * sin_p,
                accel_y_ms2,
                -accel_x_ms2 * sin_p + accel_z_ms2 * cos_p,
            ];
            let gyro = [
                imu.gyro_x * cos_p + imu.gyro_z * sin_p,
                imu.gyro_y,
                -imu.gyro_x * sin_p + imu.gyro_z * cos_p,
            ];

            let att = attitude_filter.update(gyro, accel, dt);
            (att.roll, att.pitch)
        } else {
            (0.0, 0.0)
        };

        // Read LiDAR status (polled every 10s by driver)
        let lidar_status = *lidar_status_rx.borrow();
        let lidar_core_temp_c = if lidar_status.responsive {
            Some(lidar_status.core_temp_c)
        } else {
            None
        };

        let telemetry = Telemetry {
            sequence: telem_seq,
            timestamp_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0),
            mode: state.state_machine.mode(),
            pose,
            power: PowerStatus {
                battery_voltage: state.drivetrain.battery_voltage() as f64,
                system_current: state.drivetrain.total_current() as f64,
            },
            cmd_velocity: twist,
            meas_velocity: (vel_tracker.linear() as f32, vel_tracker.angular() as f32),
            acceleration: (vel_tracker.linear_accel() as f32, vel_tracker.angular_accel() as f32),
            motor_temps,
            motor_currents,
            health: state.health,
            odometry_quality: vel_tracker.quality(),
            dt_ms: (dt * 1000.0) as f32,
            last_cmd_seq: seq_tracker.lock().map(|t| t.last_seq()).unwrap_or(0),
            ack_bits: seq_tracker.lock().map(|t| t.ack_bits()).unwrap_or(0),
            roll,
            pitch,
            cpu_percent: sys.cpu_percent as u8,
            mem_percent: sys.mem_percent as u8,
            disk_percent: sys.disk_percent as u8,
            active_tool,
            tool_status,
            slam_status,
            lidar_core_temp_c,
            lidar_work_state: lidar_status.work_state,
            mode_changed_at: state.state_machine.mode_changed_at_epoch_secs(),
            policy_intention,
            fault_code: state.fault_code as u8,
            active_tool_type: state.tool_registry.active_tool_type() as u8,
        };

        let _ = telemetry_tx.send(telemetry.clone());

        // Update metrics snapshot for Depot push
        // Refresh system metrics once per second (expensive operation)
        if last_sys_refresh.elapsed() >= Duration::from_secs(1) {
            sys_metrics.refresh();
            last_sys_refresh = Instant::now();
        }

        let gps_state = gps_rx.borrow();

        // Collect WebRTC channel metrics (take and reset counters)
        let webrtc_metrics = if let Some(ref rtc) = rtc_metrics {
            let (telem_sent, telem_us, telem_max) = rtc.telemetry.take();
            let (video_sent, video_us, video_max) = rtc.video.take();
            let (cloud_sent, cloud_us, cloud_max) = rtc.pointcloud.take();
            metrics::WebRtcChannelMetrics {
                telemetry_sent: telem_sent,
                telemetry_send_us: telem_us,
                telemetry_send_max_us: telem_max,
                video_sent,
                video_send_us: video_us,
                video_send_max_us: video_max,
                pointcloud_sent: cloud_sent,
                pointcloud_send_us: cloud_us,
                pointcloud_send_max_us: cloud_max,
            }
        } else {
            metrics::WebRtcChannelMetrics::default()
        };

        let metrics_snapshot = MetricsSnapshot {
            mode: telemetry.mode,
            battery_voltage: telemetry.power.battery_voltage,
            system_current: telemetry.power.system_current,
            motor_temps: telemetry.motor_temps,
            motor_currents: telemetry.motor_currents,
            velocity_linear: telemetry.cmd_velocity.linear,
            velocity_angular: telemetry.cmd_velocity.angular,
            gps_latitude: gps_state.coord.as_ref().map(|c| c.lat).unwrap_or(0.0),
            gps_longitude: gps_state.coord.as_ref().map(|c| c.lon).unwrap_or(0.0),
            gps_accuracy: gps_state.coord.as_ref().map(|c| c.accuracy).unwrap_or(0.0),
            system: sys_metrics.metrics(),
            webrtc: webrtc_metrics,
            lidar_core_temp: lidar_core_temp_c.unwrap_or(0.0),
            control_dt_ms: dt_last_ms,
            control_dt_max_ms: dt_max_ms,
            control_dt_jitter_ms: if dt_max_ms > dt_min_ms && dt_min_ms < f64::MAX {
                dt_max_ms - dt_min_ms
            } else {
                0.0
            },
            heartbeat_stalled: heartbeat_stalled.load(Ordering::Relaxed),
            can_error_count,
            can_consecutive_errors: consecutive_can_errors,
            can_backoff: can_backoff_active,
            ..Default::default()
        };
        drop(gps_state);
        let _ = metrics_tx.send(metrics_snapshot);

        // Reset timing min/max after metrics are captured (roughly every push cycle)
        // The metrics pusher reads at ~1Hz via watch channel; reset here at 100Hz
        // means we track max/jitter within the ~1s window between consecutive reads.
        // Reset only once per second to align with metrics push rate.
        if loop_count % 100 == 0 {
            dt_min_ms = f64::MAX;
            dt_max_ms = 0.0;
        }

        // Capture data needed for recording BEFORE releasing mutex
        // (rerun uses rayon internally, which deadlocks if called while holding a mutex)
        let commanded_twist = state.commanded_twist;
        let current_mode = telemetry.mode;
        let led_frame = if current_mode != last_mode {
            Some(state.state_machine.led_command().to_frame())
        } else {
            None
        };

        // CRITICAL: Release mutex before any recorder calls to avoid rayon deadlock
        // See docs/postmortem-2026-01-control-loop-deadlock.md
        drop(state);

        // Record telemetry to Rerun session (MUST be outside mutex)
        let recorder_start = Instant::now();
        let time_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        recorder.set_time(time_secs);

        // Log all telemetry data
        let _ = recorder.log_pose(&pose);
        let _ = recorder.log_velocity(&commanded_twist, &twist);
        let _ = recorder.log_motors(&motor_currents, &motor_temps);
        let _ = recorder.log_power(telemetry.power.battery_voltage, telemetry.power.system_current);
        let _ = recorder.log_odometry(dx, dy, dtheta);

        // Log SLAM data if enabled
        if let Some(ref slam) = slam_state {
            let _ = recorder.log_slam_trajectory(&pose);
            let _ = recorder.log_slam_keyframes(&slam.keyframe_poses);
            let _ = recorder.log_slam_status(
                slam.keyframe_count as usize,
                slam.loop_closure_count as usize,
                slam_state.as_ref().map(|s| s.match_score).unwrap_or(0.0),
            );
        }

        // Warn if recorder is slow
        let recorder_elapsed = recorder_start.elapsed();
        if recorder_elapsed > Duration::from_millis(50) {
            warn!(ms = recorder_elapsed.as_millis(), "Recorder logging slow");
        }

        // Log mode changes and update LEDs
        if current_mode != last_mode {
            // Reset sleep timer when entering Idle (restart countdown)
            if matches!(current_mode, Mode::Idle) {
                idle_since = Instant::now();
            }

            // Reset dance choreography timer when entering Dance
            if matches!(current_mode, Mode::Dance) {
                dance_start = Instant::now();
            }

            // Start/stop recording based on mode (only record during Teleop/Autonomous)
            let should_record = matches!(current_mode, Mode::Teleop | Mode::Autonomous | Mode::Dance);
            let was_recording = matches!(last_mode, Mode::Teleop | Mode::Autonomous | Mode::Dance);

            if should_record && !was_recording {
                // Entering a recording mode - start session
                if let Err(e) = recorder.start_session() {
                    warn!(?e, "Failed to start recording session");
                } else if let Some(path) = recorder.session_path() {
                    info!(path = %path.display(), mode = ?current_mode, "Recording session started");
                }

            } else if !should_record && was_recording {
                // Leaving a recording mode - end session
                recorder.end_session();
            }

            // Handle Sleep mode transitions
            if current_mode == Mode::Sleep && last_mode != Mode::Sleep {
                // Entering Sleep - stop lidar and end recording
                info!("Entering Sleep mode - shutting down sensors");
                recorder.end_session();
                if let Some(ref tx) = lidar_cmd_tx {
                    if let Err(e) = tx.try_send(LidarCommand::Stop) {
                        warn!(?e, "Failed to send lidar stop command for sleep");
                    } else {
                        info!("Lidar stopped (entering sleep mode)");
                    }
                }
            } else if last_mode == Mode::Sleep && current_mode != Mode::Sleep {
                // Waking from Sleep - restart lidar
                info!("Waking from Sleep mode - starting lidar");
                if let Some(ref tx) = lidar_cmd_tx {
                    if let Err(e) = tx.try_send(LidarCommand::Start) {
                        warn!(?e, "Failed to send lidar start command on wake");
                    } else {
                        info!("Lidar started (waking from sleep)");
                    }
                }
            }

            let _ = recorder.log_mode(current_mode);

            // Send LED command for new mode (frame was captured before mutex release)
            if let Some(frame) = led_frame {
                if let Err(e) = can_interface.send(&frame) {
                    debug!(?e, "Failed to send LED command");
                } else {
                    debug!(?current_mode, "LED command sent for mode change");
                }
            }

            // Broadcast state change to Ethernet-attached tool MCUs
            if let Some(ref mut eth) = eth_discovery {
                let mode_byte = current_mode as u8;
                let handle = tokio_handle.clone();
                // Can't await in sync loop, so fire-and-forget via tokio
                // We need to move the mutable reference, but can't across spawn.
                // Instead, use the shared state to get MCU addresses and send directly.
                let state_clone = eth.state();
                handle.spawn(async move {
                    let targets: Vec<std::net::SocketAddr> = {
                        let s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
                        s.mcus.values()
                            .filter(|m| m.online)
                            .map(|m| m.command_addr())
                            .collect()
                    };
                    if !targets.is_empty() {
                        let packet = tools::eth_protocol::build_set_state(mode_byte, 0);
                        if let Ok(socket) = tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                            for addr in targets {
                                let _ = socket.send_to(&packet, addr).await;
                            }
                        }
                    }
                });
            }

            last_mode = current_mode;
        }

        // Log tool state if active
        if let Some(ref status) = telemetry.tool_status {
            let _ = recorder.log_tool(&status.name, status.position, status.current);
        }

    } // end of loop
}
