import { Routes, Route } from "react-router-dom";
import { AppSidebar } from "@/components/AppSidebar";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { Toast } from "@/components/ui/Toast";
import { useDiscovery } from "@/hooks/useDiscovery";
import { useGpsStatus } from "@/hooks/useGpsStatus";

// Views
import { DashboardView } from "@/views/DashboardView";
import { BaseStationView } from "@/views/BaseStationView";
import { ServicesView } from "@/views/ServicesView";
import { FleetView } from "@/views/FleetView";
import { RoverView } from "@/views/RoverView";
import { TeleopView } from "@/views/TeleopView";
import { SessionsView } from "@/views/SessionsView";
import { MapsView } from "@/views/MapsView";
import { DispatchView } from "@/views/DispatchView";

function App() {
  // Connect to backend services
  useDiscovery();
  useGpsStatus();

  return (
    <div className="dark">
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
          <Routes>
            <Route path="/" element={<DashboardView />} />
            <Route path="/base-station" element={<BaseStationView />} />
            <Route path="/services" element={<ServicesView />} />
            <Route path="/fleet" element={<FleetView />} />
            <Route path="/fleet/:roverId" element={<RoverView />} />
            <Route path="/fleet/:roverId/teleop" element={<TeleopView />} />
            <Route path="/sessions" element={<SessionsView />} />
            <Route path="/maps" element={<MapsView />} />
            <Route path="/maps/:mapId" element={<MapsView />} />
            <Route path="/dispatch" element={<DispatchView />} />
          </Routes>
        </SidebarInset>
      </SidebarProvider>
      <Toast />
    </div>
  );
}

export default App;
