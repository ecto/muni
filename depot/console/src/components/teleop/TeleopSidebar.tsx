import { StatusPanel } from "./StatusPanel";
import { DiagnosticsPanel } from "./DiagnosticsPanel";
import { InputPanel } from "./InputPanel";
import { CameraFeedSection } from "@/components/scene/EquirectangularSky";

interface TeleopSidebarProps {
  onToggleLidar?: (enabled: boolean) => void;
}

export function TeleopSidebar({ onToggleLidar }: TeleopSidebarProps) {
  return (
    <div className="w-56 bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg flex flex-col max-h-full">
      <div className="overflow-y-auto overflow-x-hidden scrollbar-thin">
        <StatusPanel />
        <div className="border-t border-border" />
        <DiagnosticsPanel onToggleLidar={onToggleLidar} />
        <div className="border-t border-border" />
        <InputPanel />
        <div className="border-t border-border" />
        <CameraFeedSection />
      </div>
    </div>
  );
}
