//! Operator station for controlling BVR.
//!
//! A Bevy-based 3D interface with Xbox controller support.

use bevy::prelude::*;
use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
use bevy::input::mouse::MouseMotion;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use clap::Parser;
use image::ImageReader;
use std::f32::consts::{FRAC_PI_2, PI};
use std::io::Cursor;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use teleop::video::FrameReassembler;
use types::{Mode, Twist};

#[derive(Parser)]
#[command(name = "operator", about = "BVR Operator Station")]
struct Args {
    /// Rover address (host:port)
    #[arg(short, long, default_value = "127.0.0.1:4840")]
    rover: String,

    /// Local port for receiving telemetry
    #[arg(short, long, default_value = "4841")]
    local_port: u16,

    /// Video port on rover
    #[arg(long, default_value = "4842")]
    video_port: u16,

    /// Local port for receiving video
    #[arg(long, default_value = "4843")]
    video_local_port: u16,
}

// ============================================================================
// Resources
// ============================================================================

/// Network connection to rover
#[derive(Resource)]
struct RoverConnection {
    socket: UdpSocket,
    rover_addr: SocketAddr,
    last_send: Instant,
}

/// Raw pose from telemetry (before interpolation)
#[derive(Default, Clone, Copy)]
struct TelemetryPose {
    x: f32,
    y: f32,
    theta: f32,
}

/// Received telemetry from rover
#[derive(Resource, Default)]
struct Telemetry {
    mode: Mode,
    /// Raw rover pose from bvrd (actual position, not dead-reckoning)
    raw_pose: TelemetryPose,
    /// Flag set when new pose data arrives
    pose_updated: bool,
    battery_voltage: f64,
    system_current: f64,
    velocity: Twist,
    connected: bool,
    last_recv: Option<Instant>,
}

/// Decoded video frame from background thread
struct DecodedFrame {
    rgba_data: Vec<u8>,
    width: u32,
    height: u32,
    sequence: u32,
}

/// Video receiver (receives decoded frames from background thread)
#[derive(Resource)]
struct VideoReceiver {
    rx: Mutex<mpsc::Receiver<DecodedFrame>>,
}

/// Current video frame for display
#[derive(Resource, Default)]
struct VideoDisplay {
    /// Decoded RGBA image data
    rgba_data: Vec<u8>,
    width: u32,
    height: u32,
    /// Egui texture handle
    texture_handle: Option<egui::TextureHandle>,
    /// Last frame sequence number (from video thread)
    last_sequence: u32,
    /// Sequence number of last texture update (for egui)
    last_texture_sequence: u32,
    /// Frames per second (rolling average)
    fps: f32,
    last_frame_time: Option<Instant>,
}

/// Spawn background thread for video receiving and decoding.
fn spawn_video_thread(
    video_addr: SocketAddr,
    local_port: u16,
) -> std::io::Result<mpsc::Receiver<DecodedFrame>> {
    let (tx, rx) = mpsc::channel();

    let local_addr: SocketAddr = format!("0.0.0.0:{}", local_port).parse().unwrap();
    let socket = UdpSocket::bind(local_addr)?;
    socket.set_nonblocking(true)?;

    std::thread::spawn(move || {
        // Register with video server
        let _ = socket.send_to(&[0x00], video_addr);

        let mut reassembler = FrameReassembler::new(Duration::from_millis(200));
        let mut buf = [0u8; 2048];

        loop {
            // Receive packets (non-blocking, then sleep to avoid busy loop)
            let mut received_any = false;
            while let Ok((len, _addr)) = socket.recv_from(&mut buf) {
                received_any = true;
                if let Some(frame) = reassembler.process(&buf[..len]) {
                    // Decode JPEG to RGBA (this is the expensive part - done in background)
                    if let Some(img) = ImageReader::new(Cursor::new(&frame.data))
                        .with_guessed_format()
                        .ok()
                        .and_then(|r| r.decode().ok())
                    {
                        let rgba = img.to_rgba8();
                        let decoded = DecodedFrame {
                            rgba_data: rgba.to_vec(),
                            width: frame.width,
                            height: frame.height,
                            sequence: frame.sequence,
                        };
                        if tx.send(decoded).is_err() {
                            // Main thread dropped receiver, exit
                            return;
                        }
                    }
                }
            }

            // Re-register periodically in case server restarted
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER % 100 == 0 {
                    let _ = socket.send_to(&[0x00], video_addr);
                }
            }

            // Sleep briefly if no data to avoid busy loop
            if !received_any {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    });

    Ok(rx)
}

/// Input source type
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum InputSource {
    #[default]
    None,
    Keyboard,
    Gamepad,
}

/// Camera view mode
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum CameraMode {
    #[default]
    ThirdPerson,
    FirstPerson,
    FreeLook,
}

/// Controller input state
#[derive(Resource, Default)]
struct ControllerInput {
    linear: f32,
    angular: f32,
    tool_axis: f32,
    // Camera control (right stick / mouse)
    camera_yaw: f32,
    camera_pitch: f32,
    action_a: bool,
    action_b: bool,
    estop: bool,
    enable: bool,
    source: InputSource,
}

/// Camera state for orbit/follow behavior
#[derive(Resource)]
struct CameraState {
    /// Horizontal angle offset from behind rover (radians)
    yaw_offset: f32,
    /// Vertical angle (radians, 0 = horizontal, positive = looking down)
    pitch: f32,
    /// Distance from rover
    distance: f32,
    /// Camera mode
    mode: CameraMode,
    /// Time since last manual camera input (for auto-reset)
    last_input: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            yaw_offset: 0.0,
            pitch: 0.4, // Slightly looking down
            distance: 3.5,
            mode: CameraMode::ThirdPerson,
            last_input: 0.0,
        }
    }
}

