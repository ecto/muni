//! GPS/GNSS receiver interface for bvr.
//!
//! Parses NMEA sentences from a serial GPS receiver and provides
//! position updates as `GpsCoord` structs.

use std::io::{BufRead, BufReader};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::watch;
use tracing::{debug, error, info, trace, warn};
use types::GpsCoord;

#[derive(Error, Debug)]
pub enum GpsError {
    #[error("Serial port error: {0}")]
    Serial(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("No fix available")]
    NoFix,
}

/// GPS receiver configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Serial port path (e.g., "/dev/ttyUSB0", "/dev/ttyACM0")
    pub port: String,
    /// Baud rate (typically 9600 or 115200)
    pub baud_rate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: "/dev/ttyUSB0".into(),
            baud_rate: 9600,
        }
    }
}

/// GPS fix quality from GGA sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixQuality {
    /// No fix
    Invalid = 0,
    /// GPS fix
    GpsFix = 1,
    /// Differential GPS fix
    DgpsFix = 2,
    /// PPS fix
    PpsFix = 3,
    /// RTK fixed
    RtkFixed = 4,
    /// RTK float
    RtkFloat = 5,
    /// Estimated (dead reckoning)
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
            _ => Self::Invalid,
        }
    }
}

/// Extended GPS state with fix information.
#[derive(Debug, Clone, Default)]
pub struct GpsState {
    /// Current coordinates (if fix is valid)
    pub coord: Option<GpsCoord>,
    /// Fix quality
    pub fix_quality: u8,
    /// Number of satellites used
    pub satellites: u8,
    /// Horizontal dilution of precision
    pub hdop: f32,
}

/// GPS reader that parses NMEA sentences from a serial port.
pub struct GpsReader {
    config: Config,
}

impl GpsReader {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the GPS reader, sending updates to the provided channel.
    ///
    /// This spawns a blocking thread for serial I/O.
    pub fn spawn(
        self,
        tx: watch::Sender<GpsState>,
    ) -> Result<std::thread::JoinHandle<()>, GpsError> {
        let config = self.config.clone();

        let handle = std::thread::spawn(move || {
            if let Err(e) = run_reader(config, tx) {
                error!(?e, "GPS reader error");
            }
        });

        Ok(handle)
    }
}

/// Internal reader loop.
fn run_reader(config: Config, tx: watch::Sender<GpsState>) -> Result<(), GpsError> {
    info!(port = %config.port, baud = config.baud_rate, "Opening GPS serial port");

    // Open serial port
    let port = tokio_serial::new(&config.port, config.baud_rate)
        .timeout(Duration::from_secs(2))
        .open_native()
        .map_err(|e| GpsError::Serial(e.to_string()))?;

    let mut reader = BufReader::new(port);
    let mut line = String::new();
    let mut state = GpsState::default();

    info!("GPS reader started");

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF
                warn!("GPS serial port closed");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                trace!(sentence = %trimmed, "NMEA");

                // Parse NMEA sentence
                if let Some(new_state) = parse_nmea_sentence(trimmed, &state) {
                    state = new_state;
                    if tx.send(state.clone()).is_err() {
                        // Receiver dropped
                        break;
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is OK, just continue
                continue;
            }
            Err(e) => {
                error!(?e, "GPS read error");
                break;
            }
        }
    }

    info!("GPS reader stopped");
    Ok(())
}

/// Parse an NMEA sentence and update state if relevant.
fn parse_nmea_sentence(sentence: &str, current: &GpsState) -> Option<GpsState> {
    // Validate checksum
    if !validate_checksum(sentence) {
        trace!("Invalid checksum");
        return None;
    }

    // Remove checksum suffix
    let sentence = sentence.split('*').next()?;

    // Parse based on sentence type
    if sentence.starts_with("$GPGGA") || sentence.starts_with("$GNGGA") {
        parse_gga(sentence, current)
    } else if sentence.starts_with("$GPRMC") || sentence.starts_with("$GNRMC") {
        parse_rmc(sentence, current)
    } else {
        None
    }
}

/// Validate NMEA checksum.
fn validate_checksum(sentence: &str) -> bool {
    if !sentence.starts_with('$') {
        return false;
    }

    let parts: Vec<&str> = sentence[1..].split('*').collect();
    if parts.len() != 2 {
        return false;
    }

    let data = parts[0];
    let checksum_str = parts[1].trim();

    let expected = match u8::from_str_radix(checksum_str, 16) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let computed: u8 = data.bytes().fold(0, |acc, b| acc ^ b);
    computed == expected
}

