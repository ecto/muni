//! GPS/GNSS commands: monitor, configure-base, configure-rover
//!
//! Supports both NMEA (rover mode) and RTCM3/UBX (base station mode) monitoring.

use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Padding, Paragraph},
    Frame, Terminal,
};
use std::{
    collections::HashMap,
    io::{stdout, Read, Write},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Subcommand)]
pub enum GpsCommands {
    /// Real-time GPS monitor TUI (auto-detects NMEA or RTCM mode)
    Monitor {
        /// Serial port path
        #[arg(short, long, default_value = "/dev/tty.usbmodem101")]
        port: String,
        /// Baud rate
        #[arg(short, long, default_value = "38400")]
        baud: u32,
    },
    /// Configure ZED-F9P as RTK base station
    ConfigureBase {
        /// Serial port path
        #[arg(short, long, default_value = "/dev/ttyUSB0")]
        port: String,
        /// Baud rate
        #[arg(short, long, default_value = "38400")]
        baud: u32,
        /// Survey-in duration (seconds). Use 86400 for production.
        #[arg(long, default_value = "300")]
        survey_duration: u32,
        /// Survey-in accuracy target (meters). Use 0.1 for production.
        #[arg(long, default_value = "2.0")]
        survey_accuracy: f32,
        /// Use fixed position instead of survey-in (lat,lon,alt)
        #[arg(long, value_parser = parse_position)]
        fixed_position: Option<(f64, f64, f64)>,
    },
    /// Configure ZED-F9P as RTK rover
    ConfigureRover {
        /// Serial port path
        #[arg(short, long, default_value = "/dev/ttyACM0")]
        port: String,
        /// Baud rate (output)
        #[arg(short, long, default_value = "115200")]
        baud: u32,
        /// Update rate in Hz
        #[arg(long, default_value = "10")]
        rate: u8,
    },
}

fn parse_position(s: &str) -> Result<(f64, f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err("Expected format: lat,lon,alt (e.g., 41.48,-81.80,213.5)".to_string());
    }
    let lat = parts[0].parse().map_err(|_| "Invalid latitude")?;
    let lon = parts[1].parse().map_err(|_| "Invalid longitude")?;
    let alt = parts[2].parse().map_err(|_| "Invalid altitude")?;
    Ok((lat, lon, alt))
}

pub async fn run(cmd: GpsCommands) -> Result<()> {
    match cmd {
        GpsCommands::Monitor { port, baud } => run_monitor(&port, baud),
        GpsCommands::ConfigureBase {
            port,
            baud,
            survey_duration,
            survey_accuracy,
            fixed_position,
        } => configure_base(&port, baud, survey_duration, survey_accuracy, fixed_position),
        GpsCommands::ConfigureRover { port, baud, rate } => configure_rover(&port, baud, rate),
    }
}

// ============================================================================
// GPS Monitor TUI (supports NMEA and RTCM/UBX modes)
// ============================================================================

/// Detected receiver mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ReceiverMode {
    #[default]
    Unknown,
    Rover,      // NMEA output (normal GPS)
    BaseStation, // RTCM output (corrections)
}

/// GPS fix quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FixQuality {
    #[default]
    NoFix = 0,
    GpsFix = 1,
    DgpsFix = 2,
    PpsFix = 3,
    RtkFixed = 4,
    RtkFloat = 5,
    Estimated = 6,
}

impl From<u8> for FixQuality {
    fn from(val: u8) -> Self {
        match val {
            1 => Self::GpsFix,
            2 => Self::DgpsFix,
            3 => Self::PpsFix,
            4 => Self::RtkFixed,
            5 => Self::RtkFloat,
            6 => Self::Estimated,
            _ => Self::NoFix,
        }
    }
}

impl FixQuality {
    fn label(&self) -> &'static str {
        match self {
            Self::NoFix => "NO FIX",
            Self::GpsFix => "GPS",
            Self::DgpsFix => "DGPS",
            Self::PpsFix => "PPS",
            Self::RtkFixed => "RTK FIXED",
            Self::RtkFloat => "RTK FLOAT",
            Self::Estimated => "ESTIMATED",
        }
    }

    fn color(&self) -> Color {
        match self {
            Self::NoFix => Color::Red,
            Self::GpsFix => Color::Yellow,
            Self::DgpsFix => Color::LightYellow,
            Self::PpsFix => Color::LightGreen,
            Self::RtkFixed => Color::Green,
            Self::RtkFloat => Color::Cyan,
            Self::Estimated => Color::Gray,
        }
    }
}

/// Satellite info from GSV sentences
#[derive(Debug, Clone, Default)]
struct Satellite {
    prn: u16,
    #[allow(dead_code)]
    elevation: Option<u8>,
    #[allow(dead_code)]
    azimuth: Option<u16>,
    snr: Option<u8>,
}

