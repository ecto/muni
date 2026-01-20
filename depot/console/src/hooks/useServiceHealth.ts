/* eslint-disable react-hooks/set-state-in-effect */
import { useEffect, useState, useCallback } from "react";

export interface ServiceStatus {
  name: string;
  status: "healthy" | "unhealthy" | "checking";
  url?: string;
}

const SERVICE_CONFIGS: { name: string; url?: string }[] = [
  { name: "Discovery", url: "/api/discovery/health" },
  { name: "Grafana", url: "/grafana/api/health" },
  { name: "Map API", url: "/api/maps/health" },
  { name: "Dispatch", url: "/api/dispatch/health" },
];

export function useServiceHealth() {
  const [services, setServices] = useState<ServiceStatus[]>(
    SERVICE_CONFIGS.map((s) => ({ ...s, status: "checking" as const }))
  );

  const checkHealth = useCallback(async () => {
    const updated = await Promise.all(
      SERVICE_CONFIGS.map(async (config) => {
        if (!config.url) {
          return { ...config, status: "healthy" as const };
        }

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
    (name: string): "healthy" | "unhealthy" | "checking" => {
      return services.find((s) => s.name === name)?.status ?? "checking";
    },
    [services]
  );

  const healthyCount = services.filter((s) => s.status === "healthy").length;

  return { services, getStatus, healthyCount, totalCount: services.length };
}
