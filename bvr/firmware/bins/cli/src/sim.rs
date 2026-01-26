//! Simulation commands for multi-rover development.
//!
//! - `muni sim launch` - Build and start sim-bridge + N bvrd instances
//! - `muni sim status` - Check running simulation status

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const GREEN: &str = "\x1b[0;32m";
const BLUE: &str = "\x1b[0;34m";
const YELLOW: &str = "\x1b[1;33m";
const RED: &str = "\x1b[0;31m";
const NC: &str = "\x1b[0m";

#[derive(Subcommand)]
pub enum SimCommands {
    /// Launch multi-rover simulation (sim-bridge + bvrd instances)
    Launch {
        /// Path to scenario TOML file
        #[arg(short, long, default_value = "bvr/firmware/config/sim/scenario.toml")]
        scenario: PathBuf,

        /// Number of rovers to launch (0 = all from scenario)
        #[arg(short = 'n', long, default_value = "0")]
        count: usize,

        /// Skip cargo build step
        #[arg(long)]
        no_build: bool,

        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// sim-bridge HTTP status port
        #[arg(long, default_value = "4900")]
        port: u16,
    },

    /// Check simulation status
    Status {
        /// sim-bridge HTTP port
        #[arg(short, long, default_value = "4900")]
        port: u16,
    },
}

pub async fn run(cmd: SimCommands) -> Result<()> {
    match cmd {
        SimCommands::Launch {
            scenario,
            count,
            no_build,
            release,
            port,
        } => launch(&scenario, count, no_build, release, port).await,
        SimCommands::Status { port } => status(port).await,
    }
}

/// Parse scenario TOML to extract rover configs.
#[derive(Debug, serde::Deserialize)]
struct Scenario {
    #[serde(rename = "rover")]
    rovers: Vec<RoverEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct RoverEntry {
    id: String,
    can_port: u16,
}

async fn launch(
    scenario: &Path,
    count: usize,
    no_build: bool,
    release: bool,
    port: u16,
) -> Result<()> {
    // Find workspace root (where we run cargo from)
    let workspace_root = find_workspace_root()?;

    // Validate scenario file
    let scenario_path = if scenario.is_absolute() {
        scenario.to_path_buf()
    } else {
        workspace_root.join(scenario)
    };
    if !scenario_path.exists() {
        bail!(
            "Scenario file not found: {}",
            scenario_path.display()
        );
    }

    // Parse scenario to get rover list
    let scenario_content = std::fs::read_to_string(&scenario_path)
        .context("Failed to read scenario file")?;
    let scenario_data: Scenario =
        toml::from_str(&scenario_content).context("Failed to parse scenario")?;

    let rovers: Vec<&RoverEntry> = if count > 0 {
        scenario_data.rovers.iter().take(count).collect()
    } else {
        scenario_data.rovers.iter().collect()
    };

    println!(
        "{}=== muni sim ==={}\n  scenario: {}\n  rovers: {}",
        BLUE,
        NC,
        scenario_path.display(),
        rovers.len()
    );

    // Build
    if !no_build {
        println!("\n{}Building...{}", YELLOW, NC);

        // Build sim-bridge (standalone project in depot/sim-bridge/)
        print!("  sim-bridge... ");
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        if release {
            cmd.arg("--release");
        }
        cmd.current_dir(workspace_root.join("depot/sim-bridge"))
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        let output = cmd.output().context("Failed to run cargo build for sim-bridge")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}FAILED{}", RED, NC);
            bail!("sim-bridge build failed:\n{}", stderr);
        }
        println!("{}ok{}", GREEN, NC);

