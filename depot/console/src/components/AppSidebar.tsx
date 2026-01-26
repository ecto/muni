import { useEffect, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  TooltipProvider,
} from "@/components/ui/tooltip";
import {
  CellTower,
  Desktop,
  Robot,
  MapTrifold,
  VideoCamera,
  SquaresFour,
  ArrowSquareOut,
  ChartBar,
  Database,
  NavigationArrow,
  Sun,
  Moon,
  WifiHigh,
  HardDrive,
  Broadcast,
  Cube,
  Plugs,
  BatteryFull,
  BatteryLow,
  BatteryWarning,
  Warning,
} from "@phosphor-icons/react";
import { Badge } from "@/components/ui/badge";
import { useConsoleStore } from "@/store";
import { useServiceHealth, type ServiceStatus } from "@/hooks/useServiceHealth";
import { useTheme } from "@/hooks/useTheme";
import { Mode, ModeLabels, AlertSeverity, type RoverInfo } from "@/lib/types";
import { getBatteryPercent } from "@/lib/utils";

// Map service IDs to icons
const serviceIcons: Record<string, React.ReactNode> = {
  discovery: <WifiHigh className="h-3.5 w-3.5" />,
  dispatch: <NavigationArrow className="h-3.5 w-3.5" />,
  "gps-status": <CellTower className="h-3.5 w-3.5" />,
  "map-api": <MapTrifold className="h-3.5 w-3.5" />,
  mapper: <Cube className="h-3.5 w-3.5" />,
  grafana: <ChartBar className="h-3.5 w-3.5" />,
  influxdb: <Database className="h-3.5 w-3.5" />,
  sftp: <HardDrive className="h-3.5 w-3.5" />,
  ntrip: <Broadcast className="h-3.5 w-3.5" />,
  postgres: <Plugs className="h-3.5 w-3.5" />,
};

// External links for services
const serviceLinks: Record<string, string> = {
  grafana: "/grafana/",
  influxdb: `${window.location.protocol}//${window.location.hostname}:8086/`,
};

// Tooltip descriptions for services
const serviceTooltips: Record<string, string> = {
  discovery: "Rover registration and heartbeat service",
  dispatch: "Mission planning and task assignment",
  "gps-status": "RTK GPS base station status",
  "map-api": "Map tile and data serving API",
  mapper: "Map processing and reconstruction",
  grafana: "Metrics dashboards and alerts",
  influxdb: "Time-series metrics database",
  sftp: "Session recording file storage",
  ntrip: "RTK correction broadcast service",
  postgres: "Persistent data storage",
};

