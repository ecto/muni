import { useState, useEffect } from "react";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { GameController, Keyboard, CaretDown, CaretRight } from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { InputSource, type GamepadInput } from "@/lib/types";

function AxisBar({ value, label, tooltip }: { value: number; label: string; tooltip: string }) {
  // Map -1..1 to 0..100
  const percent = ((value + 1) / 2) * 100;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div className="space-y-0.5 cursor-help">
          <div className="flex items-center justify-between text-xs">
            <span className="text-muted-foreground">{label}</span>
            <span className="font-mono w-10 text-right text-stone-900 dark:text-stone-100">
              {value >= 0 ? "+" : ""}
              {value.toFixed(2)}
            </span>
          </div>
          <Progress value={percent} className="h-1.5" />
        </div>
      </TooltipTrigger>
      <TooltipContent side="right">{tooltip}</TooltipContent>
    </Tooltip>
  );
}

const defaultInput: GamepadInput = {
  linear: 0, angular: 0, toolAxis: 0,
  actionA: false, actionB: false, estop: false,
  enable: false, disable: false, boost: false,
  cameraYaw: 0, cameraPitch: 0,
};

export function InputPanel() {
  const inputSource = useConsoleStore((s) => s.inputSource);
  const [collapsed, setCollapsed] = useState(false);

  // Poll input at 10Hz instead of subscribing (input updates at 60Hz from keyboard/gamepad)
  const [input, setInput] = useState<GamepadInput>(defaultInput);
  useEffect(() => {
    const id = setInterval(() => {
      setInput(useConsoleStore.getState().input);
    }, 100);
    return () => clearInterval(id);
  }, []);

  const sourceIcon = inputSource === InputSource.Gamepad
    ? <GameController className="h-3.5 w-3.5" weight="fill" />
    : <Keyboard className="h-3.5 w-3.5" weight="fill" />;

  const sourceLabel = inputSource === InputSource.Gamepad ? "Gamepad" : "Keyboard";

  return (
    <div className="w-44 bg-card/90 backdrop-blur-sm border border-border rounded-lg overflow-hidden shadow-lg">
      {/* Header */}
      <div
        className="flex items-center justify-between px-2 py-1 bg-muted/50 border-b border-border cursor-pointer hover:bg-muted/70"
        onClick={() => setCollapsed(!collapsed)}
      >
        <span className="text-xs font-medium text-muted-foreground flex items-center gap-1">
          {collapsed ? <CaretRight className="h-3 w-3" /> : <CaretDown className="h-3 w-3" />}
          Input
        </span>
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="flex items-center gap-1 text-xs text-muted-foreground cursor-help">
              {sourceIcon}
              {sourceLabel}
            </span>
          </TooltipTrigger>
          <TooltipContent side="right">Active input source</TooltipContent>
        </Tooltip>
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="p-2 space-y-1.5">
          <AxisBar value={input.linear} label="Linear" tooltip="Forward (+) / Reverse (-) throttle" />
          <AxisBar value={input.angular} label="Angular" tooltip="Left (+) / Right (-) steering" />
          <AxisBar value={input.toolAxis} label="Tool" tooltip="Attachment control (plow up/down)" />

          {/* Buttons */}
          <div className="grid grid-cols-4 gap-1 pt-1 border-t border-border">
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge
                  variant={input.boost ? "default" : "outline"}
                  className="text-[10px] h-4 px-0.5 justify-center cursor-help"
                >
                  BOOST
                </Badge>
              </TooltipTrigger>
              <TooltipContent side="bottom">Hold for 2x speed multiplier</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge
                  variant={input.actionA ? "default" : "outline"}
                  className="text-[10px] h-4 px-0.5 justify-center cursor-help"
                >
                  A
                </Badge>
              </TooltipTrigger>
              <TooltipContent side="bottom">Action A (context-dependent)</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge
                  variant={input.actionB ? "default" : "outline"}
                  className="text-[10px] h-4 px-0.5 justify-center cursor-help"
                >
                  B
                </Badge>
              </TooltipTrigger>
              <TooltipContent side="bottom">Action B (context-dependent)</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge
                  variant={input.estop ? "destructive" : "outline"}
                  className="text-[10px] h-4 px-0.5 justify-center cursor-help"
                >
                  STOP
                </Badge>
              </TooltipTrigger>
              <TooltipContent side="bottom">Software E-Stop triggered</TooltipContent>
            </Tooltip>
          </div>
        </div>
      )}
    </div>
  );
}
