import {
  Plugs,
  PlugsConnected,
  GameController,
  Keyboard,
  Warning,
} from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { InputSource } from "@/lib/types";

export function ConnectionBar() {
  const { connected, latencyMs, inputSource } = useConsoleStore();

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

      {/* Input hints based on source - right aligned for stability */}
      <div className="flex-1 flex items-center justify-end gap-6 text-muted-foreground">
        {inputSource === InputSource.Gamepad ? (
          <>
            <span className="flex items-center gap-1">
              <GameController className="h-4 w-4" weight="fill" />
              L-Stick: Drive
            </span>
            <span>R-Stick: Camera</span>
            <span>Start: Enable · R3: Disable</span>
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
            <span>Enter: Enable · Bksp: Disable</span>
            <span>C: View · V: Free</span>
            <span className="text-destructive flex items-center gap-1">
              <Warning className="h-4 w-4" weight="fill" />
              Esc: E-STOP
            </span>
          </>
        )}
      </div>
    </div>
  );
}