export function AppSidebar() {
  const location = useLocation();
  const rovers = useConsoleStore((s) => s.rovers);
  const gpsStatus = useConsoleStore((s) => s.gpsStatus);
  const alerts = useConsoleStore((s) => s.alerts);
  const selectedRoverId = useConsoleStore((s) => s.selectedRoverId);
  const connected = useConsoleStore((s) => s.connected);
  const { theme, cycleTheme } = useTheme();

  // Poll latency at 2Hz instead of subscribing (avoids 10Hz re-renders)
  const [latencyMs, setLatencyMs] = useState(0);
  useEffect(() => {
    const id = setInterval(() => {
      setLatencyMs(useConsoleStore.getState().latencyMs);
    }, 500);
    return () => clearInterval(id);
  }, []);
  const { services, healthyCount, totalCount } = useServiceHealth();

  const isActive = (path: string) => location.pathname === path;
  const isActivePrefix = (prefix: string) =>
    location.pathname.startsWith(prefix);

  const onlineRovers = rovers.filter((r) => r.online);
  const gpsOk = gpsStatus?.connected && gpsStatus.fixQuality !== "no_fix";

  // Alert badge count (active, uncleared, unacknowledged critical/warning)
  const alertBadgeCount = alerts.filter(
    (a) =>
      !a.clearedAt &&
      !a.acknowledged &&
      (a.severity === AlertSeverity.Critical ||
        a.severity === AlertSeverity.Warning)
  ).length;

  // Theme toggle icon and label
  const themeIcon =
    theme === "light" ? (
      <Sun className="h-4 w-4" />
    ) : theme === "dark" ? (
      <Moon className="h-4 w-4" />
    ) : (
      <Desktop className="h-4 w-4" />
    );
  const themeLabel =
    theme === "light" ? "Light" : theme === "dark" ? "Dark" : "System";

  return (
    <Sidebar>
      <SidebarHeader className="border-b border-sidebar-border">
        <Link to="/" className="flex items-center gap-2 px-2 py-1">
          <div className="flex h-8 w-8 items-center justify-center bg-primary text-primary-foreground font-bold text-sm">
            M
          </div>
          <div className="flex flex-col">
            <span className="font-semibold text-sm tracking-wide">
              MUNI CONSOLE
            </span>
            <span className="text-xs text-sidebar-foreground/60">
              Fleet Operations
            </span>
          </div>
        </Link>
      </SidebarHeader>

      <SidebarContent>
        {/* Operations */}
        <SidebarGroup>
          <SidebarGroupLabel>Operations</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActive("/")}
                  tooltip="Dashboard"
                >
                  <Link to="/">
                    <SquaresFour />
                    <span>Dashboard</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActive("/dispatch")}
                  tooltip="Dispatch"
                >
                  <Link to="/dispatch">
                    <NavigationArrow className="h-4 w-4" />
                    <span>Dispatch</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActive("/alerts")}
                  tooltip="Alerts"
                >
                  <Link to="/alerts" className="flex items-center justify-between w-full">
                    <span className="flex items-center gap-2">
                      <Warning className="h-4 w-4" />
                      <span>Alerts</span>
                    </span>
                    {alertBadgeCount > 0 && (
                      <Badge
                        variant="destructive"
                        className="text-[10px] h-4 px-1.5 ml-auto"
                      >
                        {alertBadgeCount}
                      </Badge>
                    )}
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Fleet */}
        <SidebarGroup>
          <SidebarGroupLabel>
            Fleet ({onlineRovers.length}/{rovers.length})
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActive("/fleet")}
                  tooltip="All Rovers"
                >
                  <Link to="/fleet">
                    <Robot className="h-4 w-4" />
                    <span>All Rovers</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
            {/* Individual rovers with status */}
            {rovers.length > 0 && (
              <div className="mt-2 space-y-1 px-2">
                {rovers.map((rover) => {
                  const isThisConnected = connected && selectedRoverId === rover.id;
                  return (
                    <RoverCard
                      key={rover.id}
                      rover={rover}
                      isActive={isActivePrefix(`/fleet/${rover.id}`)}
                      isConnected={isThisConnected}
                      latencyMs={isThisConnected ? latencyMs : undefined}
                    />
                  );
                })}
              </div>
            )}
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Infrastructure */}
        <SidebarGroup>
          <SidebarGroupLabel>Infrastructure</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActive("/base-station")}
                  tooltip="Base Station"
                >
                  <Link to="/base-station">
                    <CellTower className="h-4 w-4" />
                    <span>Base Station</span>
                    <StatusDot status={gpsOk ? "ok" : gpsStatus?.connected ? "warning" : "offline"} />
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActive("/sessions")}
                  tooltip="Sessions"
                >
                  <Link to="/sessions">
                    <VideoCamera className="h-4 w-4" />
                    <span>Sessions</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActivePrefix("/maps")}
                  tooltip="Maps"
                >
                  <Link to="/maps">
                    <MapTrifold className="h-4 w-4" />
                    <span>Maps</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Services HUD */}
        <SidebarGroup>
          <SidebarGroupLabel>
            Services ({healthyCount}/{totalCount})
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <div className="px-2 py-1">
              <ServiceGrid services={services} />
            </div>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="border-t border-sidebar-border">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton onClick={cycleTheme} tooltip={`Theme: ${themeLabel}`}>
              {themeIcon}
              <span>{themeLabel}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}