/// Rover pose with interpolation for smooth rendering
#[derive(Resource)]
struct RoverPose {
    // Current interpolated pose (rendered)
    x: f32,
    y: f32,
    theta: f32,
    // Target pose from latest telemetry
    target_x: f32,
    target_y: f32,
    target_theta: f32,
    // Previous pose for interpolation
    prev_x: f32,
    prev_y: f32,
    prev_theta: f32,
    // Interpolation progress (0.0 to 1.0)
    interp_t: f32,
}

impl Default for RoverPose {
    fn default() -> Self {
        Self {
            x: 0.0, y: 0.0, theta: 0.0,
            target_x: 0.0, target_y: 0.0, target_theta: 0.0,
            prev_x: 0.0, prev_y: 0.0, prev_theta: 0.0,
            interp_t: 1.0,
        }
    }
}

impl RoverPose {
    /// Update target from new telemetry
    fn set_target(&mut self, x: f32, y: f32, theta: f32) {
        // Current becomes previous
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.prev_theta = self.theta;
        // New target
        self.target_x = x;
        self.target_y = y;
        self.target_theta = theta;
        // Reset interpolation
        self.interp_t = 0.0;
    }

    /// Interpolate toward target (call each frame)
    fn interpolate(&mut self, dt: f32) {
        // Interpolate over ~100ms (telemetry interval)
        const INTERP_DURATION: f32 = 0.1;
        self.interp_t = (self.interp_t + dt / INTERP_DURATION).min(1.0);

        // Smooth step for nicer easing
        let t = smooth_step(self.interp_t);

        self.x = lerp(self.prev_x, self.target_x, t);
        self.y = lerp(self.prev_y, self.target_y, t);
        // Use angle lerp for theta to handle wraparound
        self.theta = lerp_angle(self.prev_theta, self.target_theta, t);
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    // Find shortest path around the circle
    let mut diff = b - a;
    while diff > std::f32::consts::PI { diff -= std::f32::consts::TAU; }
    while diff < -std::f32::consts::PI { diff += std::f32::consts::TAU; }
    a + diff * t
}

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
struct RoverModel;

#[derive(Component)]
struct VideoFrustum;

/// Video texture for 3D frustum display
#[derive(Resource)]
struct VideoTexture {
    /// Handle to the material using this texture (we update its texture reference)
    material_handle: Handle<StandardMaterial>,
    /// Last sequence number we uploaded
    last_uploaded_seq: u32,
}

// ============================================================================
// Startup Systems
// ============================================================================

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.15, 0.18, 0.15))),
    ));

    // Grid lines on ground
    for i in -25..=25 {
        let color = if i == 0 {
            Color::srgb(0.4, 0.4, 0.4)
        } else if i % 5 == 0 {
            Color::srgb(0.25, 0.28, 0.25)
        } else {
            Color::srgb(0.18, 0.21, 0.18)
        };

        // X lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(50.0, 0.005, 0.015))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.001, i as f32),
        ));

        // Z lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.015, 0.005, 50.0))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(i as f32, 0.001, 0.0),
        ));
    }

    // Rover body
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, 0.2, 0.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.4, 0.7),
            metallic: 0.3,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.15, 0.0),
        RoverModel,
    )).with_children(|parent| {
        // Front indicator (red)
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.15, 0.08, 0.08))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.2, 0.2),
                emissive: LinearRgba::new(0.5, 0.1, 0.1, 1.0),
                ..default()
            })),
            Transform::from_xyz(0.28, 0.0, 0.0),
        ));

        // Top panel
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.4, 0.02, 0.4))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.1, 0.1),
                metallic: 0.9,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.11, 0.0),
        ));

        // Wheels - rotate around X so axles point sideways (Z axis)
        let wheel_mesh = meshes.add(Cylinder::new(0.082, 0.06));
        let wheel_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.05),
            perceptual_roughness: 0.9,
            ..default()
        });

        for (x, z) in [(-0.28, 0.28), (0.28, 0.28), (-0.28, -0.28), (0.28, -0.28)] {
            parent.spawn((
                Mesh3d(wheel_mesh.clone()),
                MeshMaterial3d(wheel_mat.clone()),
                Transform::from_xyz(x, -0.07, z)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ));
        }
    });

    // Create material for video (texture will be set dynamically)
    let video_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.15), // Dark placeholder color
        unlit: true, // Don't apply lighting to video
        alpha_mode: AlphaMode::Opaque,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    // Frustum dimensions (based on camera FOV and distance)
    let frustum_distance = 3.0; // Distance from rover to video plane
    let frustum_width = 2.4;    // Width of video plane (roughly 16:9 aspect)
    let frustum_height = 1.35;  // Height of video plane

    // Create a proper quad mesh facing backward (-X direction when placed at +X)
    // Using Rectangle which creates a mesh in XY plane, then we rotate it
    let quad_mesh = Rectangle::new(frustum_width, frustum_height);

    // Video frustum quad - positioned in front of rover
    commands.spawn((
        Mesh3d(meshes.add(quad_mesh)),
        MeshMaterial3d(video_material.clone()),
        Transform::from_xyz(frustum_distance, 0.4, 0.0)
            .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)), // Rotate to face back toward rover
        VideoFrustum,
    ));

    // Store video texture handles as resource
    commands.insert_resource(VideoTexture {
        material_handle: video_material,
        last_uploaded_seq: 0,
    });

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3.0, 2.5, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Sun light
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });
}

