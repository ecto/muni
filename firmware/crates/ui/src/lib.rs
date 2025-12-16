//! Onboard web dashboard for bvr.
//!
//! Serves a simple status dashboard accessible at http://localhost:8080

use std::sync::Arc;
use teleop::Telemetry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tracing::{debug, error, info};

/// Dashboard configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self { port: 8080 }
    }
}

/// Dashboard server.
pub struct Dashboard {
    config: Config,
    telemetry_rx: watch::Receiver<Telemetry>,
}

impl Dashboard {
    pub fn new(config: Config, telemetry_rx: watch::Receiver<Telemetry>) -> Self {
        Self {
            config,
            telemetry_rx,
        }
    }

    /// Run the dashboard server.
    pub async fn run(self) -> std::io::Result<()> {
        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!(addr, "Dashboard server listening");

        let telemetry_rx = Arc::new(self.telemetry_rx);

        loop {
            let (stream, addr) = listener.accept().await?;
            debug!(?addr, "Dashboard connection");

            let rx = telemetry_rx.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, rx).await {
                    error!(?e, "Dashboard connection error");
                }
            });
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    telemetry_rx: Arc<watch::Receiver<Telemetry>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/");

    // Drain headers
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
    }

    let response = match path {
        "/" => {
            let html = include_str!("dashboard.html");
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                html.len(),
                html
            )
        }
        "/api/telemetry" => {
            let telemetry = telemetry_rx.borrow().clone();
            let json = serde_json::to_string(&telemetry).unwrap_or_default();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                json.len(),
                json
            )
        }
        _ => {
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
        }
    };

    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