/// Constellation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Constellation {
    Gps,
    Glonass,
    Galileo,
    BeiDou,
}

impl Constellation {
    fn label(&self) -> &'static str {
        match self {
            Self::Gps => "GPS",
            Self::Glonass => "GLONASS",
            Self::Galileo => "Galileo",
            Self::BeiDou => "BeiDou",
        }
    }

    fn color(&self) -> Color {
        match self {
            Self::Gps => Color::Cyan,
            Self::Glonass => Color::Magenta,
            Self::Galileo => Color::Yellow,
            Self::BeiDou => Color::Red,
        }
    }
}

/// RTCM message statistics
#[derive(Debug, Clone, Default)]
struct RtcmStats {
    message_type: u16,
    count: u32,
    bytes: u64,
    last_seen: Option<Instant>,
}

/// Survey-in status from UBX-NAV-SVIN
#[derive(Debug, Clone, Default)]
struct SurveyStatus {
    active: bool,
    valid: bool,
    duration_s: u32,
    mean_accuracy_mm: u32,
    observations: u32,
}

/// Current GPS state (unified for both modes)
#[derive(Debug, Clone)]
struct GpsState {
    // Mode detection
    mode: ReceiverMode,

    // === Rover mode (NMEA) ===
    latitude: Option<f64>,
    longitude: Option<f64>,
    altitude: Option<f64>,
    fix_quality: FixQuality,
    satellites_used: u8,
    hdop: f32,
    utc_time: Option<String>,
    satellites: HashMap<Constellation, Vec<Satellite>>,

    // === Base station mode (RTCM/UBX) ===
    rtcm_messages: HashMap<u16, RtcmStats>,
    survey: SurveyStatus,
    total_rtcm_bytes: u64,

    // === Common ===
    last_sentences: Vec<String>,
    messages_received: u64,
    bytes_received: u64,
    last_update: Option<Instant>,
    start_time: Instant,
}

impl Default for GpsState {
    fn default() -> Self {
        Self {
            mode: ReceiverMode::default(),
            latitude: None,
            longitude: None,
            altitude: None,
            fix_quality: FixQuality::default(),
            satellites_used: 0,
            hdop: 0.0,
            utc_time: None,
            satellites: HashMap::new(),
            rtcm_messages: HashMap::new(),
            survey: SurveyStatus::default(),
            total_rtcm_bytes: 0,
            last_sentences: Vec::new(),
            messages_received: 0,
            bytes_received: 0,
            last_update: None,
            start_time: Instant::now(),
        }
    }
}

impl GpsState {
    fn total_satellites_in_view(&self) -> usize {
        self.satellites.values().map(|v| v.len()).sum()
    }

    fn push_log(&mut self, msg: String) {
        self.last_sentences.push(msg);
        if self.last_sentences.len() > 12 {
            self.last_sentences.remove(0);
        }
        self.messages_received += 1;
        self.last_update = Some(Instant::now());
    }

    fn rtcm_rate_kbps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (self.total_rtcm_bytes as f64 * 8.0) / elapsed / 1000.0
        } else {
            0.0
        }
    }
}

/// Parse incoming data (auto-detects NMEA, RTCM3, or UBX)
fn parse_data(data: &[u8], state: &mut GpsState) {
    let mut i = 0;
    while i < data.len() {
        match data[i] {
            // NMEA sentence start
            b'$' => {
                if let Some(end) = data[i..].iter().position(|&b| b == b'\n') {
                    let line = String::from_utf8_lossy(&data[i..i + end]).to_string();
                    parse_nmea(&line, state);
                    state.mode = ReceiverMode::Rover;
                    i += end + 1;
                } else {
                    break;
                }
            }
            // RTCM3 preamble
            0xD3 => {
                if i + 3 <= data.len() {
                    let len = (((data[i + 1] as usize) & 0x03) << 8) | (data[i + 2] as usize);
                    let total_len = 3 + len + 3; // header + payload + CRC
                    if i + total_len <= data.len() {
                        parse_rtcm3(&data[i..i + total_len], state);
                        state.mode = ReceiverMode::BaseStation;
                        i += total_len;
                        continue;
                    }
                }
                i += 1;
            }
            // UBX preamble
            0xB5 if i + 1 < data.len() && data[i + 1] == 0x62 => {
                if i + 6 <= data.len() {
                    let len = u16::from_le_bytes([data[i + 4], data[i + 5]]) as usize;
                    let total_len = 6 + len + 2; // header + payload + checksum
                    if i + total_len <= data.len() {
                        parse_ubx(&data[i..i + total_len], state);
                        i += total_len;
                        continue;
                    }
                }
                i += 1;
            }
            _ => i += 1,
        }
    }

    state.bytes_received += data.len() as u64;
}