// ============================================================================
// Update Systems
// ============================================================================

fn gamepad_connections(
    mut events: EventReader<GamepadConnectionEvent>,
) {
    for event in events.read() {
        match &event.connection {
            GamepadConnection::Connected { name, .. } => {
                info!("Gamepad connected: {}", name);
            }
            GamepadConnection::Disconnected => {
                info!("Gamepad disconnected");
            }
        }
    }
}

fn read_input(
    mut input: ResMut<ControllerInput>,
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    // Reset camera input each frame
    input.camera_yaw = 0.0;
    input.camera_pitch = 0.0;

    // Try gamepad first
    if let Some(gamepad) = gamepads.iter().next() {
        input.source = InputSource::Gamepad;

        // Left stick for movement
        let left_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        let left_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);

        // Apply deadzone
        // Stick up = forward (positive linear), stick right = turn right (negative angular)
        input.linear = if left_y.abs() > 0.1 { left_y } else { 0.0 };
        input.angular = if left_x.abs() > 0.1 { -left_x } else { 0.0 };

        // Right stick for camera
        let right_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let right_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);
        if right_x.abs() > 0.1 {
            input.camera_yaw = right_x * 2.0;
        }
        if right_y.abs() > 0.1 {
            input.camera_pitch = right_y * 1.5;
        }

        // Triggers (LT/RT) for tool axis
        // Try analog axis first, fall back to button state
        let rt_axis = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0);
        let lt_axis = gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0);
        let rt_btn = if gamepad.pressed(GamepadButton::RightTrigger2) { 1.0 } else { 0.0 };
        let lt_btn = if gamepad.pressed(GamepadButton::LeftTrigger2) { 1.0 } else { 0.0 };
        // Use whichever has a value
        let rt = if rt_axis.abs() > 0.1 { rt_axis } else { rt_btn };
        let lt = if lt_axis.abs() > 0.1 { lt_axis } else { lt_btn };
        input.tool_axis = rt - lt;

        // Buttons
        // A or RB = primary action
        input.action_a = gamepad.pressed(GamepadButton::South)
            || gamepad.pressed(GamepadButton::RightTrigger);
        // B or LB = secondary action
        input.action_b = gamepad.pressed(GamepadButton::East)
            || gamepad.pressed(GamepadButton::LeftTrigger);
        input.estop = gamepad.pressed(GamepadButton::Select);
        input.enable = gamepad.pressed(GamepadButton::Start);
        return;
    }

    // Fall back to keyboard + mouse
    // WASD or Arrow keys for movement
    let mut linear = 0.0f32;
    let mut angular = 0.0f32;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        linear += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        linear -= 1.0;
    }
    // A = turn left = positive angular velocity (counter-clockwise)
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        angular += 1.0;
    }
    // D = turn right = negative angular velocity (clockwise)
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        angular -= 1.0;
    }

    // Q/E for tool axis
    let mut tool = 0.0f32;
    if keyboard.pressed(KeyCode::KeyE) {
        tool += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        tool -= 1.0;
    }

    // Mouse for camera (when right button held)
    if mouse_buttons.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            input.camera_yaw += ev.delta.x * 0.1;
            input.camera_pitch -= ev.delta.y * 0.1;
        }
    } else {
        // Consume events even if not using them
        mouse_motion.clear();
    }

    // Check if any movement keys are pressed to determine source
    let any_key = linear != 0.0 || angular != 0.0 || tool != 0.0
        || keyboard.pressed(KeyCode::Space)
        || keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::Escape)
        || keyboard.pressed(KeyCode::Enter);

    if any_key {
        input.source = InputSource::Keyboard;
    } else if input.source == InputSource::Gamepad {
        // No gamepad connected anymore, no keys pressed
        input.source = InputSource::None;
    }

    input.linear = linear;
    input.angular = angular;
    input.tool_axis = tool;

    // Space = Action A, Shift = Action B
    input.action_a = keyboard.pressed(KeyCode::Space);
    input.action_b = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Escape = E-Stop, Enter = Enable
    input.estop = keyboard.pressed(KeyCode::Escape);
    input.enable = keyboard.pressed(KeyCode::Enter);
}

