//! Generic Ethernet MCU firmware for tool attachments.
//!
//! Runs on Waveshare ESP32-S3 PoE board. Role is determined by config.
//! Communicates with bvrd via UDP:
//! - Port 4861: Discovery broadcast (MCU → broadcast, 500ms)
//! - Port 4862: Command listener (bvrd → MCU, unicast)

mod command;
mod config;
mod discovery;
mod roles;
mod watchdog;

use command::{Command, CommandListener};
use config::ToolConfig;
use discovery::{DiscoveryBroadcaster, McuState};
use roles::lights::LightsRole;
use roles::Role;
use watchdog::Watchdog;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
};

use std::thread;
use std::time::Duration;

/// Main loop tick rate.
const TICK_INTERVAL: Duration = Duration::from_millis(10); // 100Hz

fn main() {
    // Link ESP-IDF patches (required for std)
    esp_idf_svc::sys::link_patches();

    // Initialize logging
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("mcu-eth starting");

    // ESP-IDF system init
    let _peripherals = Peripherals::take().expect("failed to take peripherals");
    let sysloop = EspSystemEventLoop::take().expect("failed to take event loop");
    let nvs = EspDefaultNvsPartition::take().expect("failed to init NVS");

    // Ethernet is configured via sdkconfig.defaults (RMII PHY, IP101, pin mapping).
    // ESP-IDF handles the low-level PHY/MAC init. We use the default netif which
    // gets DHCP automatically when Ethernet links up.
    //
    // TODO: Replace with esp_idf_svc::eth API once pin mapping is confirmed
    // on the actual Waveshare board. For now, ESP-IDF's Kconfig handles it.
    //
    // Wait for network to be ready (DHCP or link-local)
    log::info!("waiting for Ethernet link...");
    // ESP-IDF auto-starts Ethernet from sdkconfig. Give it time to link + DHCP.
    thread::sleep(Duration::from_secs(5));
    log::info!("network init complete (DHCP via sdkconfig)");

    // Load config — hardcoded default for now.
    // Production: load from SPIFFS /config/tool.toml
    let config = ToolConfig::default_lights();
    log::info!(
        "config: type={}, serial={:#06x}, name={}",
        config.identity.tool_type,
        config.identity.serial,
        config.identity.name
    );

    // Create role based on config
    let mut role: Box<dyn Role> = match config.identity.tool_type.as_str() {
        "lights" => {
            let lc = config.lights.as_ref().expect("lights config missing");
            Box::new(LightsRole::new(
                lc.headlight_pin,
                lc.headlight_count,
                lc.led_strip_pin,
                lc.led_count,
            ))
        }
        other => {
            log::warn!("unknown role '{}', using lights default", other);
            Box::new(LightsRole::new(4, 2, 5, 24))
        }
    };

    // Initialize networking
    let mut broadcaster =
        DiscoveryBroadcaster::new(&config).expect("failed to create discovery broadcaster");

    let mut listener =
        CommandListener::new(config.network.command_port).expect("failed to create command listener");

    let mut watchdog = Watchdog::new();

    log::info!(
        "running: role={}, discovery=:{}, command=:{}",
        config.identity.tool_type,
        config.network.discovery_port,
        config.network.command_port
    );

    // Keep system handles alive (Ethernet PHY, event loop, NVS)
    let _sysloop = sysloop;
    let _nvs = nvs;

    // Main loop (100Hz)
    loop {
        // Drain incoming commands
        while let Some(cmd) = listener.try_recv() {
            watchdog.feed();

            match cmd {
                Command::SetState { rover_state } => {
                    let state = roles::RoverState::from(rover_state);
                    role.on_state_change(state);
                    broadcaster.set_state(McuState::Running);
                    log::debug!("state -> {:?}", state);
                }
                Command::ToolCommand {
                    axis,
                    motor,
                    action_a,
                    action_b,
                } => {
                    role.handle_command(axis, motor, action_a, action_b);
                }
            }
        }

        // Check watchdog
        let (timed_out, should_shutdown) = watchdog.check();
        if timed_out {
            log::warn!("watchdog: command timeout");
            role.on_timeout();
            broadcaster.set_state(McuState::Idle);
        }
        if should_shutdown {
            log::warn!("watchdog: safe shutdown");
            role.on_shutdown();
            broadcaster.set_state(McuState::Idle);
        }

        // Tick role (flash animation, PWM output)
        role.tick();

        // Update heartbeat with role status
        let mut status_buf = [0u8; 8];
        role.status(&mut status_buf);
        broadcaster.set_role_status(&status_buf);
        broadcaster.tick();

        thread::sleep(TICK_INTERVAL);
    }
}