/// Parse GGA sentence (Global Positioning System Fix Data).
///
/// Format: $GPGGA,hhmmss.ss,llll.ll,a,yyyyy.yy,a,x,xx,x.x,x.x,M,x.x,M,x.x,xxxx*hh
fn parse_gga(sentence: &str, _current: &GpsState) -> Option<GpsState> {
    let fields: Vec<&str> = sentence.split(',').collect();
    if fields.len() < 15 {
        return None;
    }

    // Fix quality (field 6)
    let fix_quality: u8 = fields[6].parse().unwrap_or(0);
    if fix_quality == 0 {
        // No fix - return state with no coord
        return Some(GpsState {
            coord: None,
            fix_quality: 0,
            satellites: fields[7].parse().unwrap_or(0),
            hdop: fields[8].parse().unwrap_or(99.99),
        });
    }

    // Latitude (field 2-3): ddmm.mmmm,N/S
    let lat = parse_coordinate(fields[2], fields[3], true)?;

    // Longitude (field 4-5): dddmm.mmmm,E/W
    let lon = parse_coordinate(fields[4], fields[5], false)?;

    // Altitude (field 9): meters above MSL
    let alt: f64 = fields[9].parse().unwrap_or(0.0);

    // HDOP (field 8)
    let hdop: f32 = fields[8].parse().unwrap_or(99.99);

    // Satellites (field 7)
    let satellites: u8 = fields[7].parse().unwrap_or(0);

    // Estimate horizontal accuracy from HDOP (rough: HDOP * 2.5m typical)
    let accuracy = hdop * 2.5;

    let coord = GpsCoord {
        lat,
        lon,
        alt,
        accuracy,
    };

    debug!(
        lat = coord.lat,
        lon = coord.lon,
        alt = coord.alt,
        fix = fix_quality,
        sats = satellites,
        hdop = hdop,
        "GPS fix"
    );

    Some(GpsState {
        coord: Some(coord),
        fix_quality,
        satellites,
        hdop,
    })
}

/// Parse RMC sentence (Recommended Minimum Navigation Information).
///
/// Format: $GPRMC,hhmmss.ss,A,llll.ll,a,yyyyy.yy,a,x.x,x.x,ddmmyy,x.x,a*hh
fn parse_rmc(sentence: &str, current: &GpsState) -> Option<GpsState> {
    let fields: Vec<&str> = sentence.split(',').collect();
    if fields.len() < 12 {
        return None;
    }

    // Status (field 2): A=Active, V=Void
    if fields[2] != "A" {
        return Some(GpsState {
            coord: None,
            fix_quality: 0,
            satellites: current.satellites,
            hdop: current.hdop,
        });
    }

    // Latitude (field 3-4): ddmm.mmmm,N/S
    let lat = parse_coordinate(fields[3], fields[4], true)?;

    // Longitude (field 5-6): dddmm.mmmm,E/W
    let lon = parse_coordinate(fields[5], fields[6], false)?;

    // RMC doesn't have altitude, keep current if available
    let alt = current.coord.as_ref().map(|c| c.alt).unwrap_or(0.0);
    let accuracy = current.coord.as_ref().map(|c| c.accuracy).unwrap_or(0.0);

    let coord = GpsCoord {
        lat,
        lon,
        alt,
        accuracy,
    };

    Some(GpsState {
        coord: Some(coord),
        fix_quality: current.fix_quality.max(1), // At least GPS fix if RMC is valid
        satellites: current.satellites,
        hdop: current.hdop,
    })
}

/// Parse a coordinate from NMEA format to decimal degrees.
///
/// NMEA format: ddmm.mmmm (latitude) or dddmm.mmmm (longitude)
fn parse_coordinate(value: &str, direction: &str, is_latitude: bool) -> Option<f64> {
    if value.is_empty() {
        return None;
    }

    let value: f64 = value.parse().ok()?;

    // Split into degrees and minutes
    // NMEA format: ddmm.mmmm (lat) or dddmm.mmmm (lon)
    // Both use the same formula: degrees = floor(value/100), minutes = remainder
    let _ = is_latitude; // Latitude/longitude have same parsing logic
    let degrees = (value / 100.0).floor();
    let minutes = value - (degrees * 100.0);

    let mut decimal = degrees + (minutes / 60.0);

    // Apply direction
    if direction == "S" || direction == "W" {
        decimal = -decimal;
    }

    Some(decimal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_checksum() {
        // Valid GGA sentence (checksum is XOR of bytes between $ and *)
        assert!(validate_checksum(
            "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,*4F"
        ));

        // Invalid checksum
        assert!(!validate_checksum(
            "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,*00"
        ));

        // Missing checksum
        assert!(!validate_checksum("$GPGGA,123519,4807.038,N,01131.000,E"));
    }

    #[test]
    fn test_parse_coordinate() {
        // Latitude: 48° 07.038' N
        let lat = parse_coordinate("4807.038", "N", true).unwrap();
        assert!((lat - 48.1173).abs() < 0.001);

        // Longitude: 11° 31.000' E
        let lon = parse_coordinate("01131.000", "E", false).unwrap();
        assert!((lon - 11.5167).abs() < 0.001);

        // Southern latitude
        let lat_s = parse_coordinate("3723.456", "S", true).unwrap();
        assert!(lat_s < 0.0);

        // Western longitude
        let lon_w = parse_coordinate("12212.345", "W", false).unwrap();
        assert!(lon_w < 0.0);
    }

    #[test]
    fn test_parse_gga() {
        let state = GpsState::default();
        let sentence = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,";

        let result = parse_gga(sentence, &state).unwrap();
        assert!(result.coord.is_some());

        let coord = result.coord.unwrap();
        assert!((coord.lat - 48.1173).abs() < 0.001);
        assert!((coord.lon - 11.5167).abs() < 0.001);
        assert!((coord.alt - 545.4).abs() < 0.1);
        assert_eq!(result.fix_quality, 1);
        assert_eq!(result.satellites, 8);
    }

    #[test]
    fn test_parse_gga_no_fix() {
        let state = GpsState::default();
        let sentence = "$GPGGA,123519,,,,,0,00,,,,,,,";

        let result = parse_gga(sentence, &state).unwrap();
        assert!(result.coord.is_none());
        assert_eq!(result.fix_quality, 0);
    }
}