        // Build bvrd (firmware workspace)
        print!("  bvrd... ");
        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--bin").arg("bvrd");
        if release {
            cmd.arg("--release");
        }
        cmd.current_dir(workspace_root.join("bvr/firmware"))
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        let output = cmd.output().context("Failed to run cargo build for bvrd")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}FAILED{}", RED, NC);
            bail!("bvrd build failed:\n{}", stderr);
        }
        println!("{}ok{}", GREEN, NC);
    }

    // Determine binary paths
    let profile_dir = if release { "release" } else { "debug" };
    let sim_bridge_bin = workspace_root
        .join("depot/sim-bridge/target")
        .join(profile_dir)
        .join("sim-bridge");
    let bvrd_bin = workspace_root
        .join("bvr/firmware/target")
        .join(profile_dir)
        .join("bvrd");

    if !sim_bridge_bin.exists() {
        bail!("sim-bridge binary not found at {}", sim_bridge_bin.display());
    }
    if !bvrd_bin.exists() {
        bail!("bvrd binary not found at {}", bvrd_bin.display());
    }

    // Launch sim-bridge
    println!("\n{}Launching sim-bridge...{}", BLUE, NC);
    let mut sim_bridge = Command::new(&sim_bridge_bin)
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(&workspace_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start sim-bridge")?;

    // Wait for sim-bridge to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Check if sim-bridge is still running
    match sim_bridge.try_wait() {
        Ok(Some(status)) => {
            bail!("sim-bridge exited early with status: {}", status);
        }
        Ok(None) => {} // Still running
        Err(e) => {
            bail!("Failed to check sim-bridge status: {}", e);
        }
    }

    println!(
        "  {}sim-bridge running{} (http://localhost:{}/status)",
        GREEN, NC, port
    );

    // Launch bvrd instances
    println!("\n{}Launching rovers...{}", BLUE, NC);
    let mut children: Vec<(String, Child)> = Vec::new();
    let config_dir = workspace_root.join("bvr/firmware/config/sim");

    for rover in &rovers {
        let config_path = config_dir.join(format!("{}.toml", rover.id));
        if !config_path.exists() {
            println!(
                "  {}WARNING{}: config not found for {}, skipping",
                YELLOW, NC, rover.id
            );
            continue;
        }

        // Derive RTC port from CAN port: 4910->4852, 4911->4853, etc.
        let rtc_port = 4852 + (rover.can_port - 4910);

        let child = Command::new(&bvrd_bin)
            .arg("--remote-can")
            .arg(format!("tcp://127.0.0.1:{}", rover.can_port))
            .arg("--config")
            .arg(&config_path)
            .arg("--rtc-port")
            .arg(rtc_port.to_string())
            .arg("--no-camera")
            .arg("--no-recording")
            .arg("--ui-port")
            .arg("0")
            .current_dir(&workspace_root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to start bvrd for {}", rover.id))?;

        println!(
            "  {}{}{}  CAN:{} RTC:{}",
            GREEN, rover.id, NC, rover.can_port, rtc_port
        );
        children.push((rover.id.clone(), child));

        // Stagger launches slightly
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!(
        "\n{}All {} rovers launched.{}",
        GREEN,
        children.len(),
        NC
    );
    println!("  status:  http://localhost:{}/status", port);
    println!("  console: http://localhost (if depot running)");
    println!("\nPress Ctrl+C to stop.");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    // Cleanup
    println!("\n{}Shutting down...{}", YELLOW, NC);
    for (id, mut child) in children {
        let _ = child.kill();
        let _ = child.wait();
        println!("  stopped {}", id);
    }
    let _ = sim_bridge.kill();
    let _ = sim_bridge.wait();
    println!("  stopped sim-bridge");
    println!("{}Done.{}", GREEN, NC);

    Ok(())
}

async fn status(port: u16) -> Result<()> {
    let url = format!("http://localhost:{}/status", port);
    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to connect to sim-bridge at {}", url))?;

    if !resp.status().is_success() {
        bail!("sim-bridge returned status {}", resp.status());
    }

    let body: serde_json::Value = resp.json().await?;

    // Pretty print
    println!("{}sim-bridge status{}", BLUE, NC);

    if let Some(rovers) = body.get("rovers").and_then(|v| v.as_array()) {
        println!("  rovers: {}", rovers.len());
        for rover in rovers {
            let id = rover.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let connected = rover
                .get("connected")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let pose = rover.get("pose");
            let x = pose.and_then(|p| p.get("x")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = pose.and_then(|p| p.get("y")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let theta = pose
                .and_then(|p| p.get("theta"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let status_str = if connected {
                format!("{}connected{}", GREEN, NC)
            } else {
                format!("{}waiting{}", YELLOW, NC)
            };

            println!(
                "  {}  {} | ({:.2}, {:.2}) θ={:.1}°",
                id,
                status_str,
                x,
                y,
                theta.to_degrees()
            );
        }
    }

    Ok(())
}

/// Find the workspace root by looking for the bvr/firmware/Cargo.toml
fn find_workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;

    // Walk up looking for the muni repo root (has bvr/ and depot/)
    for _ in 0..10 {
        if dir.join("bvr").is_dir() && dir.join("depot").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    // Fallback: assume current directory
    Ok(std::env::current_dir()?)
}