fn parse_nmea(line: &str, state: &mut GpsState) {
    let line = line.trim();
    if !line.starts_with('$') {
        return;
    }

    state.push_log(line.to_string());

    let sentence = line.split('*').next().unwrap_or(line);
    let fields: Vec<&str> = sentence.split(',').collect();

    if fields.is_empty() {
        return;
    }

    match fields[0] {
        "$GNGGA" | "$GPGGA" => parse_gga(&fields, state),
        "$GPGSV" => parse_gsv(&fields, state, Constellation::Gps),
        "$GLGSV" => parse_gsv(&fields, state, Constellation::Glonass),
        "$GAGSV" => parse_gsv(&fields, state, Constellation::Galileo),
        "$GBGSV" => parse_gsv(&fields, state, Constellation::BeiDou),
        _ => {}
    }
}

fn parse_gga(fields: &[&str], state: &mut GpsState) {
    if fields.len() < 15 {
        return;
    }

    if !fields[1].is_empty() && fields[1].len() >= 6 {
        state.utc_time = Some(format!(
            "{}:{}:{}",
            &fields[1][0..2],
            &fields[1][2..4],
            &fields[1][4..6]
        ));
    }

    state.fix_quality = fields[6].parse().unwrap_or(0).into();
    state.satellites_used = fields[7].parse().unwrap_or(0);
    state.hdop = fields[8].parse().unwrap_or(99.99);

    if state.fix_quality != FixQuality::NoFix {
        if let Some(lat) = parse_coordinate(fields[2], fields[3]) {
            state.latitude = Some(lat);
        }
        if let Some(lon) = parse_coordinate(fields[4], fields[5]) {
            state.longitude = Some(lon);
        }
        state.altitude = fields[9].parse().ok();
    }
}

fn parse_gsv(fields: &[&str], state: &mut GpsState, constellation: Constellation) {
    if fields.len() < 4 {
        return;
    }

    let mut sats = Vec::new();
    let mut i = 4;
    while i + 3 < fields.len() {
        let prn: u16 = fields[i].parse().unwrap_or(0);
        if prn > 0 {
            let sat = Satellite {
                prn,
                elevation: fields.get(i + 1).and_then(|s| s.parse().ok()),
                azimuth: fields.get(i + 2).and_then(|s| s.parse().ok()),
                snr: fields.get(i + 3).and_then(|s| s.parse().ok()),
            };
            sats.push(sat);
        }
        i += 4;
    }

    let msg_num: u8 = fields[2].parse().unwrap_or(1);
    if msg_num == 1 {
        state.satellites.insert(constellation, sats);
    } else if let Some(existing) = state.satellites.get_mut(&constellation) {
        existing.extend(sats);
    }
}

fn parse_coordinate(value: &str, direction: &str) -> Option<f64> {
    if value.is_empty() {
        return None;
    }

    let val: f64 = value.parse().ok()?;
    let degrees = (val / 100.0).floor();
    let minutes = val - (degrees * 100.0);
    let mut decimal = degrees + (minutes / 60.0);

    if direction == "S" || direction == "W" {
        decimal = -decimal;
    }

    Some(decimal)
}

fn parse_rtcm3(data: &[u8], state: &mut GpsState) {
    if data.len() < 6 {
        return;
    }

    // Extract message type from first 12 bits of payload
    let msg_type = ((data[3] as u16) << 4) | ((data[4] as u16) >> 4);
    let msg_len = data.len();

    // Update stats
    let stats = state.rtcm_messages.entry(msg_type).or_insert(RtcmStats {
        message_type: msg_type,
        ..Default::default()
    });
    stats.count += 1;
    stats.bytes += msg_len as u64;
    stats.last_seen = Some(Instant::now());

    state.total_rtcm_bytes += msg_len as u64;

    // Log message
    let name = rtcm_message_name(msg_type);
    state.push_log(format!("RTCM {} ({}) - {} bytes", msg_type, name, msg_len));
}

fn rtcm_message_name(msg_type: u16) -> &'static str {
    match msg_type {
        1005 => "ARP",
        1006 => "ARP+Height",
        1074 => "GPS MSM4",
        1075 => "GPS MSM5",
        1077 => "GPS MSM7",
        1084 => "GLO MSM4",
        1085 => "GLO MSM5",
        1087 => "GLO MSM7",
        1094 => "GAL MSM4",
        1095 => "GAL MSM5",
        1097 => "GAL MSM7",
        1124 => "BDS MSM4",
        1125 => "BDS MSM5",
        1127 => "BDS MSM7",
        1230 => "GLO Bias",
        _ => "Unknown",
    }
}