fn send_commands(
    input: Res<ControllerInput>,
    mut connection: ResMut<RoverConnection>,
) {
    // Send at ~50Hz
    if connection.last_send.elapsed() < Duration::from_millis(20) {
        return;
    }
    connection.last_send = Instant::now();

    // E-Stop takes priority
    if input.estop {
        let buf = [0x02u8]; // E-Stop command
        let _ = connection.socket.send_to(&buf, connection.rover_addr);
        return;
    }

    // Build command
    let linear = input.linear as f64 * 2.0; // Max 2 m/s
    let angular = input.angular as f64 * 1.5; // Max 1.5 rad/s

    // Send twist command
    let mut buf = Vec::with_capacity(17);
    buf.push(0x01); // Twist command type
    buf.extend_from_slice(&linear.to_le_bytes());
    buf.extend_from_slice(&angular.to_le_bytes());

    let _ = connection.socket.send_to(&buf, connection.rover_addr);
}

fn receive_telemetry(
    connection: Res<RoverConnection>,
    mut telemetry: ResMut<Telemetry>,
) {
    let mut buf = [0u8; 256];

    // Non-blocking receive
    while let Ok((len, _addr)) = connection.socket.recv_from(&mut buf) {
        if len > 0 && buf[0] == 0x10 {
            // Parse telemetry - format matches teleop::serialize_telemetry
            telemetry.connected = true;
            telemetry.last_recv = Some(Instant::now());

            // Mode (byte 1)
            if len >= 2 {
                telemetry.mode = match buf[1] {
                    0 => Mode::Disabled,
                    1 => Mode::Idle,
                    2 => Mode::Teleop,
                    3 => Mode::Autonomous,
                    4 => Mode::EStop,
                    5 => Mode::Fault,
                    _ => Mode::Disabled,
                };
            }

            // Pose: x, y, theta (bytes 2-25, three f64s)
            if len >= 26 {
                let x = f64::from_le_bytes(buf[2..10].try_into().unwrap_or([0; 8]));
                let y = f64::from_le_bytes(buf[10..18].try_into().unwrap_or([0; 8]));
                let theta = f64::from_le_bytes(buf[18..26].try_into().unwrap_or([0; 8]));
                telemetry.raw_pose = TelemetryPose {
                    x: x as f32,
                    y: y as f32,
                    theta: theta as f32,
                };
                telemetry.pose_updated = true;
            }

            // Battery voltage (bytes 26-33)
            if len >= 34 {
                telemetry.battery_voltage = f64::from_le_bytes(
                    buf[26..34].try_into().unwrap_or([0; 8])
                );
            }

            // Velocity (bytes 42-57, after timestamp)
            if len >= 58 {
                telemetry.velocity.linear = f64::from_le_bytes(
                    buf[42..50].try_into().unwrap_or([0; 8])
                );
                telemetry.velocity.angular = f64::from_le_bytes(
                    buf[50..58].try_into().unwrap_or([0; 8])
                );
            }
        }
    }

    // Check for timeout
    if let Some(last) = telemetry.last_recv {
        if last.elapsed() > Duration::from_secs(2) {
            telemetry.connected = false;
        }
    }
}

fn receive_video(
    video_rx: Res<VideoReceiver>,
    mut video_display: ResMut<VideoDisplay>,
) {
    // Pick up any decoded frames from the background thread (fast - no decoding here)
    let rx = video_rx.rx.lock().unwrap();
    while let Ok(frame) = rx.try_recv() {
        video_display.rgba_data = frame.rgba_data;
        video_display.width = frame.width;
        video_display.height = frame.height;
        video_display.last_sequence = frame.sequence;
        // Don't clear texture_handle here - let UI update it when ready
        // This prevents flickering when frames arrive faster than UI renders

        // Update FPS
        let now = Instant::now();
        if let Some(last) = video_display.last_frame_time {
            let dt = last.elapsed().as_secs_f32();
            if dt > 0.0 {
                let instant_fps = 1.0 / dt;
                video_display.fps = video_display.fps * 0.9 + instant_fps * 0.1;
            }
        }
        video_display.last_frame_time = Some(now);
    }
}

