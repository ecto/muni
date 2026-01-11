import { useEffect, useRef, useCallback, useState } from "react";
import type {
  Zone,
  Mission,
  Task,
  Waypoint,
  GpsCoord,
  Schedule,
  ConnectedRover,
} from "@/lib/types";

const RECONNECT_DELAY_MS = 3000;

// API response types
interface CreateZoneRequest {
  name: string;
  zoneType?: string;
  waypoints: Waypoint[];
  polygon?: GpsCoord[];
  mapId?: string;
}

interface UpdateZoneRequest {
  name?: string;
  zoneType?: string;
  waypoints?: Waypoint[];
  polygon?: GpsCoord[];
  mapId?: string;
}

interface CreateMissionRequest {
  name: string;
  zoneId: string;
  roverId?: string;
  schedule?: Schedule;
}

interface UpdateMissionRequest {
  name?: string;
  zoneId?: string;
  roverId?: string;
  schedule?: Schedule;
  enabled?: boolean;
}

// WebSocket message types
interface TaskUpdateMessage {
  type: "task_update";
  task: Task;
}

interface RoverUpdateMessage {
  type: "rover_update";
  rover_id: string;
  connected: boolean;
  task_id?: string;
}

interface ZoneUpdateMessage {
  type: "zone_update";
  zone: Zone;
}

interface MissionUpdateMessage {
  type: "mission_update";
  mission: Mission;
}

type DispatchMessage =
  | TaskUpdateMessage
  | RoverUpdateMessage
  | ZoneUpdateMessage
  | MissionUpdateMessage;

/**
 * Hook to interact with the dispatch service for mission planning.
 */
