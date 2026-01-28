//! Logging initialization with stdout and rolling file output.

use anyhow::Result;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging with stdout and rolling file output.
///
/// Returns a guard that must be held for the lifetime of the program to ensure
/// logs are properly flushed on shutdown.
pub(crate) fn init_logging(
    log_dir: &std::path::Path,
    level: &str,
    enable_console: bool,
) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    // Create log directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(log_dir) {
        eprintln!("Error: Cannot create log directory '{}': {}", log_dir.display(), e);
        eprintln!();
        eprintln!("The default log directory requires root permissions.");
        eprintln!("Try running with local directories:");
        eprintln!();
        eprintln!("  cargo run --bin bvrd -- --sim --log-dir ./logs --recording-dir ./sessions");
        eprintln!();
        eprintln!("Or disable recording for quick testing:");
        eprintln!();
        eprintln!("  cargo run --bin bvrd -- --sim --no-recording --log-dir /tmp");
        eprintln!();
        return Err(e.into());
    }

    // Rolling file appender: daily rotation
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "bvrd.log");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    // Build filter from level string, with fallback
    // Include teleop for command timing diagnostics
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("bvrd={},teleop={},recording=info", level, level)));

    // Stdout layer: human-readable, colored
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false);

    // File layer: no ANSI codes, includes timestamps
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(true);

    // Initialize with or without console-subscriber
    #[cfg(feature = "console")]
    {
        if enable_console {
            // Use console_subscriber's builder which handles layer composition
            console_subscriber::ConsoleLayer::builder()
                .server_addr(([0, 0, 0, 0], 6669))
                .with_default_env()
                .init();

            eprintln!("tokio-console enabled on port 6669 (default tracing, file logging disabled)");

            // Note: When using console, we use its built-in subscriber setup
            // File logging is disabled in this mode for simplicity
            return Ok(guard);
        }

        // Non-console path
        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
    }

    #[cfg(not(feature = "console"))]
    {
        let _ = enable_console; // Suppress unused warning
        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
    }

    Ok(guard)
}