fn parse_ubx(data: &[u8], state: &mut GpsState) {
    if data.len() < 8 {
        return;
    }

    let class = data[2];
    let id = data[3];
    let payload = &data[6..data.len() - 2];

    // UBX-NAV-SVIN (0x01 0x3B) - Survey-in status (base station only)
    if class == 0x01 && id == 0x3B && payload.len() >= 40 {
        // NAV-SVIN is a base station message, so this is definitely base mode
        state.mode = ReceiverMode::BaseStation;

        state.survey.duration_s = u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
        state.survey.mean_accuracy_mm = u32::from_le_bytes([payload[28], payload[29], payload[30], payload[31]]);
        state.survey.observations = u32::from_le_bytes([payload[32], payload[33], payload[34], payload[35]]);
        state.survey.valid = payload[36] != 0;
        state.survey.active = payload[37] != 0;

        state.push_log(format!(
            "UBX-NAV-SVIN: {}s, {:.2}m accuracy, {} obs, valid={}",
            state.survey.duration_s,
            state.survey.mean_accuracy_mm as f64 / 1000.0,
            state.survey.observations,
            state.survey.valid
        ));
    }
}

fn run_monitor(port: &str, baud: u32) -> Result<()> {
    let state = Arc::new(Mutex::new(GpsState::default()));
    let state_clone = Arc::clone(&state);

    let port_path = port.to_string();
    std::thread::spawn(move || {
        if let Err(e) = run_serial_reader(&port_path, baud, state_clone) {
            eprintln!("Serial reader error: {}", e);
        }
    });

    run_tui(state, port)
}

fn run_serial_reader(port: &str, baud: u32, state: Arc<Mutex<GpsState>>) -> Result<()> {
    let mut serial = tokio_serial::new(port, baud)
        .timeout(Duration::from_millis(100))
        .open_native()
        .context("Failed to open serial port")?;

    let mut buf = [0u8; 1024];

    // Request UBX-NAV-SVIN periodically for survey status
    let poll_svin = build_ubx_message(0x01, 0x3B, &[]);
    let mut last_poll = Instant::now();

    loop {
        // Poll for survey status every second (in case we're in base station mode)
        if last_poll.elapsed() > Duration::from_secs(1) {
            let _ = serial.write_all(&poll_svin);
            last_poll = Instant::now();
        }

        match serial.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let mut state = state.lock().unwrap();
                parse_data(&buf[..n], &mut state);
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(_) => break,
        }
    }

    Ok(())
}

fn run_tui(state: Arc<Mutex<GpsState>>, port: &str) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = run_ui_loop(&mut terminal, state, port);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_ui_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: Arc<Mutex<GpsState>>,
    port: &str,
) -> Result<()> {
    loop {
        let state_snapshot = state.lock().unwrap().clone();
        terminal.draw(|f| ui(f, &state_snapshot, port))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return Ok(()),
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, state: &GpsState, port: &str) {
    match state.mode {
        ReceiverMode::BaseStation => ui_base_station(f, state, port),
        _ => ui_rover(f, state, port),
    }
}

// ============================================================================
// Rover Mode UI (NMEA)
// ============================================================================

fn ui_rover(f: &mut Frame, state: &GpsState, port: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(10),
            Constraint::Length(14),
        ])
        .split(f.area());

    render_header_rover(f, chunks[0], state, port);
    render_position(f, chunks[1], state);
    render_satellites(f, chunks[2], state);
    render_raw(f, chunks[3], state);
}

