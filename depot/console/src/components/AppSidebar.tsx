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
  Satellite,
  Server,
  Bot,
  Map,
  Video,
  LayoutDashboard,
  ExternalLink,
  BarChart3,
  Database,
} from "lucide-react";
import { useConsoleStore } from "@/store";
import { useDiscovery } from "@/hooks/useDiscovery";

export function AppSidebar() {
  const location = useLocation();
  const { rovers, gpsStatus } = useConsoleStore();

  // Connect to discovery service for live rover updates
  useDiscovery();

  const isActive = (path: string) => location.pathname === path;
  const isActivePrefix = (prefix: string) =>
    location.pathname.startsWith(prefix);

  const onlineRovers = rovers.filter((r) => r.online);
  const gpsOk = gpsStatus?.connected && gpsStatus.fixQuality !== "no_fix";

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
                  <LayoutDashboard />
                  <span>Dashboard</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
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
                    <Satellite />
                    <span>Base Station</span>
                    <StatusDot status={gpsOk ? "ok" : "unknown"} />
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  asChild
                  isActive={isActive("/services")}
                  tooltip="Services"
                >
                  <Link to="/services">
                    <Server />
                    <span>Services</span>
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
                    <Bot />
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
                  isActive={isActive("/sessions")}
                  tooltip="Sessions"
                >
                  <Link to="/sessions">
                    <Video />
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
                    <Map />
                    <span>Maps</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="border-t border-sidebar-border">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild tooltip="Grafana Dashboards">
              <a href="/grafana/" target="_blank" rel="noopener noreferrer">
                <BarChart3 />
                <span>Dashboards</span>
                <ExternalLink className="ml-auto h-3 w-3 opacity-50" />
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton asChild tooltip="InfluxDB">
              <a
                href={`//${window.location.hostname}:8086/`}
                target="_blank"
                rel="noopener noreferrer"
              >
                <Database />
                <span>Database</span>
                <ExternalLink className="ml-auto h-3 w-3 opacity-50" />
              </a>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}

function StatusDot({ status }: { status: "ok" | "offline" | "unknown" }) {
  const colors = {
    ok: "bg-green-500",
    offline: "bg-red-500",
    unknown: "bg-muted-foreground",
  };

  return (
    <span
      className={`ml-auto h-2 w-2 rounded-full ${colors[status]}`}
      aria-label={status}
    />
  );
}
