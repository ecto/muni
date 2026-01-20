//! Geographic types for depot services.

use serde::{Deserialize, Serialize};

/// GPS bounding box in WGS84 coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GpsBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl GpsBounds {
    /// Create a new bounding box from coordinates.
    pub fn new(min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Self {
        Self {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        }
    }

    /// Get the center point of the bounding box.
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_lat + self.max_lat) / 2.0,
            (self.min_lon + self.max_lon) / 2.0,
        )
    }

    /// Check if this bounding box overlaps with another.
    pub fn overlaps(&self, other: &GpsBounds) -> bool {
        self.min_lat <= other.max_lat
            && self.max_lat >= other.min_lat
            && self.min_lon <= other.max_lon
            && self.max_lon >= other.min_lon
    }

    /// Expand this bounding box to include another.
    pub fn expand(&mut self, other: &GpsBounds) {
        self.min_lat = self.min_lat.min(other.min_lat);
        self.max_lat = self.max_lat.max(other.max_lat);
        self.min_lon = self.min_lon.min(other.min_lon);
        self.max_lon = self.max_lon.max(other.max_lon);
    }

    /// Check if the bounding box is valid (non-zero area).
    pub fn is_valid(&self) -> bool {
        self.min_lat < self.max_lat && self.min_lon < self.max_lon
    }
}

/// GPS coordinate in WGS84.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct GpsCoord {
    pub lat: f64,
    pub lon: f64,
}

impl GpsCoord {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_center() {
        let bounds = GpsBounds::new(42.0, 43.0, -72.0, -71.0);
        let (lat, lon) = bounds.center();
        assert!((lat - 42.5).abs() < 0.001);
        assert!((lon - -71.5).abs() < 0.001);
    }

    #[test]
    fn test_bounds_overlaps() {
        let a = GpsBounds::new(42.0, 43.0, -72.0, -71.0);
        let b = GpsBounds::new(42.5, 43.5, -71.5, -70.5);
        let c = GpsBounds::new(44.0, 45.0, -70.0, -69.0);

        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
        assert!(!a.overlaps(&c));
        assert!(!c.overlaps(&a));
    }

    #[test]
    fn test_bounds_expand() {
        let mut a = GpsBounds::new(42.0, 43.0, -72.0, -71.0);
        let b = GpsBounds::new(42.5, 44.0, -73.0, -70.0);
        a.expand(&b);

        assert_eq!(a.min_lat, 42.0);
        assert_eq!(a.max_lat, 44.0);
        assert_eq!(a.min_lon, -73.0);
        assert_eq!(a.max_lon, -70.0);
    }

    #[test]
    fn test_bounds_is_valid() {
        assert!(GpsBounds::new(42.0, 43.0, -72.0, -71.0).is_valid());
        assert!(!GpsBounds::new(43.0, 42.0, -72.0, -71.0).is_valid());
        assert!(!GpsBounds::default().is_valid());
    }

    #[test]
    fn test_gps_coord_serde() {
        let coord = GpsCoord::new(42.3601, -71.0589);
        let json = serde_json::to_string(&coord).unwrap();
        let decoded: GpsCoord = serde_json::from_str(&json).unwrap();
        assert!((decoded.lat - coord.lat).abs() < 0.0001);
        assert!((decoded.lon - coord.lon).abs() < 0.0001);
    }
}
