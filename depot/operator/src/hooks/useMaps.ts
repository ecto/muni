import { useState, useEffect, useCallback } from "react";
import type { MapSummary, MapManifest } from "@/lib/types";

const MAP_API_URL =
  import.meta.env.VITE_MAP_API_URL || "http://localhost:4870";

interface MapsState {
  maps: MapSummary[];
  loading: boolean;
  error: string | null;
}

/**
 * Hook to fetch and manage maps from the map-api service.
 */
export function useMaps() {
  const [state, setState] = useState<MapsState>({
    maps: [],
    loading: true,
    error: null,
  });

  const fetchMaps = useCallback(async () => {
    setState((s) => ({ ...s, loading: true, error: null }));

    try {
      const response = await fetch(`${MAP_API_URL}/maps`);
      if (!response.ok) {
        throw new Error(`Failed to fetch maps: ${response.statusText}`);
      }

      const data = await response.json();
      setState({
        maps: data.maps || [],
        loading: false,
        error: null,
      });
    } catch (err) {
      setState((s) => ({
        ...s,
        loading: false,
        error: err instanceof Error ? err.message : "Unknown error",
      }));
    }
  }, []);

  useEffect(() => {
    fetchMaps();

    // Refresh every 30 seconds
    const interval = setInterval(fetchMaps, 30000);
    return () => clearInterval(interval);
  }, [fetchMaps]);

  return {
    ...state,
    refresh: fetchMaps,
  };
}

/**
 * Hook to fetch a single map's full manifest.
 */
export function useMapDetails(mapId: string | null) {
  const [manifest, setManifest] = useState<MapManifest | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!mapId) {
      setManifest(null);
      return;
    }

    const fetchManifest = async () => {
      setLoading(true);
      setError(null);

      try {
        const response = await fetch(`${MAP_API_URL}/maps/${mapId}`);
        if (!response.ok) {
          throw new Error(`Failed to fetch map: ${response.statusText}`);
        }

        const data = await response.json();
        setManifest(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Unknown error");
      } finally {
        setLoading(false);
      }
    };

    fetchManifest();
  }, [mapId]);

  return { manifest, loading, error };
}

/**
 * Get the URL for a map asset.
 */
export function getMapAssetUrl(mapId: string, asset: string): string {
  return `${MAP_API_URL}/maps/${mapId}/${asset}`;
}