function ServiceGrid({ services }: { services: ServiceStatus[] }) {
  return (
    <TooltipProvider delayDuration={0}>
      <div className="grid grid-cols-2 gap-1">
        {services.map((service) => {
          const link = serviceLinks[service.id];
          const icon = serviceIcons[service.id] || <Desktop className="h-3.5 w-3.5" />;
          const tooltip = serviceTooltips[service.id] || service.name;

          const content = (
            <div
              className={`flex items-center gap-1.5 px-2 py-1.5 rounded text-xs transition-colors ${
                service.status === "healthy"
                  ? "bg-green-500/10 text-green-600 dark:text-green-400"
                  : service.status === "unhealthy"
                  ? "bg-red-500/10 text-red-600 dark:text-red-400"
                  : "bg-muted text-muted-foreground"
              } ${link ? "hover:bg-opacity-20 cursor-pointer" : ""}`}
            >
              <StatusDot status={serviceStatusToDot(service.status)} />
              {icon}
              <span className="truncate flex-1">{service.name}</span>
              {link && <ArrowSquareOut className="h-2.5 w-2.5 opacity-50" />}
            </div>
          );

          if (link) {
            return (
              <Tooltip key={service.id}>
                <TooltipTrigger asChild>
                  <a
                    href={link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {content}
                  </a>
                </TooltipTrigger>
                <TooltipContent side="right">{tooltip}</TooltipContent>
              </Tooltip>
            );
          }

          return (
            <Tooltip key={service.id}>
              <TooltipTrigger asChild>
                <div>{content}</div>
              </TooltipTrigger>
              <TooltipContent side="right">{tooltip}</TooltipContent>
            </Tooltip>
          );
        })}
      </div>
    </TooltipProvider>
  );
}

function serviceStatusToDot(status: "healthy" | "unhealthy" | "checking" | "unknown"): "ok" | "offline" | "unknown" {
  if (status === "healthy") return "ok";
  if (status === "unhealthy") return "offline";
  return "unknown"; // covers "checking" and "unknown"
}

function StatusDot({ status, pulse }: { status: "ok" | "offline" | "warning" | "unknown"; pulse?: boolean }) {
  const colors = {
    ok: "bg-green-500",
    offline: "bg-red-500",
    warning: "bg-yellow-500",
    unknown: "bg-muted-foreground",
  };

  return (
    <span
      className={`h-1.5 w-1.5 rounded-full ${colors[status]} flex-shrink-0${pulse ? " animate-pulse" : ""}`}
      aria-label={status}
    />
  );
}

function timeAgo(ms: number): string {
  const secs = Math.floor((Date.now() - ms) / 1000);
  if (secs < 10) return "just now";
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

function RoverCard({ rover, isActive, isConnected, latencyMs }: {
  rover: RoverInfo;
  isActive: boolean;
  isConnected?: boolean;
  latencyMs?: number;
}) {
  const isOnline = rover.online || isConnected;
  // Disabled and Idle are both "motors off, not driving" — merge for display
  const displayMode = rover.mode === Mode.Disabled ? Mode.Idle : rover.mode;
  const batteryPercent = getBatteryPercent(rover.batteryVoltage);

  const BatteryIcon = rover.batteryVoltage < 42
    ? BatteryWarning
    : rover.batteryVoltage < 45
    ? BatteryLow
    : BatteryFull;

  const batteryColor = rover.batteryVoltage < 42
    ? "text-red-500"
    : rover.batteryVoltage < 45
    ? "text-orange-500"
    : "text-green-500";

  const getModeVariant = (mode: Mode): "default" | "secondary" | "destructive" | "outline" => {
    switch (mode) {
      case Mode.Teleop:
      case Mode.Autonomous:
        return "default";
      case Mode.EStop:
      case Mode.Fault:
        return "destructive";
      case Mode.Idle:
        return "secondary";
      default:
        return "outline";
    }
  };

  return (
    <Link
      to={`/fleet/${rover.id}`}
      className={`block rounded-md p-2 transition-colors ${
        isActive
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "hover:bg-sidebar-accent/50"
      } ${!isOnline ? "opacity-50" : ""}`}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <StatusDot status={isOnline ? "ok" : "offline"} pulse={isConnected} />
          <span className="text-sm font-medium truncate">{rover.name || rover.id}</span>
        </div>
        {isOnline && (
          <Badge variant={getModeVariant(displayMode)} className="text-[10px] h-4 px-1.5 shrink-0">
            {ModeLabels[displayMode]}
          </Badge>
        )}
      </div>
      {isOnline && (
        <div className="flex items-center gap-1 mt-1 ml-3.5 text-xs text-muted-foreground">
          <BatteryIcon className={`h-3 w-3 ${batteryColor}`} weight="fill" />
          <span>{batteryPercent.toFixed(0)}%</span>
          <span className="text-muted-foreground/60">({rover.batteryVoltage.toFixed(1)}V)</span>
          {isConnected && latencyMs != null && (
            <>
              <span className="text-muted-foreground/30 mx-0.5">&middot;</span>
              <span className="text-muted-foreground/60">{latencyMs}ms</span>
            </>
          )}
        </div>
      )}
      {!isOnline && rover.lastSeen > 0 && (
        <div className="mt-1 ml-3.5 text-xs text-muted-foreground/50">
          seen {timeAgo(rover.lastSeen)}
        </div>
      )}
    </Link>
  );
}
