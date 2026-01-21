/**
 * Hook for loading 3D map data (Gaussian splats).
 *
 * Fetches map list from map-api, auto-selects map by GPS bounds,
 * and provides splat URL for rendering.
 */

import { useState, useEffect, useCallback } from "react";
import { useConsoleStore } from "@/store";
import {
  type GpsCoord,
  type CoordinateTransform,
  createTransform,
} from "@/lib/coordinates";

/** Map summary from map-api */
interface MapSummary {
  id: string;
  name: string;
  bounds: {
    minLat: number;
    maxLat: number;
    minLon: number;
    maxLon: number;
  };
  version: number;
  updatedAt: string;
  sessionCount: number;
  hasSplat: boolean;
  thumbnailUrl: string | null;
}

/** Map manifest from map-api */
interface MapManifest {
  id: string;
  name: string;
  bounds: {
    minLat: number;
    maxLat: number;
    minLon: number;
    maxLon: number;
  };
  origin: {
    lat: number;
    lon: number;
  };
  version: number;
  createdAt: string;
  updatedAt: string;
  sessions: string[];
  assets: {
    splat?: string;
    pointcloud?: string;
    mesh?: string;
    thumbnail?: string;
  };
}

interface UseMap3DResult {
  /** List of available maps */
  maps: MapSummary[];
  /** Currently loaded map manifest */
  manifest: MapManifest | null;
  /** URL to download splat file */
  splatUrl: string | null;
  /** Coordinate transform for this map */
  transform: CoordinateTransform | null;
  /** Currently selected map ID */
  selectedMapId: string | null;
  /** Whether map data is loading */
  loading: boolean;
  /** Error message if any */
  error: string | null;
  /** Manually select a map by ID */
  selectMap: (id: string) => void;
  /** Reload maps list */
  refresh: () => void;
}

/** Map API base URL (from docker-compose) */
const MAP_API_URL = "/api/maps";

export function useMap3D(): UseMap3DResult {
  const [maps, setMaps] = useState<MapSummary[]>([]);
  const [manifest, setManifest] = useState<MapManifest | null>(null);
  const [selectedMapId, setSelectedMapId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Get GPS position from telemetry for auto-selection
  const gpsStatus = useConsoleStore((s) => s.gpsStatus);

  // Fetch maps list
  const fetchMaps = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await fetch(MAP_API_URL);
      if (!response.ok) {
        throw new Error(`Failed to fetch maps: ${response.status}`);
      }

      const data = await response.json();
      setMaps(data.maps || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch maps");
      setMaps([]);
    } finally {
      setLoading(false);
    }
  }, []);

  // Fetch map manifest
  const fetchManifest = useCallback(async (mapId: string) => {
    try {
      setLoading(true);
      setError(null);

      const response = await fetch(`${MAP_API_URL}/${mapId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch map manifest: ${response.status}`);
      }

      const data = await response.json();
      setManifest(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch manifest");
      setManifest(null);
    } finally {
      setLoading(false);
    }
  }, []);

  // Select a map
  const selectMap = useCallback(
    (id: string) => {
      setSelectedMapId(id);
      fetchManifest(id);
    },
    [fetchManifest]
  );

  // Auto-select map by GPS bounds
  useEffect(() => {
    if (!gpsStatus || !gpsStatus.latitude || !gpsStatus.longitude) return;
    if (maps.length === 0) return;
    if (selectedMapId) return; // Already selected

    const currentPos: GpsCoord = {
      lat: gpsStatus.latitude,
      lon: gpsStatus.longitude,
    };

    // Find map that contains current position
    const matchingMap = maps.find((m) => {
      if (!m.hasSplat) return false;
      return (
        currentPos.lat >= m.bounds.minLat &&
        currentPos.lat <= m.bounds.maxLat &&
        currentPos.lon >= m.bounds.minLon &&
        currentPos.lon <= m.bounds.maxLon
      );
    });

    if (matchingMap) {
      selectMap(matchingMap.id);
    }
  }, [gpsStatus, maps, selectedMapId, selectMap]);

  // Initial fetch
  useEffect(() => {
    fetchMaps();
  }, [fetchMaps]);

  // Build splat URL
  const splatUrl =
    manifest && manifest.assets.splat
      ? `${MAP_API_URL}/${manifest.id}/splat.ply`
      : null;

  // Build coordinate transform
  const transform: CoordinateTransform | null = manifest?.origin
    ? createTransform({
        lat: manifest.origin.lat,
        lon: manifest.origin.lon,
      })
    : null;

  return {
    maps,
    manifest,
    splatUrl,
    transform,
    selectedMapId,
    loading,
    error,
    selectMap,
    refresh: fetchMaps,
  };
}
