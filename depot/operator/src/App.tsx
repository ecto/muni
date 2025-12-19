import { Scene } from "@/components/scene/Scene";
import { VideoStatusBadge } from "@/components/scene/EquirectangularSky";
import { TelemetryPanel } from "@/components/ui/TelemetryPanel";
import { InputPanel } from "@/components/ui/InputPanel";
import { PositionPanel } from "@/components/ui/PositionPanel";
import { ConnectionBar } from "@/components/ui/ConnectionBar";
import { useKeyboard } from "@/hooks/useKeyboard";
import { useGamepad } from "@/hooks/useGamepad";
import { useRoverConnection } from "@/hooks/useRoverConnection";
import { useVideoStream } from "@/hooks/useVideoStream";

function App() {
  // Initialize input and connection hooks
  useKeyboard();
  useGamepad();
  useRoverConnection();
  useVideoStream();

  return (
    <div className="h-screen w-screen overflow-hidden bg-background">
      {/* 3D Scene (full screen) */}
      <Scene />

      {/* UI Overlay */}
      <div className="absolute inset-0 pointer-events-none">
        {/* Top left panels */}
        <div className="absolute top-4 left-4 flex flex-col gap-4 pointer-events-auto">
          <TelemetryPanel />
          <InputPanel />
          <PositionPanel />
        </div>

        {/* Top right video status */}
        <div className="pointer-events-auto">
          <VideoStatusBadge />
        </div>

        {/* Bottom bar */}
        <div className="absolute bottom-0 left-0 right-0 pointer-events-auto">
          <ConnectionBar />
        </div>
      </div>
    </div>
  );
}

export default App;