fn update_rover_pose(
    mut telemetry: ResMut<Telemetry>,
    mut pose: ResMut<RoverPose>,
    time: Res<Time>,
) {
    // When new telemetry arrives, set it as the interpolation target
    if telemetry.connected && telemetry.pose_updated {
        pose.set_target(
            telemetry.raw_pose.x,
            telemetry.raw_pose.y,
            telemetry.raw_pose.theta,
        );
        telemetry.pose_updated = false;
    }

    // Smoothly interpolate toward target each frame
    pose.interpolate(time.delta_secs());
}

fn update_rover_model(
    pose: Res<RoverPose>,
    mut query: Query<&mut Transform, With<RoverModel>>,
) {
    for mut transform in &mut query {
        // Map physics 2D coordinates to Bevy 3D:
        // physics.x → Bevy X
        // physics.y → Bevy -Z (2D Y-up maps to 3D Z-back)
        transform.translation.x = pose.x;
        transform.translation.z = -pose.y;
        // Positive theta = counter-clockwise in 2D = counter-clockwise around Y in 3D
        transform.rotation = Quat::from_rotation_y(pose.theta);
    }
}

fn update_video_frustum(
    pose: Res<RoverPose>,
    video_display: Res<VideoDisplay>,
    video_texture: Option<ResMut<VideoTexture>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut frustum_query: Query<&mut Transform, With<VideoFrustum>>,
) {
    let Some(mut video_texture) = video_texture else { return };

    // Update frustum position to follow rover
    let frustum_distance = 3.0;
    let frustum_height_offset = 0.4;

    for mut transform in &mut frustum_query {
        // Position in front of rover
        let forward = Vec3::new(pose.theta.cos(), 0.0, -pose.theta.sin());
        let rover_pos = Vec3::new(pose.x, 0.0, -pose.y);

        transform.translation = rover_pos + forward * frustum_distance + Vec3::Y * frustum_height_offset;
        // Rotate to face the rover - the quad is in XY plane, rotated -90° around Y
        // So we add that base rotation to the rover's heading
        transform.rotation = Quat::from_rotation_y(pose.theta - FRAC_PI_2);
    }

    // Update texture if we have new video data
    if video_display.width > 0
        && !video_display.rgba_data.is_empty()
        && video_display.last_sequence != video_texture.last_uploaded_seq
    {
        // Create new image from video data (no flip needed)
        let size = bevy::render::render_resource::Extent3d {
            width: video_display.width,
            height: video_display.height,
            depth_or_array_layers: 1,
        };

        let image = Image::new(
            size,
            bevy::render::render_resource::TextureDimension::D2,
            video_display.rgba_data.clone(),
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD
                | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );

        let image_handle = images.add(image);

        // Update material to use new texture
        if let Some(material) = materials.get_mut(&video_texture.material_handle) {
            material.base_color_texture = Some(image_handle);
            material.base_color = Color::WHITE; // Show texture, not tint
        }

        video_texture.last_uploaded_seq = video_display.last_sequence;
    }
}

fn draw_frustum_gizmos(
    pose: Res<RoverPose>,
    mut gizmos: Gizmos,
) {
    // Frustum parameters (must match setup_scene)
    let frustum_distance = 3.0;
    let frustum_width = 2.4;
    let frustum_height = 1.35;
    let camera_height = 0.25; // Height of camera on rover
    let frustum_center_height = 0.4;

    // Rover position and orientation in 3D
    let rover_pos = Vec3::new(pose.x, 0.0, -pose.y);
    let forward = Vec3::new(pose.theta.cos(), 0.0, -pose.theta.sin());
    let right = Vec3::new(-pose.theta.sin(), 0.0, -pose.theta.cos());

    // Camera origin (on top of rover)
    let camera_origin = rover_pos + Vec3::Y * camera_height;

    // Frustum plane center
    let plane_center = rover_pos + forward * frustum_distance + Vec3::Y * frustum_center_height;

    // Frustum corners (in world space)
    let half_w = frustum_width / 2.0;
    let half_h = frustum_height / 2.0;

    let top_left = plane_center + Vec3::Y * half_h - right * half_w;
    let top_right = plane_center + Vec3::Y * half_h + right * half_w;
    let bottom_left = plane_center - Vec3::Y * half_h - right * half_w;
    let bottom_right = plane_center - Vec3::Y * half_h + right * half_w;

    // Frustum edge color
    let edge_color = Color::srgba(0.3, 0.6, 1.0, 0.6);
    let corner_color = Color::srgba(0.3, 0.6, 1.0, 0.3);

    // Draw lines from camera to corners
    gizmos.line(camera_origin, top_left, corner_color);
    gizmos.line(camera_origin, top_right, corner_color);
    gizmos.line(camera_origin, bottom_left, corner_color);
    gizmos.line(camera_origin, bottom_right, corner_color);

    // Draw rectangle around video plane
    gizmos.line(top_left, top_right, edge_color);
    gizmos.line(top_right, bottom_right, edge_color);
    gizmos.line(bottom_right, bottom_left, edge_color);
    gizmos.line(bottom_left, top_left, edge_color);
}

