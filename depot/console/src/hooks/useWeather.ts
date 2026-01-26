import { useEffect, useState, useCallback } from "react";
import type { WeatherResponse } from "@/lib/types";

/**
 * Fetches weather data from the dispatch server's NOAA integration.
 * Polls every 5 minutes (NOAA updates every 30 minutes).
 */
export function useWeather() {
  const [weather, setWeather] = useState<WeatherResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getDispatchUrl = useCallback((path: string) => {
    if (window.location.hostname === "localhost") {
      return `http://depot:4890${path}`;
    }
    const protocol = window.location.protocol;
    return `${protocol}//${window.location.hostname}:4890${path}`;
  }, []);

  const fetchWeather = useCallback(async () => {
    setLoading(true);
    try {
      const url = getDispatchUrl("/api/weather");
      const response = await fetch(url);
      if (response.ok) {
        const data: WeatherResponse = await response.json();
        setWeather(data);
        setError(null);
      } else {
        setError("Failed to fetch weather");
      }
    } catch {
      setError("Weather service unavailable");
    } finally {
      setLoading(false);
    }
  }, [getDispatchUrl]);

  useEffect(() => {
    fetchWeather();
    const interval = setInterval(fetchWeather, 5 * 60 * 1000);
    return () => clearInterval(interval);
  }, [fetchWeather]);

  return { weather, loading, error, fetchWeather };
}