fn render_header_rover(f: &mut Frame, area: Rect, state: &GpsState, port: &str) {
    let fix_style = Style::default()
        .fg(state.fix_quality.color())
        .add_modifier(Modifier::BOLD);

    let status = if state.last_update.is_some() {
        format!(" {} ", state.fix_quality.label())
    } else {
        " CONNECTING... ".to_string()
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("GPS MONITOR", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  "),
        Span::styled("ROVER", Style::default().fg(Color::Blue).bold()),
        Span::raw("  "),
        Span::styled(status, fix_style),
        Span::raw("  "),
        Span::styled(port, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            format!("{} msgs", state.messages_received),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(header, area);
}

fn render_position(f: &mut Frame, area: Rect, state: &GpsState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let pos_text = if let (Some(lat), Some(lon)) = (state.latitude, state.longitude) {
        let alt = state
            .altitude
            .map(|a| format!("{:.1}m", a))
            .unwrap_or_default();
        vec![
            Line::from(vec![
                Span::styled("LAT ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:>12.8}°", lat),
                    Style::default().fg(Color::White).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("LON ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:>12.8}°", lon),
                    Style::default().fg(Color::White).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("ALT ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>12}", alt), Style::default().fg(Color::White)),
            ]),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "Waiting for fix...",
                Style::default().fg(Color::DarkGray),
            )),
            Line::raw(""),
            Line::raw(""),
        ]
    };

    let position = Paragraph::new(pos_text).block(
        Block::default()
            .title(" Position ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(position, chunks[0]);

    let utc = state.utc_time.as_deref().unwrap_or("--:--:--");
    let fix_info = vec![
        Line::from(vec![
            Span::styled("UTC  ", Style::default().fg(Color::DarkGray)),
            Span::styled(utc, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("SATS ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>2} used / {:>2} view",
                    state.satellites_used,
                    state.total_satellites_in_view()
                ),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("HDOP ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if state.hdop < 99.0 {
                    format!("{:.1}", state.hdop)
                } else {
                    "--.--".to_string()
                },
                Style::default().fg(if state.hdop < 2.0 {
                    Color::Green
                } else if state.hdop < 5.0 {
                    Color::Yellow
                } else {
                    Color::Red
                }),
            ),
        ]),
    ];

    let fix = Paragraph::new(fix_info).block(
        Block::default()
            .title(" Fix Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(fix, chunks[1]);
}

fn render_satellites(f: &mut Frame, area: Rect, state: &GpsState) {
    let constellations = [
        Constellation::Gps,
        Constellation::Glonass,
        Constellation::Galileo,
        Constellation::BeiDou,
    ];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    for (i, constellation) in constellations.iter().enumerate() {
        let sats = state
            .satellites
            .get(constellation)
            .cloned()
            .unwrap_or_default();

        let bars: Vec<Bar> = sats
            .iter()
            .filter(|s| s.snr.is_some() || s.prn > 0)
            .take(8)
            .map(|s| {
                let snr = s.snr.unwrap_or(0) as u64;
                let color = if snr >= 40 {
                    Color::Green
                } else if snr >= 25 {
                    Color::Yellow
                } else if snr > 0 {
                    Color::Red
                } else {
                    Color::DarkGray
                };
                Bar::default()
                    .value(snr)
                    .label(Line::from(format!("{}", s.prn)))
                    .style(Style::default().fg(color))
            })
            .collect();

        let title = format!(" {} ({}) ", constellation.label(), sats.len());

        let chart = BarChart::default()
            .block(
                Block::default()
                    .title(title)
                    .title_style(Style::default().fg(constellation.color()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .data(BarGroup::default().bars(&bars))
            .bar_width(3)
            .bar_gap(1)
            .max(50);

        f.render_widget(chart, chunks[i]);
    }
}

// ============================================================================
// Base Station Mode UI (RTCM)
// ============================================================================

fn ui_base_station(f: &mut Frame, state: &GpsState, port: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(6),  // Survey status
            Constraint::Min(12),    // RTCM messages
            Constraint::Length(14), // Raw log
        ])
        .split(f.area());

    render_header_base(f, chunks[0], state, port);
    render_survey_status(f, chunks[1], state);
    render_rtcm_messages(f, chunks[2], state);
    render_raw(f, chunks[3], state);
}

fn render_header_base(f: &mut Frame, area: Rect, state: &GpsState, port: &str) {
    let status = if state.survey.valid {
        (" TRANSMITTING ", Color::Green)
    } else if state.survey.active {
        (" SURVEYING ", Color::Yellow)
    } else {
        (" INITIALIZING ", Color::DarkGray)
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("GPS MONITOR", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  "),
        Span::styled("BASE STATION", Style::default().fg(Color::Magenta).bold()),
        Span::raw("  "),
        Span::styled(status.0, Style::default().fg(status.1).bold()),
        Span::raw("  "),
        Span::styled(port, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            format!("{:.1} kbps", state.rtcm_rate_kbps()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(header, area);
}

fn render_survey_status(f: &mut Frame, area: Rect, state: &GpsState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Survey progress
    let survey = &state.survey;
    let accuracy_m = survey.mean_accuracy_mm as f64 / 1000.0;

    let progress_text = if survey.valid {
        vec![
            Line::from(vec![
                Span::styled("STATUS ", Style::default().fg(Color::DarkGray)),
                Span::styled("COMPLETE", Style::default().fg(Color::Green).bold()),
            ]),
            Line::from(vec![
                Span::styled("ACCURACY ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:.3} m", accuracy_m), Style::default().fg(Color::Green)),
            ]),
        ]
    } else if survey.active {
        vec![
            Line::from(vec![
                Span::styled("ELAPSED ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} s", survey.duration_s), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("ACCURACY ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:.3} m", accuracy_m), Style::default().fg(Color::Yellow)),
            ]),
        ]
    } else {
        vec![
            Line::from(Span::styled("Waiting for survey data...", Style::default().fg(Color::DarkGray))),
            Line::raw(""),
        ]
    };

    let progress = Paragraph::new(progress_text).block(
        Block::default()
            .title(" Survey-In ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(progress, chunks[0]);

    // Stats
    let stats = vec![
        Line::from(vec![
            Span::styled("OBS ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", survey.observations), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("RTCM ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} msgs", state.rtcm_messages.values().map(|s| s.count).sum::<u32>()),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats).block(
        Block::default()
            .title(" Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(stats_widget, chunks[1]);
}

fn render_rtcm_messages(f: &mut Frame, area: Rect, state: &GpsState) {
    // Sort messages by type
    let mut messages: Vec<_> = state.rtcm_messages.values().collect();
    messages.sort_by_key(|m| m.message_type);

    let lines: Vec<Line> = messages
        .iter()
        .map(|m| {
            let name = rtcm_message_name(m.message_type);
            let age = m.last_seen
                .map(|t| format!("{:.1}s", t.elapsed().as_secs_f32()))
                .unwrap_or_else(|| "--".to_string());

            Line::from(vec![
                Span::styled(format!("{:>4} ", m.message_type), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<12} ", name), Style::default().fg(Color::White)),
                Span::styled(format!("{:>6} msgs  ", m.count), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>8} B  ", m.bytes), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("({} ago)", age), Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let rtcm = Paragraph::new(lines).block(
        Block::default()
            .title(" RTCM3 Messages ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(rtcm, area);
}

fn render_raw(f: &mut Frame, area: Rect, state: &GpsState) {
    let lines: Vec<Line> = state
        .last_sentences
        .iter()
        .map(|s| {
            let color = if s.contains("GGA") {
                Color::Cyan
            } else if s.contains("GSV") {
                Color::Yellow
            } else if s.contains("RMC") {
                Color::Green
            } else if s.starts_with("RTCM") {
                Color::Magenta
            } else if s.starts_with("UBX") {
                Color::Blue
            } else {
                Color::DarkGray
            };
            Line::from(Span::styled(s.clone(), Style::default().fg(color)))
        })
        .collect();

    let raw = Paragraph::new(lines).block(
        Block::default()
            .title(" Log ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(raw, area);
}

// ============================================================================
// Base Station Configuration
// ============================================================================

fn configure_base(
    port: &str,
    baud: u32,
    survey_duration: u32,
    survey_accuracy: f32,
    fixed_position: Option<(f64, f64, f64)>,
) -> Result<()> {
    println!("=== Configuring ZED-F9P as Base Station ===\n");

    let mut serial = open_serial(port, baud)?;

    // Disable NMEA on both USB and UART1
    println!("Disabling NMEA output (USB + UART1)...");
    // USB
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GGA_USB", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_RMC_USB", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GSV_USB", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GSA_USB", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GLL_USB", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_VTG_USB", 0)?;
    // UART1
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GGA_UART1", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_RMC_UART1", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GSV_UART1", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GSA_UART1", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GLL_UART1", 0)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_VTG_UART1", 0)?;

    // Enable RTCM on both USB and UART1
    println!("Enabling RTCM3 messages (USB + UART1)...");
    // USB
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1005_USB", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1074_USB", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1084_USB", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1094_USB", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1124_USB", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1230_USB", 5)?;
    // UART1
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1005_UART1", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1074_UART1", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1084_UART1", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1094_UART1", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1124_UART1", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-RTCM_3X_TYPE1230_UART1", 5)?;

    // Enable UBX-NAV-SVIN output for survey status (USB + UART1)
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-UBX_NAV_SVIN_USB", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-UBX_NAV_SVIN_UART1", 1)?;

    if let Some((lat, lon, alt)) = fixed_position {
        println!("Setting fixed position: {:.8}, {:.8}, {:.2}m", lat, lon, alt);
        configure_fixed_mode(&mut serial, lat, lon, alt)?;
    } else {
        println!("Starting survey-in mode...");
        println!("  Duration: {} seconds minimum", survey_duration);
        println!("  Accuracy: {} meters target", survey_accuracy);
        configure_survey_mode(&mut serial, survey_duration, survey_accuracy)?;
    }

    // Verify configuration by reopening port and checking output
    println!("\nVerifying configuration...");
    drop(serial); // Close write handle

    std::thread::sleep(Duration::from_millis(500));

    // Reopen port for reading to verify
    let mut verify_port = tokio_serial::new(port, baud)
        .timeout(Duration::from_millis(1000))
        .open_native()
        .context("Failed to reopen port for verification")?;

    let mut buf = [0u8; 512];
    let mut verified = false;

    // Try reading a few times to collect data
    for _ in 0..3 {
        match verify_port.read(&mut buf) {
            Ok(n) if n > 0 => {
                let has_rtcm = buf[..n].contains(&0xD3);
                let has_ubx = buf[..n].windows(2).any(|w| w[0] == 0xB5 && w[1] == 0x62);
                let has_nmea = buf[..n].contains(&b'$');

                if has_rtcm {
                    println!("  [ok] RTCM output detected (base station mode active)");
                    verified = true;
                    break;
                } else if has_ubx && !has_nmea {
                    println!("  [ok] UBX output detected, NMEA disabled (base station mode active)");
                    verified = true;
                    break;
                } else if has_nmea {
                    println!("  [!!] NMEA still active (configuration may have failed)");
                    println!("       Try power cycling the module and running configure-base again.");
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => {}
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    if !verified {
        // No data received: either no satellites or config worked
        println!("  [ok] No NMEA detected (RTCM requires satellite lock to output)");
    }

    println!("\n=== Configuration Complete (saved to flash) ===");
    println!("\nMonitor with: muni gps monitor --port {}", port);
    println!("Start NTRIP caster with: docker compose --profile rtk up -d ntrip");

    Ok(())
}

fn configure_survey_mode(serial: &mut Box<dyn Write>, duration: u32, accuracy: f32) -> Result<()> {
    send_ubx_cfg(serial, "CFG-TMODE-MODE", 1)?;
    send_ubx_cfg(serial, "CFG-TMODE-SVIN_MIN_DUR", duration)?;
    let acc_01mm = (accuracy * 10000.0) as u32;
    send_ubx_cfg(serial, "CFG-TMODE-SVIN_ACC_LIMIT", acc_01mm)?;
    Ok(())
}

fn configure_fixed_mode(serial: &mut Box<dyn Write>, lat: f64, lon: f64, alt: f64) -> Result<()> {
    send_ubx_cfg(serial, "CFG-TMODE-MODE", 2)?;
    send_ubx_cfg(serial, "CFG-TMODE-POS_TYPE", 1)?;

    let lat_i = (lat * 1e7) as i32;
    let lon_i = (lon * 1e7) as i32;
    let alt_cm = (alt * 100.0) as i32;

    send_ubx_cfg(serial, "CFG-TMODE-LAT", lat_i as u32)?;
    send_ubx_cfg(serial, "CFG-TMODE-LON", lon_i as u32)?;
    send_ubx_cfg(serial, "CFG-TMODE-HEIGHT", alt_cm as u32)?;

    Ok(())
}

// ============================================================================
// Rover Configuration
// ============================================================================

fn configure_rover(port: &str, baud: u32, rate: u8) -> Result<()> {
    println!("=== Configuring ZED-F9P as Rover ===\n");

    let mut serial = open_serial(port, baud)?;

    println!("Setting baud rate to {}...", baud);
    send_ubx_cfg(&mut serial, "CFG-UART1-BAUDRATE", baud)?;

    println!("Setting update rate to {} Hz...", rate);
    let meas_rate = 1000 / rate as u32;
    send_ubx_cfg(&mut serial, "CFG-RATE-MEAS", meas_rate)?;
    send_ubx_cfg(&mut serial, "CFG-RATE-NAV", 1)?;

    println!("Enabling NMEA output...");
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_GGA_UART1", 1)?;
    send_ubx_cfg(&mut serial, "CFG-MSGOUT-NMEA_ID_RMC_UART1", 1)?;

    println!("Setting rover mode (disable TMODE)...");
    send_ubx_cfg(&mut serial, "CFG-TMODE-MODE", 0)?;

    println!("\n=== Configuration Complete ===");
    println!("\nRover will output NMEA at {} Hz.", rate);
    println!("Connect NTRIP corrections to UART1 RX for RTK.");

    Ok(())
}

// ============================================================================
// UBX Protocol Helpers
// ============================================================================

fn open_serial(port: &str, baud: u32) -> Result<Box<dyn Write>> {
    for try_baud in [baud, 38400, 9600, 115200] {
        if let Ok(s) = tokio_serial::new(port, try_baud)
            .timeout(Duration::from_secs(1))
            .open_native()
        {
            println!("Connected to {} at {} baud", port, try_baud);
            return Ok(Box::new(s));
        }
    }
    anyhow::bail!("Failed to open serial port {}", port)
}

fn send_ubx_cfg(serial: &mut Box<dyn Write>, key: &str, value: u32) -> Result<()> {
    let key_id: u32 = match key {
        // UART settings
        "CFG-UART1-BAUDRATE" => 0x40520001,
        "CFG-RATE-MEAS" => 0x30210001,
        "CFG-RATE-NAV" => 0x30210002,

        // Time mode
        "CFG-TMODE-MODE" => 0x20030001,
        "CFG-TMODE-POS_TYPE" => 0x20030002,
        "CFG-TMODE-LAT" => 0x40030009,
        "CFG-TMODE-LON" => 0x4003000A,
        "CFG-TMODE-HEIGHT" => 0x4003000B,
        "CFG-TMODE-SVIN_MIN_DUR" => 0x40030010,
        "CFG-TMODE-SVIN_ACC_LIMIT" => 0x40030011,

        // NMEA on UART1
        "CFG-MSGOUT-NMEA_ID_GGA_UART1" => 0x209100BB,
        "CFG-MSGOUT-NMEA_ID_RMC_UART1" => 0x209100AC,
        "CFG-MSGOUT-NMEA_ID_GSV_UART1" => 0x209100C5,
        "CFG-MSGOUT-NMEA_ID_GSA_UART1" => 0x209100C0,
        "CFG-MSGOUT-NMEA_ID_GLL_UART1" => 0x209100CA,
        "CFG-MSGOUT-NMEA_ID_VTG_UART1" => 0x209100B1,

        // NMEA on USB (UART1 + 2)
        "CFG-MSGOUT-NMEA_ID_GGA_USB" => 0x209100BD,
        "CFG-MSGOUT-NMEA_ID_RMC_USB" => 0x209100AE,
        "CFG-MSGOUT-NMEA_ID_GSV_USB" => 0x209100C7,
        "CFG-MSGOUT-NMEA_ID_GSA_USB" => 0x209100C2,
        "CFG-MSGOUT-NMEA_ID_GLL_USB" => 0x209100CC,
        "CFG-MSGOUT-NMEA_ID_VTG_USB" => 0x209100B3,

        // RTCM on UART1
        "CFG-MSGOUT-RTCM_3X_TYPE1005_UART1" => 0x209102BD,
        "CFG-MSGOUT-RTCM_3X_TYPE1074_UART1" => 0x2091035E,
        "CFG-MSGOUT-RTCM_3X_TYPE1084_UART1" => 0x20910363,
        "CFG-MSGOUT-RTCM_3X_TYPE1094_UART1" => 0x20910368,
        "CFG-MSGOUT-RTCM_3X_TYPE1124_UART1" => 0x2091036D,
        "CFG-MSGOUT-RTCM_3X_TYPE1230_UART1" => 0x20910303,

        // RTCM on USB
        "CFG-MSGOUT-RTCM_3X_TYPE1005_USB" => 0x209102BF,
        "CFG-MSGOUT-RTCM_3X_TYPE1074_USB" => 0x20910360,
        "CFG-MSGOUT-RTCM_3X_TYPE1084_USB" => 0x20910365,
        "CFG-MSGOUT-RTCM_3X_TYPE1094_USB" => 0x2091036A,
        "CFG-MSGOUT-RTCM_3X_TYPE1124_USB" => 0x2091036F,
        "CFG-MSGOUT-RTCM_3X_TYPE1230_USB" => 0x20910305,

        // UBX on UART1
        "CFG-MSGOUT-UBX_NAV_SVIN_UART1" => 0x20910088,
        // UBX on USB
        "CFG-MSGOUT-UBX_NAV_SVIN_USB" => 0x2091008A,

        _ => {
            println!("  Unknown key: {}", key);
            return Ok(());
        }
    };

    let size = match (key_id >> 28) & 0x7 {
        1 => 1,
        2 => 1,
        3 => 2,
        4 => 4,
        5 => 8,
        _ => 4,
    };

    // layers = 7: RAM (1) + BBR (2) + Flash (4) = persist across power cycles
    let mut payload = vec![0x00, 0x07, 0x00, 0x00];
    payload.extend_from_slice(&key_id.to_le_bytes());

    match size {
        1 => payload.push(value as u8),
        2 => payload.extend_from_slice(&(value as u16).to_le_bytes()),
        4 => payload.extend_from_slice(&value.to_le_bytes()),
        _ => payload.extend_from_slice(&value.to_le_bytes()),
    }

    let msg = build_ubx_message(0x06, 0x8A, &payload);
    serial.write_all(&msg)?;
    serial.flush()?;

    std::thread::sleep(Duration::from_millis(50));
    Ok(())
}

fn build_ubx_message(class: u8, id: u8, payload: &[u8]) -> Vec<u8> {
    let mut msg = vec![0xB5, 0x62, class, id];
    msg.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    msg.extend_from_slice(payload);

    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for byte in &msg[2..] {
        ck_a = ck_a.wrapping_add(*byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }
    msg.push(ck_a);
    msg.push(ck_b);

    msg
}
