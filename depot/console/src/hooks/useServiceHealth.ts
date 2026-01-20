/* eslint-disable react-hooks/set-state-in-effect */
import { useEffect, useState, useCallback } from "react";

export interface ServiceStatus {
  id: string;
  name: string;
  status: "healthy" | "unhealthy" | "checking";
  url?: string;
  port?: number;
  external?: boolean;
}

const SERVICE_CONFIGS: Omit<ServiceStatus, "status">[] = [
  { id: "discovery", name: "Discovery", url: "/api/discovery/health" },
  { id: "dispatch", name: "Dispatch", url: "/api/dispatch/health" },
  { id: "gps-status", name: "GPS Status", url: "/api/gps/health" },
  { id: "map-api", name: "Map API", url: "/api/maps/health" },
  { id: "mapper", name: "Mapper", url: "/api/mapper/health" },
  { id: "grafana", name: "Grafana", url: "/grafana/api/health", external: true },
  { id: "influxdb", name: "InfluxDB", port: 8086, external: true },
  { id: "sftp", name: "SFTP", port: 2222 },
  { id: "ntrip", name: "NTRIP", port: 2101 },
  { id: "postgres", name: "PostgreSQL", port: 5432 },
];

export function useServiceHealth() {
  const [services, setServices] = useState<ServiceStatus[]>(
    SERVICE_CONFIGS.map((s) => ({ ...s, status: "checking" as const }))
  );

  const checkHealth = useCallback(async () => {
    const updated = await Promise.all(
      SERVICE_CONFIGS.map(async (config) => {
        // Services with HTTP health endpoints
        if (config.url) {
          try {
            const response = await fetch(config.url, {
              method: "GET",
              mode: "no-cors",
            });
            return {
              ...config,
              status: response.ok || response.type === "opaque" ? "healthy" as const : "unhealthy" as const,
            };
          } catch {
            return { ...config, status: "unhealthy" as const };
          }
        }

        // Services without HTTP endpoints - assume healthy (optimistic)
        // In production, these could check via a backend status endpoint
        return { ...config, status: "healthy" as const };
      })
    );
    setServices(updated);
  }, []);

  useEffect(() => {
    checkHealth();
    const interval = setInterval(checkHealth, 30000);
    return () => clearInterval(interval);
  }, [checkHealth]);

  const getStatus = useCallback(
    (id: string): "healthy" | "unhealthy" | "checking" => {
      return services.find((s) => s.id === id)?.status ?? "checking";
    },
    [services]
  );

  const healthyCount = services.filter((s) => s.status === "healthy").length;

  return { services, getStatus, healthyCount, totalCount: services.length };
}
