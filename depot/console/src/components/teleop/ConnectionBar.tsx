import {
  GameController,
  Keyboard,
  Warning,
} from "@phosphor-icons/react";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { useConsoleStore } from "@/store";
import { InputSource } from "@/lib/types";

export function ConnectionBar() {
  const { inputSource } = useConsoleStore();

  return (
    <div className="px-3 py-1.5 flex items-center justify-end gap-4 text-xs text-muted-foreground">
        {inputSource === InputSource.Gamepad ? (
          <>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="flex items-center gap-1 cursor-help">
                  <GameController className="h-3.5 w-3.5" weight="fill" />
                  L-Stick: Drive
                </span>
              </TooltipTrigger>
              <TooltipContent side="top">Left stick controls forward/back and steering</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="cursor-help">R-Stick: Camera</span>
              </TooltipTrigger>
              <TooltipContent side="top">Right stick controls camera in Free mode</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="cursor-help">Start: Enable · R3: Disable</span>
              </TooltipTrigger>
              <TooltipContent side="top">Start enters teleop mode, R3 (right stick click) exits</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="text-destructive flex items-center gap-1 cursor-help">
                  <Warning className="h-3.5 w-3.5" weight="fill" />
                  Select: E-STOP
                </span>
              </TooltipTrigger>
              <TooltipContent side="top">Emergency stop - immediately halts all motors</TooltipContent>
            </Tooltip>
          </>
        ) : (
          <>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="flex items-center gap-1 cursor-help">
                  <Keyboard className="h-3.5 w-3.5" weight="fill" />
                  WASD: Drive
                </span>
              </TooltipTrigger>
              <TooltipContent side="top">W/S forward/back, A/D turn left/right</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="cursor-help">Enter: Enable · Bksp: Disable</span>
              </TooltipTrigger>
              <TooltipContent side="top">Enter key enables teleop, Backspace disables</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="cursor-help">C: View · V: Free</span>
              </TooltipTrigger>
              <TooltipContent side="top">C cycles camera view, V toggles free-look mode</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="text-destructive flex items-center gap-1 cursor-help">
                  <Warning className="h-3.5 w-3.5" weight="fill" />
                  Esc: E-STOP
                </span>
              </TooltipTrigger>
              <TooltipContent side="top">Emergency stop - immediately halts all motors</TooltipContent>
            </Tooltip>
          </>
        )}
    </div>
  );
}
