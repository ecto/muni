import { Routes, Route } from "react-router-dom";
import { AppSidebar } from "@/components/AppSidebar";
import { AlertBanner } from "@/components/AlertBanner";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { Toast } from "@/components/ui/Toast";
import { useDiscovery } from "@/hooks/useDiscovery";
import { useGpsStatus } from "@/hooks/useGpsStatus";
import { useAlerts } from "@/hooks/useAlerts";
import { useTheme } from "@/hooks/useTheme";
import { ErrorBoundary } from "@/components/ErrorBoundary";

// Views
import { DashboardView } from "@/views/DashboardView";
import { BaseStationView } from "@/views/BaseStationView";
import { FleetView } from "@/views/FleetView";
import { RoverView } from "@/views/RoverView";
import { SessionsView } from "@/views/SessionsView";
import { SessionDetailView } from "@/views/SessionDetailView";
import { MapsView } from "@/views/MapsView";
import { DispatchView } from "@/views/DispatchView";
import { AlertsView } from "@/views/AlertsView";

function App() {
  // Connect to backend services
  useDiscovery();
  useGpsStatus();
  useAlerts();

  const { resolvedTheme } = useTheme();
  const themeClass = resolvedTheme === "dark" ? "dark" : "";

  return (
    <div className={`${themeClass} min-h-screen`}>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
          <AlertBanner />
          <ErrorBoundary>
            <Routes>
              <Route path="/" element={<DashboardView />} />
              <Route path="/base-station" element={<BaseStationView />} />
              <Route path="/fleet" element={<FleetView />} />
              <Route path="/fleet/:roverId" element={<RoverView />} />
              <Route path="/sessions" element={<SessionsView />} />
              <Route path="/sessions/:sessionName" element={<SessionDetailView />} />
              <Route path="/maps" element={<MapsView />} />
              <Route path="/maps/:mapId" element={<MapsView />} />
              <Route path="/dispatch" element={<DispatchView />} />
              <Route path="/alerts" element={<AlertsView />} />
            </Routes>
          </ErrorBoundary>
        </SidebarInset>
      </SidebarProvider>
      <Toast />
    </div>
  );
}

export default App;
