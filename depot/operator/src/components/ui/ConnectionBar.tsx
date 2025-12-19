import {
  Plugs,
  PlugsConnected,
  GameController,
  Keyboard,
  Warning,
} from "@phosphor-icons/react";
import { useOperatorStore } from "@/store";
import { InputSource } from "@/lib/types";

export function ConnectionBar() {
  const { connected, latencyMs, inputSource } = useOperatorStore();

  return (
    <div className="bg-card/90 backdrop-blur px-4 py-2 flex items-center gap-6 text-sm">
      {/* Connection status */}
      <div className="flex items-center gap-2">
        {connected ? (
          <PlugsConnected className="h-4 w-4 text-green-500" weight="fill" />
        ) : (
          <Plugs className="h-4 w-4 text-destructive" weight="regular" />
        )}
        <span className={connected ? "text-foreground" : "text-destructive"}>
          {connected ? `${latencyMs}ms` : "Disconnected"}
        </span>
      </div>

      {/* Input hints based on source */}
      <div className="flex-1 flex items-center justify-center gap-6 text-muted-foreground">
        {inputSource === InputSource.Gamepad ? (
          <>
            <span className="flex items-center gap-1">
              <GameController className="h-4 w-4" weight="fill" />
              L-Stick: Drive
            </span>
            <span>R-Stick: Camera</span>
            <span>Triggers: Tool</span>
            <span className="text-destructive flex items-center gap-1">
              <Warning className="h-4 w-4" weight="fill" />
              Select: E-STOP
            </span>
          </>
        ) : (
          <>
            <span className="flex items-center gap-1">
              <Keyboard className="h-4 w-4" weight="fill" />
              WASD: Drive
            </span>
            <span>RMB+Drag: Camera</span>
            <span>C: View · V: Free</span>
            <span className="text-destructive flex items-center gap-1">
              <Warning className="h-4 w-4" weight="fill" />
              Esc: E-STOP
            </span>
          </>
        )}
      </div>

      {/* Spacer for symmetry */}
      <div className="w-24" />
    </div>
  );
}
