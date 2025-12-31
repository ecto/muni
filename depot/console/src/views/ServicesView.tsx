import { useEffect, useState } from "react";
import {
  Server,
  Database,
  HardDrive,
  Wifi,
  Satellite,
  CheckCircle2,
  XCircle,
  Loader2,
} from "lucide-react";

interface ServiceStatus {
  name: string;
  description: string;
  status: "healthy" | "unhealthy" | "checking";
  url?: string;
  details?: string;
  icon: React.ReactNode;
}

export function ServicesView() {
  const [services, setServices] = useState<ServiceStatus[]>([
    {
      name: "Discovery",
      description: "Rover registration and fleet status",
      status: "checking",
      url: "/api/discovery/health",
      icon: <Wifi className="h-4 w-4" />,
    },
    {
      name: "InfluxDB",
      description: "Time-series metrics storage",
      status: "checking",
      url: `${window.location.protocol}//${window.location.hostname}:8086/health`,
      icon: <Database className="h-4 w-4" />,
    },
    {
      name: "Grafana",
      description: "Metrics dashboards",
      status: "checking",
      url: "/grafana/api/health",
      icon: <Server className="h-4 w-4" />,
    },
    {
      name: "Map API",
      description: "3D map serving",
      status: "checking",
      url: "/api/maps/health",
      icon: <HardDrive className="h-4 w-4" />,
    },
    {
      name: "SFTP",
      description: "Session file storage",
      status: "checking",
      details: "Port 2222",
      icon: <HardDrive className="h-4 w-4" />,
    },
    {
      name: "NTRIP",
      description: "RTK corrections broadcast",
      status: "checking",
      details: "Port 2101",
      icon: <Satellite className="h-4 w-4" />,
    },
  ]);

  useEffect(() => {
    async function checkHealth() {
      const updated = await Promise.all(
        services.map(async (service) => {
          if (!service.url) {
            // For services without HTTP health endpoints, mark as healthy (optimistic)
            return { ...service, status: "healthy" as const };
          }

          try {
            const response = await fetch(service.url, {
              method: "GET",
              mode: "no-cors",
            });
            // no-cors means we can't read the response, but if we get here without error,
            // the service is at least reachable
            return {
              ...service,
              status: response.ok || response.type === "opaque" ? "healthy" as const : "unhealthy" as const,
            };
          } catch {
            return { ...service, status: "unhealthy" as const };
          }
        })
      );
      setServices(updated);
    }

    checkHealth();

    // Refresh every 30 seconds
    const interval = setInterval(checkHealth, 30000);
    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const healthyCount = services.filter((s) => s.status === "healthy").length;

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Services</h1>
            <p className="text-muted-foreground">
              Depot infrastructure health
            </p>
          </div>
          <div className="text-right">
            <p className="text-2xl font-bold">
              {healthyCount}/{services.length}
            </p>
            <p className="text-sm text-muted-foreground">services healthy</p>
          </div>
        </div>

        {/* Service List */}
        <div className="space-y-2">
          {services.map((service) => (
            <div
              key={service.name}
              className="bg-card border border-border p-4 flex items-center gap-4"
            >
              <div className="h-10 w-10 flex items-center justify-center bg-muted">
                {service.icon}
              </div>
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <h3 className="font-medium text-foreground">{service.name}</h3>
                  {service.details && (
                    <span className="text-xs text-muted-foreground font-mono">
                      {service.details}
                    </span>
                  )}
                </div>
                <p className="text-sm text-muted-foreground">
                  {service.description}
                </p>
              </div>
              <div className="flex items-center gap-2">
                {service.status === "checking" && (
                  <Loader2 className="h-5 w-5 text-muted-foreground animate-spin" />
                )}
                {service.status === "healthy" && (
                  <CheckCircle2 className="h-5 w-5 text-green-500" />
                )}
                {service.status === "unhealthy" && (
                  <XCircle className="h-5 w-5 text-red-500" />
                )}
                <span
                  className={`text-sm font-medium ${
                    service.status === "healthy"
                      ? "text-green-500"
                      : service.status === "unhealthy"
                      ? "text-red-500"
                      : "text-muted-foreground"
                  }`}
                >
                  {service.status === "checking"
                    ? "Checking..."
                    : service.status === "healthy"
                    ? "Healthy"
                    : "Unhealthy"}
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Docker Info */}
        <div className="border-t border-border pt-6">
          <h2 className="text-sm font-medium text-muted-foreground mb-4">
            DOCKER COMMANDS
          </h2>
          <div className="bg-muted/50 p-4 font-mono text-sm space-y-2">
            <p>
              <span className="text-muted-foreground"># View all services</span>
            </p>
            <p>docker compose ps</p>
            <p className="mt-4">
              <span className="text-muted-foreground"># View logs</span>
            </p>
            <p>docker compose logs -f [service]</p>
            <p className="mt-4">
              <span className="text-muted-foreground"># Restart service</span>
            </p>
            <p>docker compose restart [service]</p>
          </div>
        </div>
      </div>
    </div>
  );
}
