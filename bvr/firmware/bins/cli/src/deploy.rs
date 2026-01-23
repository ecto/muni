//! Deploy commands for depot and rovers
//!
//! Developer tool for pushing local changes:
//! - `muni deploy depot` - rsync + docker compose build/up
//! - `muni deploy rover <hostname>` - cross-compile + scp

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

/// ANSI color codes
const GREEN: &str = "\x1b[0;32m";
const BLUE: &str = "\x1b[0;34m";
const YELLOW: &str = "\x1b[1;33m";
const RED: &str = "\x1b[0;31m";
const NC: &str = "\x1b[0m"; // No color

#[derive(Subcommand)]
pub enum DeployCommands {
    /// Deploy depot services (rsync + docker compose)
    Depot {
        /// Depot hostname or IP (default: depot)
        #[arg(short = 'H', long, default_value = "depot")]
        host: String,
        /// SSH user
        #[arg(short, long, default_value = "depot")]
        user: String,
        /// Remote path for depot directory
        #[arg(long, default_value = "~/depot")]
        path: String,
        /// Only sync files, don't rebuild containers
        #[arg(long)]
        sync_only: bool,
        /// Specific service to rebuild (default: all)
        #[arg(short, long)]
        service: Option<String>,
    },
    /// Deploy firmware to a rover (cross-compile + scp)
    Rover {
        /// Rover hostname (e.g., frog-0)
        hostname: String,
        /// SSH user
        #[arg(short, long, default_value = "cam")]
        user: String,
        /// Also deploy the muni CLI tool
        #[arg(long)]
        cli: bool,
        /// Sync config file (bvr.toml)
        #[arg(long)]
        config: bool,
        /// Don't restart bvrd service after deploy
        #[arg(long)]
        no_restart: bool,
        /// Full deploy: cli + config + restart
        #[arg(long)]
        all: bool,
        /// Skip upload if binary is unchanged (compare SHA256 hashes)
        #[arg(long)]
        skip_unchanged: bool,
    },
}

pub async fn run(cmd: DeployCommands) -> Result<()> {
    match cmd {
        DeployCommands::Depot {
            host,
            user,
            path,
            sync_only,
            service,
        } => deploy_depot(&host, &user, &path, sync_only, service.as_deref()).await,
        DeployCommands::Rover {
            hostname,
            user,
            cli,
            config,
            no_restart,
            all,
            skip_unchanged,
        } => {
            let deploy_cli = cli || all;
            let sync_config = config || all;
            let restart = !no_restart;
            deploy_rover(&hostname, &user, deploy_cli, sync_config, restart, skip_unchanged).await
        }
    }
}