fn update_camera(
    pose: Res<RoverPose>,
    input: Res<ControllerInput>,
    mut camera_state: ResMut<CameraState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Toggle camera mode with C or right stick click
    if keyboard.just_pressed(KeyCode::KeyC) {
        camera_state.mode = match camera_state.mode {
            CameraMode::ThirdPerson => CameraMode::FirstPerson,
            CameraMode::FirstPerson => CameraMode::ThirdPerson,
            CameraMode::FreeLook => CameraMode::ThirdPerson,
        };
    }

    // Toggle free look with V
    if keyboard.just_pressed(KeyCode::KeyV) {
        camera_state.mode = match camera_state.mode {
            CameraMode::FreeLook => CameraMode::ThirdPerson,
            _ => CameraMode::FreeLook,
        };
    }

    // Apply camera input
    let has_camera_input = input.camera_yaw.abs() > 0.01 || input.camera_pitch.abs() > 0.01;
    if has_camera_input {
        camera_state.yaw_offset += input.camera_yaw * dt;
        camera_state.pitch += input.camera_pitch * dt;
        camera_state.pitch = camera_state.pitch.clamp(0.1, 1.4); // Limit vertical angle
        camera_state.last_input = 0.0;
    } else {
        camera_state.last_input += dt;
    }

    // Scroll wheel for zoom (in third person)
    // (Would need mouse wheel event reader - skip for now)

    // Auto-reset camera behind rover when no camera input
    if !has_camera_input && camera_state.mode == CameraMode::ThirdPerson {
        // Smoothly return to behind rover
        let return_speed = if input.linear.abs() > 0.1 || input.angular.abs() > 0.1 {
            4.0 // Faster return when moving
        } else {
            2.0 // Slower return when stopped
        };
        camera_state.yaw_offset = camera_state.yaw_offset * (1.0 - return_speed * dt);
    }

    // Wrap yaw offset
    if camera_state.yaw_offset > PI {
        camera_state.yaw_offset -= 2.0 * PI;
    } else if camera_state.yaw_offset < -PI {
        camera_state.yaw_offset += 2.0 * PI;
    }

    for mut transform in &mut query {
        // Map physics coords to Bevy: physics.y → -Z
        let rover_pos = Vec3::new(pose.x, 0.15, -pose.y);

        match camera_state.mode {
            CameraMode::FirstPerson => {
                // Camera at rover position, looking forward
                // In physics: forward = (cos(theta), sin(theta))
                // In Bevy: forward = (cos(theta), 0, -sin(theta))
                let forward = Vec3::new(pose.theta.cos(), 0.0, -pose.theta.sin());
                transform.translation = rover_pos + Vec3::Y * 0.3 + forward * 0.2;
                let look_target = transform.translation + forward;
                transform.look_at(look_target, Vec3::Y);
            }
            CameraMode::ThirdPerson => {
                // Camera orbits behind rover
                let total_yaw = pose.theta + camera_state.yaw_offset + PI; // +PI to be behind
                let horizontal_dist = camera_state.distance * camera_state.pitch.cos();
                let height = camera_state.distance * camera_state.pitch.sin();

                let target_pos = Vec3::new(
                    pose.x + horizontal_dist * total_yaw.cos(),
                    height + 0.3,
                    -pose.y - horizontal_dist * total_yaw.sin(),
                );

                // Smooth follow
                transform.translation = transform.translation.lerp(target_pos, 8.0 * dt);
                transform.look_at(rover_pos, Vec3::Y);
            }
            CameraMode::FreeLook => {
                // Free orbit around rover (doesn't follow rover rotation)
                let horizontal_dist = camera_state.distance * camera_state.pitch.cos();
                let height = camera_state.distance * camera_state.pitch.sin();

                let target_pos = Vec3::new(
                    pose.x + horizontal_dist * camera_state.yaw_offset.cos(),
                    height + 0.3,
                    -pose.y - horizontal_dist * camera_state.yaw_offset.sin(),
                );

                transform.translation = transform.translation.lerp(target_pos, 8.0 * dt);
                transform.look_at(rover_pos, Vec3::Y);
            }
        }
    }
}

// ============================================================================
// UI System
// ============================================================================

