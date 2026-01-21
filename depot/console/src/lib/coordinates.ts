/**
 * Coordinate transformations for Muni.
 *
 * Coordinate systems:
 * - WGS84: GPS coordinates (latitude, longitude, altitude)
 * - ENU: Local East-North-Up meters from a reference point
 * - Three.js: Right-handed Y-up (X=right, Y=up, Z=toward camera)
 *
 * The physics system uses ENU where:
 * - X = East (meters)
 * - Y = North (meters)
 * - Theta = heading (radians, 0 = East, CCW positive)
 *
 * Three.js mapping:
 * - ENU.X -> Three.js X
 * - ENU.Y -> Three.js -Z (North is forward/negative Z)
 * - Theta -> rotation around Y axis
 */

/** WGS84 coordinate (degrees) */
export interface GpsCoord {
  lat: number;
  lon: number;
  alt?: number;
}

/** Local ENU coordinate (meters) */
export interface EnuCoord {
  x: number;
  y: number;
  z?: number;
}

/** Three.js coordinate */
export interface ThreeCoord {
  x: number;
  y: number;
  z: number;
}

/** Transform configuration for a map origin */
export interface CoordinateTransform {
  /** Reference point in WGS84 */
  origin: GpsCoord;
  /** Meters per degree latitude at this origin */
  metersPerDegLat: number;
  /** Meters per degree longitude at this origin */
  metersPerDegLon: number;
}

// WGS84 ellipsoid constants
const WGS84_A = 6378137.0; // Semi-major axis (meters)
const WGS84_E2 = 0.00669437999014; // Eccentricity squared

/**
 * Create a coordinate transform for a given origin.
 */
export function createTransform(origin: GpsCoord): CoordinateTransform {
  const latRad = (origin.lat * Math.PI) / 180;

  // Radius of curvature in the meridian
  const Rm =
    (WGS84_A * (1 - WGS84_E2)) / Math.pow(1 - WGS84_E2 * Math.sin(latRad) ** 2, 1.5);

  // Radius of curvature in the prime vertical
  const Rn = WGS84_A / Math.sqrt(1 - WGS84_E2 * Math.sin(latRad) ** 2);

  // Meters per degree
  const metersPerDegLat = (Math.PI / 180) * Rm;
  const metersPerDegLon = (Math.PI / 180) * Rn * Math.cos(latRad);

  return {
    origin,
    metersPerDegLat,
    metersPerDegLon,
  };
}

/**
 * Convert GPS coordinate to local ENU meters relative to transform origin.
 */
export function gpsToEnu(transform: CoordinateTransform, gps: GpsCoord): EnuCoord {
  const dLat = gps.lat - transform.origin.lat;
  const dLon = gps.lon - transform.origin.lon;

  return {
    x: dLon * transform.metersPerDegLon, // East
    y: dLat * transform.metersPerDegLat, // North
    z: (gps.alt ?? 0) - (transform.origin.alt ?? 0), // Up
  };
}

/**
 * Convert local ENU meters to GPS coordinate.
 */
export function enuToGps(transform: CoordinateTransform, enu: EnuCoord): GpsCoord {
  const dLat = enu.y / transform.metersPerDegLat;
  const dLon = enu.x / transform.metersPerDegLon;

  return {
    lat: transform.origin.lat + dLat,
    lon: transform.origin.lon + dLon,
    alt: (transform.origin.alt ?? 0) + (enu.z ?? 0),
  };
}

/**
 * Convert ENU coordinate to Three.js coordinate.
 *
 * ENU: X=East, Y=North, Z=Up
 * Three.js: X=Right, Y=Up, Z=Toward camera
 *
 * Mapping:
 * - ENU.X (East) -> Three.js X
 * - ENU.Y (North) -> Three.js -Z
 * - ENU.Z (Up) -> Three.js Y
 */
export function enuToThree(enu: EnuCoord): ThreeCoord {
  return {
    x: enu.x,
    y: enu.z ?? 0,
    z: -enu.y,
  };
}

/**
 * Convert Three.js coordinate to ENU.
 */
export function threeToEnu(three: ThreeCoord): EnuCoord {
  return {
    x: three.x,
    y: -three.z,
    z: three.y,
  };
}

/**
 * Convert GPS to Three.js directly.
 */
export function gpsToThree(transform: CoordinateTransform, gps: GpsCoord): ThreeCoord {
  return enuToThree(gpsToEnu(transform, gps));
}

/**
 * Check if a GPS coordinate is within bounds.
 */
export function isInBounds(
  gps: GpsCoord,
  bounds: { minLat: number; maxLat: number; minLon: number; maxLon: number }
): boolean {
  return (
    gps.lat >= bounds.minLat &&
    gps.lat <= bounds.maxLat &&
    gps.lon >= bounds.minLon &&
    gps.lon <= bounds.maxLon
  );
}

/**
 * Calculate distance between two GPS coordinates using Haversine formula.
 */
export function gpsDistance(a: GpsCoord, b: GpsCoord): number {
  const R = 6371000; // Earth radius in meters
  const dLat = ((b.lat - a.lat) * Math.PI) / 180;
  const dLon = ((b.lon - a.lon) * Math.PI) / 180;
  const lat1 = (a.lat * Math.PI) / 180;
  const lat2 = (b.lat * Math.PI) / 180;

  const sinDLat = Math.sin(dLat / 2);
  const sinDLon = Math.sin(dLon / 2);
  const h = sinDLat * sinDLat + Math.cos(lat1) * Math.cos(lat2) * sinDLon * sinDLon;

  return 2 * R * Math.asin(Math.sqrt(h));
}
