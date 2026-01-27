import { useState, useEffect } from "react";
import { StatusPanel } from "./StatusPanel";
import { DiagnosticsPanel } from "./DiagnosticsPanel";
import { InputPanel } from "./InputPanel";
import { CameraFeedSection } from "@/components/scene/EquirectangularSky";
import { useConsoleStore } from "@/store";

interface TeleopSidebarProps {
  onToggleLidar?: (enabled: boolean) => void;
}

export function TeleopSidebar({ onToggleLidar }: TeleopSidebarProps) {
  const connected = useConsoleStore((s) => s.connected);
  const [latencyMs, setLatencyMs] = useState(0);
  useEffect(() => {
    const id = setInterval(() => {
      setLatencyMs(useConsoleStore.getState().latencyMs);
    }, 250);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="w-56 bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg flex flex-col max-h-full">
      {/* Connection / latency top bar */}
      <div className="flex items-center justify-center px-2 py-0.5 border-b border-border">
        <span className={`text-xs font-mono ${connected ? 'text-green-500' : 'text-red-500'}`}>
          {connected ? `${latencyMs}ms` : 'OFFLINE'}
        </span>
      </div>

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