fn ui_system(
    mut contexts: EguiContexts,
    telemetry: Res<Telemetry>,
    input: Res<ControllerInput>,
    pose: Res<RoverPose>,
    camera_state: Res<CameraState>,
    mut video_display: ResMut<VideoDisplay>,
) {
    // Telemetry panel
    egui::Window::new("📡 Telemetry")
        .default_pos([10.0, 10.0])
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            // Connection status
            ui.horizontal(|ui| {
                let (color, text) = if telemetry.connected {
                    (egui::Color32::from_rgb(80, 200, 120), "● Connected")
                } else {
                    (egui::Color32::from_rgb(200, 80, 80), "● Disconnected")
                };
                ui.colored_label(color, text);
            });

            ui.separator();

            // Mode
            let mode_info = match telemetry.mode {
                Mode::Disabled => ("DISABLED", egui::Color32::GRAY),
                Mode::Idle => ("IDLE", egui::Color32::from_rgb(100, 165, 255)),
                Mode::Teleop => ("TELEOP", egui::Color32::from_rgb(80, 200, 120)),
                Mode::Autonomous => ("AUTO", egui::Color32::from_rgb(180, 130, 255)),
                Mode::EStop => ("E-STOP", egui::Color32::from_rgb(255, 80, 80)),
                Mode::Fault => ("FAULT", egui::Color32::from_rgb(255, 180, 80)),
            };

            ui.horizontal(|ui| {
                ui.label("Mode:");
                ui.colored_label(mode_info.1, egui::RichText::new(mode_info.0).strong());
            });

            ui.separator();

            // Battery
            ui.horizontal(|ui| {
                ui.label("Battery:");
                let voltage = telemetry.battery_voltage;
                let color = if voltage > 45.0 {
                    egui::Color32::from_rgb(80, 200, 120)
                } else if voltage > 42.0 {
                    egui::Color32::from_rgb(255, 180, 80)
                } else {
                    egui::Color32::from_rgb(255, 80, 80)
                };
                ui.colored_label(color, format!("{:.1}V", voltage));
            });

            ui.horizontal(|ui| {
                ui.label("Current:");
                ui.label(format!("{:.1}A", telemetry.system_current));
            });

            ui.separator();

            // Velocity
            ui.label(egui::RichText::new("Velocity").strong());
            ui.horizontal(|ui| {
                ui.label("Linear:");
                ui.label(format!("{:.2} m/s", telemetry.velocity.linear));
            });
            ui.horizontal(|ui| {
                ui.label("Angular:");
                ui.label(format!("{:.2} rad/s", telemetry.velocity.angular));
            });
        });

    // Controller panel
    egui::Window::new("🎮 Input")
        .default_pos([10.0, 280.0])
        .default_width(220.0)
        .show(contexts.ctx_mut(), |ui| {
            // Input source
            ui.horizontal(|ui| {
                let (color, text) = match input.source {
                    InputSource::Gamepad => (egui::Color32::from_rgb(80, 200, 120), "● Gamepad"),
                    InputSource::Keyboard => (egui::Color32::from_rgb(100, 165, 255), "● Keyboard"),
                    InputSource::None => (egui::Color32::from_rgb(255, 180, 80), "○ No Input"),
                };
                ui.colored_label(color, text);
            });

            ui.separator();

            // Stick values with visual bars
            ui.horizontal(|ui| {
                ui.label("Linear: ");
                let bar_val = (input.linear + 1.0) / 2.0;
                ui.add(egui::ProgressBar::new(bar_val).desired_width(80.0));
                ui.label(format!("{:+.2}", input.linear));
            });

            ui.horizontal(|ui| {
                ui.label("Angular:");
                let bar_val = (input.angular + 1.0) / 2.0;
                ui.add(egui::ProgressBar::new(bar_val).desired_width(80.0));
                ui.label(format!("{:+.2}", input.angular));
            });

            ui.horizontal(|ui| {
                ui.label("Tool:   ");
                let bar_val = (input.tool_axis + 1.0) / 2.0;
                ui.add(egui::ProgressBar::new(bar_val).desired_width(80.0));
                ui.label(format!("{:+.2}", input.tool_axis));
            });

            ui.separator();

            // Buttons
            ui.horizontal(|ui| {
                let btn = |pressed: bool, label: &str| {
                    if pressed {
                        egui::RichText::new(label).color(egui::Color32::from_rgb(80, 200, 120)).strong()
                    } else {
                        egui::RichText::new(label).color(egui::Color32::GRAY)
                    }
                };

                ui.label(btn(input.action_a, "[A]"));
                ui.label(btn(input.action_b, "[B]"));
                ui.label(btn(input.estop, "[STOP]"));
                ui.label(btn(input.enable, "[EN]"));
            });
        });

    // Position panel
    egui::Window::new("📍 Position")
        .default_pos([10.0, 480.0])
        .default_width(150.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("X: {:+.2} m", pose.x));
            ui.label(format!("Y: {:+.2} m", pose.y));
            ui.label(format!("θ: {:+.1}°", pose.theta.to_degrees()));

            ui.separator();

            let cam_mode = match camera_state.mode {
                CameraMode::ThirdPerson => "3rd Person",
                CameraMode::FirstPerson => "1st Person",
                CameraMode::FreeLook => "Free Look",
            };
            ui.horizontal(|ui| {
                ui.label("Camera:");
                ui.label(egui::RichText::new(cam_mode).strong());
            });
            ui.label(egui::RichText::new("C: toggle  V: free").small().weak());
        });

    // Video feed window
    egui::Window::new("📹 Camera")
        .default_pos([220.0, 10.0])
        .default_width(320.0)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            let has_video = video_display.width > 0 && !video_display.rgba_data.is_empty();

            if has_video {
                // Extract values we need before mutably borrowing texture_handle
                let width = video_display.width;
                let height = video_display.height;
                let fps = video_display.fps;
                let seq = video_display.last_sequence;

                // Create/update texture when sequence changes
                let needs_update = video_display.texture_handle.is_none()
                    || video_display.last_sequence != video_display.last_texture_sequence;

                if needs_update {
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [width as usize, height as usize],
                        &video_display.rgba_data,
                    );
                    let texture = ui.ctx().load_texture("video_feed", image, egui::TextureOptions::LINEAR);
                    video_display.texture_handle = Some(texture);
                    video_display.last_texture_sequence = video_display.last_sequence;
                }

                // Display video with aspect ratio preserved
                if let Some(texture) = &video_display.texture_handle {
                    let available_width = ui.available_width();
                    let aspect = width as f32 / height as f32;
                    let display_height = available_width / aspect;

                    ui.image(egui::load::SizedTexture::new(
                        texture.id(),
                        egui::vec2(available_width, display_height),
                    ));

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!("{}x{} @ {:.1} fps", width, height, fps));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("#{}", seq));
                        });
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("No video feed").weak());
                });
            }
        });

    // Help bar at bottom
    egui::TopBottomPanel::bottom("help")
        .frame(egui::Frame::default().fill(egui::Color32::from_rgba_unmultiplied(30, 30, 30, 220)))
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                match input.source {
                    InputSource::Gamepad => {
                        ui.label("🕹️ L-Stick: Drive");
                        ui.label("R-Stick: Camera");
                        ui.label("🎚️ Triggers: Tool");
                        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "Select: E-STOP");
                    }
                    _ => {
                        ui.label("⌨️ WASD: Drive");
                        ui.label("🖱️ RMB+Drag: Camera");
                        ui.label("C: View  V: Free");
                        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "Esc: E-STOP");
                    }
                }
            });
        });
}