export function useDispatch() {
  const [zones, setZones] = useState<Zone[]>([]);
  const [missions, setMissions] = useState<Mission[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [connectedRovers, setConnectedRovers] = useState<ConnectedRover[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );
  const connectRef = useRef<() => void>(() => {});

  const getDispatchUrl = useCallback((path: string) => {
    if (window.location.hostname === "localhost") {
      return `http://localhost:4890${path}`;
    }
    const protocol = window.location.protocol;
    return `${protocol}//${window.location.hostname}:4890${path}`;
  }, []);

  const getDispatchWsUrl = useCallback(() => {
    if (window.location.hostname === "localhost") {
      return "ws://localhost:4890/ws/console";
    }
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${window.location.hostname}:4890/ws/console`;
  }, []);

  // ==========================================================================
  // WebSocket for real-time updates
  // ==========================================================================

  const connect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    const url = getDispatchWsUrl();
    console.debug("[dispatch] Connecting to", url);

    try {
      const ws = new WebSocket(url);

      ws.onopen = () => {
        console.debug("[dispatch] WebSocket connected");
      };

      ws.onmessage = (event) => {
        try {
          const message: DispatchMessage = JSON.parse(event.data);

          switch (message.type) {
            case "task_update":
              setTasks((prev) => {
                const idx = prev.findIndex((t) => t.id === message.task.id);
                if (idx >= 0) {
                  const updated = [...prev];
                  updated[idx] = message.task;
                  return updated;
                }
                return [message.task, ...prev];
              });
              break;

            case "rover_update":
              setConnectedRovers((prev) => {
                const idx = prev.findIndex(
                  (r) => r.roverId === message.rover_id
                );
                const rover: ConnectedRover = {
                  roverId: message.rover_id,
                  connected: message.connected,
                  taskId: message.task_id,
                };
                if (idx >= 0) {
                  const updated = [...prev];
                  if (message.connected) {
                    updated[idx] = rover;
                  } else {
                    updated.splice(idx, 1);
                  }
                  return updated;
                }
                if (message.connected) {
                  return [...prev, rover];
                }
                return prev;
              });
              break;

            case "zone_update":
              setZones((prev) => {
                const idx = prev.findIndex((z) => z.id === message.zone.id);
                if (idx >= 0) {
                  const updated = [...prev];
                  updated[idx] = message.zone;
                  return updated;
                }
                return [message.zone, ...prev];
              });
              break;

            case "mission_update":
              setMissions((prev) => {
                const idx = prev.findIndex((m) => m.id === message.mission.id);
                if (idx >= 0) {
                  const updated = [...prev];
                  updated[idx] = message.mission;
                  return updated;
                }
                return [message.mission, ...prev];
              });
              break;
          }
        } catch (e) {
          console.error("[dispatch] Failed to parse message:", e);
        }
      };

      ws.onclose = () => {
        console.debug("[dispatch] WebSocket disconnected, reconnecting...");
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current();
        }, RECONNECT_DELAY_MS);
      };

      ws.onerror = (e) => {
        console.error("[dispatch] WebSocket error:", e);
      };

      wsRef.current = ws;
    } catch (e) {
      console.error("[dispatch] Connection error:", e);
      reconnectTimeoutRef.current = setTimeout(() => {
        connectRef.current();
      }, RECONNECT_DELAY_MS);
    }
  }, [getDispatchWsUrl]);

  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  // ==========================================================================
  // REST API helpers
  // ==========================================================================

  const fetchApi = useCallback(
    async <T>(
      path: string,
      options?: RequestInit
    ): Promise<T> => {
      const url = getDispatchUrl(path);
      const response = await fetch(url, {
        ...options,
        headers: {
          "Content-Type": "application/json",
          ...options?.headers,
        },
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || response.statusText);
      }

      // Handle 204 No Content
      if (response.status === 204) {
        return undefined as T;
      }

      return response.json();
    },
    [getDispatchUrl]
  );

  // ==========================================================================
  // Zone CRUD
  // ==========================================================================

  const fetchZones = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchApi<Zone[]>("/zones");
      setZones(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to fetch zones");
    } finally {
      setLoading(false);
    }
  }, [fetchApi]);

  const createZone = useCallback(
    async (zone: CreateZoneRequest): Promise<Zone> => {
      const data = await fetchApi<Zone>("/zones", {
        method: "POST",
        body: JSON.stringify(zone),
      });
      return data;
    },
    [fetchApi]
  );

  const updateZone = useCallback(
    async (id: string, updates: UpdateZoneRequest): Promise<Zone> => {
      const data = await fetchApi<Zone>(`/zones/${id}`, {
        method: "PUT",
        body: JSON.stringify(updates),
      });
      return data;
    },
    [fetchApi]
  );

  const deleteZone = useCallback(
    async (id: string): Promise<void> => {
      await fetchApi<void>(`/zones/${id}`, { method: "DELETE" });
      setZones((prev) => prev.filter((z) => z.id !== id));
    },
    [fetchApi]
  );

  // ==========================================================================
  // Mission CRUD
  // ==========================================================================

  const fetchMissions = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchApi<Mission[]>("/missions");
      setMissions(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to fetch missions");
    } finally {
      setLoading(false);
    }
  }, [fetchApi]);

  const createMission = useCallback(
    async (mission: CreateMissionRequest): Promise<Mission> => {
      const data = await fetchApi<Mission>("/missions", {
        method: "POST",
        body: JSON.stringify(mission),
      });
      return data;
    },
    [fetchApi]
  );

  const updateMission = useCallback(
    async (id: string, updates: UpdateMissionRequest): Promise<Mission> => {
      const data = await fetchApi<Mission>(`/missions/${id}`, {
        method: "PUT",
        body: JSON.stringify(updates),
      });
      return data;
    },
    [fetchApi]
  );

  const deleteMission = useCallback(
    async (id: string): Promise<void> => {
      await fetchApi<void>(`/missions/${id}`, { method: "DELETE" });
      setMissions((prev) => prev.filter((m) => m.id !== id));
    },
    [fetchApi]
  );

  const startMission = useCallback(
    async (id: string): Promise<Task> => {
      const data = await fetchApi<Task>(`/missions/${id}/start`, {
        method: "POST",
      });
      return data;
    },
    [fetchApi]
  );

  const stopMission = useCallback(
    async (id: string): Promise<Task> => {
      const data = await fetchApi<Task>(`/missions/${id}/stop`, {
        method: "POST",
      });
      return data;
    },
    [fetchApi]
  );

  // ==========================================================================
  // Task operations
  // ==========================================================================

  const fetchTasks = useCallback(
    async (options?: {
      status?: string;
      roverId?: string;
      missionId?: string;
    }) => {
      setLoading(true);
      try {
        const params = new URLSearchParams();
        if (options?.status) params.set("status", options.status);
        if (options?.roverId) params.set("rover_id", options.roverId);
        if (options?.missionId) params.set("mission_id", options.missionId);

        const query = params.toString();
        const path = query ? `/tasks?${query}` : "/tasks";
        const data = await fetchApi<Task[]>(path);
        setTasks(data);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to fetch tasks");
      } finally {
        setLoading(false);
      }
    },
    [fetchApi]
  );

  const cancelTask = useCallback(
    async (id: string): Promise<Task> => {
      const data = await fetchApi<Task>(`/tasks/${id}/cancel`, {
        method: "POST",
      });
      return data;
    },
    [fetchApi]
  );

  // ==========================================================================
  // Lifecycle
  // ==========================================================================

  useEffect(() => {
    // Connect WebSocket
    connect();

    // Fetch initial data
    fetchZones();
    fetchMissions();
    fetchTasks();

    return () => {
      disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    // State
    zones,
    missions,
    tasks,
    connectedRovers,
    loading,
    error,

    // Zone operations
    fetchZones,
    createZone,
    updateZone,
    deleteZone,

    // Mission operations
    fetchMissions,
    createMission,
    updateMission,
    deleteMission,
    startMission,
    stopMission,

    // Task operations
    fetchTasks,
    cancelTask,

    // Connection
    connect,
    disconnect,
  };
}
