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
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
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
} from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { useDiscovery } from "@/hooks/useDiscovery";
import { useServiceHealth, type ServiceStatus } from "@/hooks/useServiceHealth";
import { useTheme } from "@/hooks/useTheme";

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

export function AppSidebar() {
  const location = useLocation();
  const { rovers, gpsStatus } = useConsoleStore();
  const { theme, cycleTheme } = useTheme();
  const { services, healthyCount, totalCount } = useServiceHealth();

  // Connect to discovery service for live rover updates
  useDiscovery();

  const isActive = (path: string) => location.pathname === path;
  const isActivePrefix = (prefix: string) =>
    location.pathname.startsWith(prefix);

  const onlineRovers = rovers.filter((r) => r.online);
  const gpsOk = gpsStatus?.connected && gpsStatus.fixQuality !== "no_fix";

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
        {/* Overview */}
        <SidebarGroup>
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
          </SidebarMenu>
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
                {rovers.length > 0 && (
                  <SidebarMenuSub>
                    {rovers.map((rover) => (
                      <SidebarMenuSubItem key={rover.id}>
                        <SidebarMenuSubButton
                          asChild
                          isActive={isActivePrefix(`/fleet/${rover.id}`)}
                        >
                          <Link to={`/fleet/${rover.id}`}>
                            <StatusDot
                              status={rover.online ? "ok" : "offline"}
                            />
                            <span>{rover.name || rover.id}</span>
                          </Link>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                    ))}
                  </SidebarMenuSub>
                )}
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
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Data */}
        <SidebarGroup>
          <SidebarGroupLabel>Data</SidebarGroupLabel>
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
    <div className="grid grid-cols-2 gap-1">
      {services.map((service) => {
        const link = serviceLinks[service.id];
        const icon = serviceIcons[service.id] || <Desktop className="h-3.5 w-3.5" />;

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
            <a
              key={service.id}
              href={link}
              target="_blank"
              rel="noopener noreferrer"
            >
              {content}
            </a>
          );
        }

        return <div key={service.id}>{content}</div>;
      })}
    </div>
  );
}

function serviceStatusToDot(status: "healthy" | "unhealthy" | "checking"): "ok" | "offline" | "unknown" {
  if (status === "healthy") return "ok";
  if (status === "unhealthy") return "offline";
  return "unknown";
}

function StatusDot({ status }: { status: "ok" | "offline" | "warning" | "unknown" }) {
  const colors = {
    ok: "bg-green-500",
    offline: "bg-red-500",
    warning: "bg-yellow-500",
    unknown: "bg-muted-foreground",
  };

  return (
    <span
      className={`h-1.5 w-1.5 rounded-full ${colors[status]} flex-shrink-0`}
      aria-label={status}
    />
  );
}
