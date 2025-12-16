//! Operator station for controlling BVR.
//!
//! A Bevy-based 3D interface with Xbox controller support.

use bevy::prelude::*;
use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
use bevy::input::mouse::MouseMotion;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use clap::Parser;
use std::f32::consts::{FRAC_PI_2, PI};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
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

/// Received telemetry from rover
#[derive(Resource, Default)]
struct Telemetry {
    mode: Mode,
    battery_voltage: f64,
    system_current: f64,
    velocity: Twist,
    connected: bool,
    last_recv: Option<Instant>,
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

/// Simulated rover pose (from commands, for visualization)
#[derive(Resource, Default)]
struct RoverPose {
    x: f32,
    y: f32,
    theta: f32,
}

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
struct RoverModel;

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
        // Stick up = forward, stick right = turn right (negative angular)
        input.linear = if left_y.abs() > 0.1 { -left_y } else { 0.0 };
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

        // Triggers for tool axis
        let rt = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0);
        let lt = gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0);
        input.tool_axis = rt - lt;

        // Buttons
        input.action_a = gamepad.pressed(GamepadButton::South);
        input.action_b = gamepad.pressed(GamepadButton::East);
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
            // Parse telemetry
            telemetry.connected = true;
            telemetry.last_recv = Some(Instant::now());

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

            if len >= 10 {
                telemetry.battery_voltage = f64::from_le_bytes(
                    buf[2..10].try_into().unwrap_or([0; 8])
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

fn update_rover_pose(
    input: Res<ControllerInput>,
    mut pose: ResMut<RoverPose>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Simple integration of commanded velocity (for visualization)
    let linear = input.linear * 2.0 * dt;
    let angular = input.angular * 1.5 * dt;

    pose.theta += angular;
    pose.x += linear * pose.theta.cos();
    pose.y += linear * pose.theta.sin();
}

fn update_rover_model(
    pose: Res<RoverPose>,
    mut query: Query<&mut Transform, With<RoverModel>>,
) {
    for mut transform in &mut query {
        transform.translation.x = pose.x;
        transform.translation.z = pose.y;
        // Positive theta = turned left = counter-clockwise from above
        transform.rotation = Quat::from_rotation_y(pose.theta);
    }
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

    // Auto-reset camera behind rover when moving and no recent camera input
    let is_moving = input.linear.abs() > 0.1 || input.angular.abs() > 0.1;
    if is_moving && camera_state.last_input > 1.0 && camera_state.mode == CameraMode::ThirdPerson {
        // Smoothly return to behind rover
        camera_state.yaw_offset = camera_state.yaw_offset * (1.0 - 2.0 * dt);
    }

    // Wrap yaw offset
    if camera_state.yaw_offset > PI {
        camera_state.yaw_offset -= 2.0 * PI;
    } else if camera_state.yaw_offset < -PI {
        camera_state.yaw_offset += 2.0 * PI;
    }

    for mut transform in &mut query {
        let rover_pos = Vec3::new(pose.x, 0.15, pose.y);

        match camera_state.mode {
            CameraMode::FirstPerson => {
                // Camera at rover position, looking forward
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
                    pose.y - horizontal_dist * total_yaw.sin(),
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
                    pose.y - horizontal_dist * camera_state.yaw_offset.sin(),
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

    // Setup UDP socket
    let local_addr: SocketAddr = format!("0.0.0.0:{}", args.local_port).parse()?;
    let socket = UdpSocket::bind(local_addr)?;
    socket.set_nonblocking(true)?;

    let rover_addr: SocketAddr = args.rover.parse()?;

    println!("╔═══════════════════════════════════════════╗");
    println!("║         BVR Operator Station              ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  Rover:  {:26}   ║", rover_addr);
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
        .insert_resource(Telemetry::default())
        .insert_resource(ControllerInput::default())
        .insert_resource(RoverPose::default())
        .insert_resource(CameraState::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            gamepad_connections,
            read_input,
            send_commands,
            receive_telemetry,
            update_rover_pose,
            update_rover_model,
            update_camera,
            ui_system,
        ))
        .run();

    Ok(())
}
