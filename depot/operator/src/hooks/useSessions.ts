import { useEffect, useCallback, useRef } from "react";
import { useOperatorStore } from "@/store";
import type { Session } from "@/lib/types";

/**
 * Hook to fetch and manage historical sessions.
 *
 * Sessions are fetched from the discovery service which has access
 * to synced session data from all rovers.
 */
export function useSessions() {
  const {
    sessions,
    sessionsLoading,
    sessionsError,
    sessionRoverFilter,
    setSessions,
    setSessionsLoading,
    setSessionsError,
  } = useOperatorStore();

  const fetchedRef = useRef(false);

  // Build the sessions API URL
  const getSessionsUrl = useCallback(() => {
    // In development, use localhost
    if (window.location.hostname === "localhost") {
      return "http://localhost:4860/api/sessions";
    }
    // In production, use same origin with discovery port
    const protocol = window.location.protocol;
    return `${protocol}//${window.location.hostname}:4860/api/sessions`;
  }, []);

  const fetchSessions = useCallback(async () => {
    setSessionsLoading(true);
    setSessionsError(null);

    try {
      const url = getSessionsUrl();
      const params = new URLSearchParams();
      if (sessionRoverFilter) {
        params.set("rover_id", sessionRoverFilter);
      }
      const fullUrl = params.toString() ? `${url}?${params}` : url;

      console.log("[sessions] Fetching from", fullUrl);

      const response = await fetch(fullUrl);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      const sessions: Session[] = data.sessions || [];

      // Sort by start time, newest first
      sessions.sort(
        (a, b) =>
          new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
      );

      console.log("[sessions] Loaded", sessions.length, "sessions");
      setSessions(sessions);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to fetch sessions";
      console.error("[sessions] Error:", message);
      setSessionsError(message);
    } finally {
      setSessionsLoading(false);
    }
  }, [
    getSessionsUrl,
    sessionRoverFilter,
    setSessions,
    setSessionsLoading,
    setSessionsError,
  ]);

  // Fetch sessions on mount
  useEffect(() => {
    if (!fetchedRef.current) {
      fetchedRef.current = true;
      fetchSessions();
    }
  }, [fetchSessions]);

  // Refetch when rover filter changes
  useEffect(() => {
    if (fetchedRef.current) {
      fetchSessions();
    }
  }, [sessionRoverFilter, fetchSessions]);

  return {
    sessions,
    loading: sessionsLoading,
    error: sessionsError,
    refresh: fetchSessions,
  };
}

/**
 * Get the URL to stream/download a session's RRD file.
 */
export function getSessionRrdUrl(session: Session): string {
  const base =
    window.location.hostname === "localhost"
      ? "http://localhost:4860"
      : `${window.location.protocol}//${window.location.hostname}:4860`;

  return `${base}/api/sessions/${session.rover_id}/${session.session_dir}/session.rrd`;
}

/**
 * Get the Rerun web viewer URL for a session.
 *
 * Uses Rerun's hosted web viewer with our session URL.
 */
export function getRerunViewerUrl(session: Session): string {
  const rrdUrl = getSessionRrdUrl(session);
  // Rerun web viewer can load remote .rrd files
  return `https://app.rerun.io/version/0.22.0/?url=${encodeURIComponent(rrdUrl)}`;
}