/// Deploy depot services
async fn deploy_depot(
    host: &str,
    user: &str,
    remote_path: &str,
    sync_only: bool,
    service: Option<&str>,
) -> Result<()> {
    let start = Instant::now();
    let remote = format!("{}@{}", user, host);

    println!("{BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!("{BLUE}  Depot Deploy{NC}");
    println!("{BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!("  Host:       {GREEN}{host}{NC}");
    println!("  User:       {user}");
    println!("  Path:       {remote_path}");
    println!("  Sync only:  {sync_only}");
    if let Some(svc) = service {
        println!("  Service:    {svc}");
    }
    println!("{BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!();

    // Find depot directory (relative to muni repo root)
    let depot_path = find_depot_path()?;
    println!("{BLUE}▸ Source:{NC} {}", depot_path.display());

    // Test SSH connection
    print!("{BLUE}▸ Testing connection...{NC}");
    let ssh_test = Command::new("ssh")
        .args(["-o", "ConnectTimeout=5", &remote, "echo ok"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to run ssh")?;

    if !ssh_test.success() {
        println!(" {RED}FAILED{NC}");
        bail!(
            "Cannot connect to {}. Is Tailscale running?",
            remote
        );
    }
    println!(" {GREEN}OK{NC}");

    // Rsync depot directory
    println!("{BLUE}▸ Syncing files...{NC}");
    let rsync_status = Command::new("rsync")
        .args([
            "-avz",
            "--delete",
            "--exclude",
            "node_modules",
            "--exclude",
            "target",
            "--exclude",
            ".git",
            "--exclude",
            ".env", // Don't clobber remote .env
            "--exclude",
            "*.rrd",
            "--exclude",
            "data/",
            &format!("{}/", depot_path.display()),
            &format!("{}:{}/", remote, remote_path),
        ])
        .status()
        .context("Failed to run rsync")?;

    if !rsync_status.success() {
        bail!("rsync failed");
    }
    println!("{GREEN}✓ Files synced{NC}");

    if sync_only {
        println!();
        println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
        println!("{GREEN}  ✓ Sync complete ({:.1}s){NC}", start.elapsed().as_secs_f32());
        println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
        return Ok(());
    }

    // Docker compose build and up
    let compose_cmd = if let Some(svc) = service {
        format!(
            "cd {} && docker compose build {} && docker compose up -d {}",
            remote_path, svc, svc
        )
    } else {
        format!(
            "cd {} && docker compose build && docker compose up -d",
            remote_path
        )
    };

    println!("{BLUE}▸ Building containers...{NC}");
    let build_status = Command::new("ssh")
        .args([&remote, &compose_cmd])
        .status()
        .context("Failed to run docker compose")?;

    if !build_status.success() {
        bail!("docker compose failed");
    }
    println!("{GREEN}✓ Containers updated{NC}");

    println!();
    println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!(
        "{GREEN}  ✓ Deploy complete ({:.1}s){NC}",
        start.elapsed().as_secs_f32()
    );
    println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");

    Ok(())
}

/// Deploy firmware to a rover
async fn deploy_rover(
    hostname: &str,
    user: &str,
    deploy_cli: bool,
    sync_config: bool,
    restart: bool,
    skip_unchanged: bool,
) -> Result<()> {
    let start = Instant::now();
    let remote = format!("{}@{}", user, hostname);
    let target = "aarch64-unknown-linux-gnu";

    println!("{BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!("{BLUE}  Rover Deploy{NC}");
    println!("{BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!("  Target:   {GREEN}{hostname}{NC}");
    println!("  User:     {user}");
    println!("  CLI:      {deploy_cli}");
    println!("  Config:   {sync_config}");
    println!("  Restart:  {restart}");
    if skip_unchanged {
        println!("  Skip:     {YELLOW}unchanged binaries{NC}");
    }
    println!("{BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!();

    // Find firmware directory
    let firmware_path = find_firmware_path()?;

    // Check for cross-compilation toolchain
    let build_cmd = if Command::new("cross")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
    {
        "cross"
    } else {
        println!("{YELLOW}Note: 'cross' not found, using cargo directly{NC}");
        println!("{YELLOW}      Install cross for easier cross-compilation: cargo install cross{NC}");
        "cargo"
    };

    // Build binaries - use single command if building both bvrd and CLI (shared deps)
    if deploy_cli {
        println!("{BLUE}▸ Building bvrd + muni CLI for {target}...{NC}");
        let build_status = Command::new(build_cmd)
            .current_dir(&firmware_path)
            .args([
                "build",
                "--release",
                "--target",
                target,
                "--bin",
                "bvrd",
                "--bin",
                "muni",
            ])
            .status()
            .context("Failed to build binaries")?;

        if !build_status.success() {
            bail!("Build failed");
        }
    } else {
        println!("{BLUE}▸ Building bvrd for {target}...{NC}");
        let build_status = Command::new(build_cmd)
            .current_dir(&firmware_path)
            .args(["build", "--release", "--target", target, "--bin", "bvrd"])
            .status()
            .context("Failed to build bvrd")?;

        if !build_status.success() {
            bail!("Build failed");
        }
    }

    let bvrd_path = firmware_path
        .join("target")
        .join(target)
        .join("release")
        .join("bvrd");

    if !bvrd_path.exists() {
        bail!("Build artifact not found: {}", bvrd_path.display());
    }

    // Get binary size
    let metadata = std::fs::metadata(&bvrd_path)?;
    let size_mb = metadata.len() as f64 / 1_000_000.0;
    println!("{GREEN}✓ Built bvrd ({:.1} MB){NC}", size_mb);

    // Get CLI path if requested
    let cli_path = if deploy_cli {
        let path = firmware_path
            .join("target")
            .join(target)
            .join("release")
            .join("muni");

        if path.exists() {
            let cli_metadata = std::fs::metadata(&path)?;
            let cli_size_mb = cli_metadata.len() as f64 / 1_000_000.0;
            println!("{GREEN}✓ Built muni CLI ({:.1} MB){NC}", cli_size_mb);
            Some(path)
        } else {
            println!("{YELLOW}Warning: CLI binary not found{NC}");
            None
        }
    } else {
        None
    };

    // Test SSH connection
    print!("{BLUE}▸ Testing connection to {hostname}...{NC}");
    let ssh_test = Command::new("ssh")
        .args(["-o", "ConnectTimeout=5", &remote, "echo ok"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to run ssh")?;

    if !ssh_test.success() {
        println!(" {RED}FAILED{NC}");
        bail!(
            "Cannot connect to {}. Is Tailscale running?",
            remote
        );
    }
    println!(" {GREEN}OK{NC}");

    // Get current version
    print!("{BLUE}▸ Current version: {NC}");
    let version_output = Command::new("ssh")
        .args([&remote, "/usr/local/bin/bvrd --version 2>/dev/null || echo 'not installed'"])
        .output()
        .context("Failed to check version")?;
    let current_version = String::from_utf8_lossy(&version_output.stdout);
    println!("{YELLOW}{}{NC}", current_version.trim());

    // Upload binaries (with optional hash comparison)
    let bvrd_skipped = if skip_unchanged {
        let local_hash = sha256_file(&bvrd_path)?;
        let remote_hash = get_remote_hash(&remote, "/usr/local/bin/bvrd")?;
        if local_hash == remote_hash {
            println!("{GREEN}✓ bvrd unchanged, skipping upload{NC}");
            true
        } else {
            upload_file(&bvrd_path, &remote, "/tmp/bvrd.new", "bvrd")?;
            false
        }
    } else {
        upload_file(&bvrd_path, &remote, "/tmp/bvrd.new", "bvrd")?;
        false
    };

    // Upload CLI if built
    let cli_skipped = if let Some(ref cli) = cli_path {
        if skip_unchanged {
            let local_hash = sha256_file(cli)?;
            let remote_hash = get_remote_hash(&remote, "/usr/local/bin/muni")?;
            if local_hash == remote_hash {
                println!("{GREEN}✓ muni CLI unchanged, skipping upload{NC}");
                true
            } else {
                upload_file(cli, &remote, "/tmp/muni.new", "muni CLI")?;
                false
            }
        } else {
            upload_file(cli, &remote, "/tmp/muni.new", "muni CLI")?;
            false
        }
    } else {
        true // No CLI to upload
    };

    // Upload config if requested
    if sync_config {
        let config_path = firmware_path.join("config").join("bvr.toml");
        if config_path.exists() {
            println!("{BLUE}▸ Uploading config...{NC}");
            let scp_status = Command::new("scp")
                .args([
                    "-q",
                    config_path.to_str().unwrap(),
                    &format!("{}:/tmp/bvr.toml.new", remote),
                ])
                .status()
                .context("Failed to upload config")?;

            if !scp_status.success() {
                bail!("Config upload failed");
            }
            println!("{GREEN}✓ Uploaded config{NC}");
        } else {
            println!("{YELLOW}Warning: config/bvr.toml not found{NC}");
        }
    }

    // Check if there's anything to install
    let has_bvrd = !bvrd_skipped;
    let has_cli = !cli_skipped && cli_path.is_some();
    let has_config = sync_config;
    let has_work = has_bvrd || has_cli || has_config;

    if !has_work && !restart {
        println!("{GREEN}✓ Nothing to install (all binaries unchanged){NC}");
        println!();
        println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
        println!(
            "{GREEN}  ✓ Deploy complete ({:.1}s){NC}",
            start.elapsed().as_secs_f32()
        );
        println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
        return Ok(());
    }

    // Install on remote
    println!("{BLUE}▸ Installing...{NC}");

    let install_script = format!(
        r#"
set -e

# Stop service if we're restarting and bvrd changed
if [ "{restart}" = "true" ] && [ "{has_bvrd}" = "true" ] && systemctl is-active --quiet bvrd 2>/dev/null; then
    echo "  Stopping bvrd..."
    sudo systemctl stop bvrd
fi

# Atomic move of bvrd binary if uploaded
if [ -f /tmp/bvrd.new ]; then
    sudo mv /tmp/bvrd.new /usr/local/bin/bvrd
    sudo chmod +x /usr/local/bin/bvrd
fi

# CLI if present
if [ -f /tmp/muni.new ]; then
    sudo mv /tmp/muni.new /usr/local/bin/muni
    sudo chmod +x /usr/local/bin/muni
    # Create legacy symlink for compatibility
    sudo ln -sf /usr/local/bin/muni /usr/local/bin/bvr
fi

# Config if present
if [ -f /tmp/bvr.toml.new ]; then
    sudo mkdir -p /etc/bvr
    sudo mv /tmp/bvr.toml.new /etc/bvr/bvr.toml
fi

# Restart service if requested and we have changes
if [ "{restart}" = "true" ] && [ "{has_bvrd}" = "true" ]; then
    if systemctl list-unit-files | grep -q bvrd; then
        # Ensure CAN interface is up
        if systemctl list-unit-files | grep -q can.service; then
            sudo systemctl start can.service
        fi
        echo "  Starting bvrd..."
        sudo systemctl restart bvrd
        sleep 1
        if systemctl is-active --quiet bvrd; then
            echo "  Service running"
        else
            echo "  WARNING: Service failed to start"
            sudo journalctl -u bvrd -n 10 --no-pager
        fi
    else
        echo "  Note: bvrd.service not installed, skipping restart"
    fi
fi
"#,
        restart = restart,
        has_bvrd = has_bvrd,
    );

    // Run the install script
    let mut child = Command::new("ssh")
        .args([&remote, "bash", "-s"])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start SSH")?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(install_script.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        bail!("Install failed");
    }
    println!("{GREEN}✓ Installed{NC}");

    // Verify
    print!("{BLUE}▸ New version: {NC}");
    let version_output = Command::new("ssh")
        .args([&remote, "/usr/local/bin/bvrd --version 2>/dev/null || echo 'unknown'"])
        .output()
        .context("Failed to check version")?;
    let new_version = String::from_utf8_lossy(&version_output.stdout);
    println!("{GREEN}{}{NC}", new_version.trim());

    println!();
    println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");
    println!(
        "{GREEN}  ✓ Deploy complete ({:.1}s){NC}",
        start.elapsed().as_secs_f32()
    );
    println!("{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{NC}");

    Ok(())
}

/// Find the depot directory relative to the current working directory or muni repo
fn find_depot_path() -> Result<PathBuf> {
    // Try relative paths from common locations
    let candidates = [
        PathBuf::from("depot"),
        PathBuf::from("../depot"),
        PathBuf::from("../../depot"),
        PathBuf::from("../../../depot"),
    ];

    for path in &candidates {
        if path.join("docker-compose.yml").exists() {
            return Ok(std::fs::canonicalize(path)?);
        }
    }

    // Try from MUNI_ROOT env var
    if let Ok(root) = std::env::var("MUNI_ROOT") {
        let path = PathBuf::from(root).join("depot");
        if path.join("docker-compose.yml").exists() {
            return Ok(path);
        }
    }

    bail!(
        "Could not find depot directory. Run from muni repo or set MUNI_ROOT env var."
    )
}

/// Find the firmware directory
fn find_firmware_path() -> Result<PathBuf> {
    // Try relative paths from common locations
    let candidates = [
        PathBuf::from("bvr/firmware"),
        PathBuf::from("../bvr/firmware"),
        PathBuf::from("../../bvr/firmware"),
        PathBuf::from("firmware"),
        PathBuf::from("../firmware"),
        PathBuf::from("."), // If run from firmware dir
    ];

    for path in &candidates {
        if path.join("Cargo.toml").exists()
            && path.join("bins").exists()
        {
            return Ok(std::fs::canonicalize(path)?);
        }
    }

    // Try from MUNI_ROOT env var
    if let Ok(root) = std::env::var("MUNI_ROOT") {
        let path = PathBuf::from(root).join("bvr/firmware");
        if path.join("Cargo.toml").exists() {
            return Ok(path);
        }
    }

    bail!(
        "Could not find firmware directory. Run from muni repo or set MUNI_ROOT env var."
    )
}

/// Calculate SHA256 hash of a local file
fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).context("Failed to open file for hashing")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Get SHA256 hash of a remote file via SSH
fn get_remote_hash(remote: &str, remote_path: &str) -> Result<String> {
    let output = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=5",
            remote,
            &format!("sha256sum {} 2>/dev/null | cut -d' ' -f1", remote_path),
        ])
        .output()
        .context("Failed to get remote hash")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Upload a file via SCP
fn upload_file(local_path: &Path, remote: &str, remote_path: &str, name: &str) -> Result<()> {
    println!("{BLUE}▸ Uploading {name}...{NC}");
    let scp_status = Command::new("scp")
        .args(["-q", local_path.to_str().unwrap(), &format!("{remote}:{remote_path}")])
        .status()
        .context("Failed to upload")?;

    if !scp_status.success() {
        bail!("{} upload failed", name);
    }
    println!("{GREEN}✓ Uploaded {name}{NC}");
    Ok(())
}