// ============================================================================
// Main
// ============================================================================

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Setup UDP socket for telemetry
    let local_addr: SocketAddr = format!("0.0.0.0:{}", args.local_port).parse()?;
    let socket = UdpSocket::bind(local_addr)?;
    socket.set_nonblocking(true)?;

    let rover_addr: SocketAddr = args
        .rover
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve rover address: {}", args.rover))?;

    // Parse rover host for video address
    let rover_host = rover_addr.ip();
    let video_addr: SocketAddr = format!("{}:{}", rover_host, args.video_port).parse()?;

    // Spawn background thread for video receiving and decoding
    let video_rx = spawn_video_thread(video_addr, args.video_local_port)?;

    println!("╔═══════════════════════════════════════════╗");
    println!("║         BVR Operator Station              ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  Rover:  {:26}   ║", rover_addr);
    println!("║  Video:  {:26}   ║", video_addr);
    println!("║  Local:  {:26}   ║", local_addr);
    println!("╠═══════════════════════════════════════════╣");
    println!("║  Gamepad:             Keyboard:           ║");
    println!("║    Left Stick         WASD / Arrows       ║");
    println!("║    Right Stick        Right-click + drag  ║");
    println!("║    Triggers           Q / E               ║");
    println!("║    A Button           Space               ║");
    println!("║    Select             Escape    (E-STOP)  ║");
    println!("║                       C = toggle 1st/3rd  ║");
    println!("║                       V = free look       ║");
    println!("╚═══════════════════════════════════════════╝");
    println!();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BVR Operator Station".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(RoverConnection {
            socket,
            rover_addr,
            last_send: Instant::now(),
        })
        .insert_resource(VideoReceiver { rx: Mutex::new(video_rx) })
        .insert_resource(Telemetry::default())
        .insert_resource(ControllerInput::default())
        .insert_resource(RoverPose::default())
        .insert_resource(CameraState::default())
        .insert_resource(VideoDisplay::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            gamepad_connections,
            read_input,
            send_commands,
            receive_telemetry,
            receive_video,
            update_rover_pose,
            update_rover_model,
            update_video_frustum,
            draw_frustum_gizmos,
            update_camera,
        ))
        .add_systems(Update, ui_system)
        .run();

    Ok(())
}


