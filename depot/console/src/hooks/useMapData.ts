import { useState, useEffect, useCallback } from "react";
import type { Zone } from "@/lib/types";

/**
 * Hook to fetch map data (zones) from the dispatch service.
 * Simplified version of useDispatch for read-only scene rendering.
 */
export function useMapData() {
  const [zones, setZones] = useState<Zone[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getDispatchUrl = useCallback((path: string) => {
    if (window.location.hostname === "localhost") {
      // Dev mode: connect to depot directly
      const depotHost = import.meta.env.VITE_DEPOT_HOST || "depot";
      return `http://${depotHost}:4890${path}`;
    }
    // Production: route through nginx reverse proxy
    const protocol = window.location.protocol;
    return `${protocol}//${window.location.host}/api/dispatch${path}`;
  }, []);

  const fetchZones = useCallback(async () => {
    setLoading(true);
    try {
      const url = getDispatchUrl("/zones");
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to fetch zones: ${response.statusText}`);
      }
      const data = await response.json();
      setZones(data);
      setError(null);
    } catch (e) {
      console.warn("[useMapData] Failed to fetch zones:", e);
      setError(e instanceof Error ? e.message : "Failed to fetch zones");
    } finally {
      setLoading(false);
    }
  }, [getDispatchUrl]);

  // Fetch zones on mount
  useEffect(() => {
    fetchZones();
  }, [fetchZones]);

  return {
    zones,
    loading,
    error,
    refetch: fetchZones,
  };
}
