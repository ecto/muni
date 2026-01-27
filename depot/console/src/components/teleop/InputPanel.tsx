import { useState, useEffect, memo } from "react";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { useConsoleStore } from "@/store";
import { InputSource, type GamepadInput } from "@/lib/types";

const BANK_ARC_TICKS = [0, 15, 30, -15, -30];

const CommandHud = memo(function CommandHud({
  linear,
  angular,
  toolAxis,
  inputSource,
}: {
  linear: number;
  angular: number;
  toolAxis: number;
  inputSource: InputSource;
}) {
  const w = 208;
  const h = 110;
  const cx = w / 2;
  const cy = h / 2;
  const range = 35; // ±1 maps to ±35px

  // FPM position: X = angular, Y = -linear (up = forward)
  const fpmX = cx + angular * range;
  const fpmY = cy - linear * range;

  // Pitch ladder shift
  const ladderShift = -linear * range;

  // Bank pointer angle (angular maps to rotation)
  const bankDeg = angular * 30;

  // Tool bar: fills from center, range ±30px
  const toolBarH = 30;
  const toolBarX = w - 14;
  const toolFill = toolAxis * toolBarH;

  const sourceGlyph = inputSource === InputSource.Gamepad ? "🎮" : "⌨";

  const fmt = (v: number) => (v >= 0 ? "+" : "") + v.toFixed(2);

  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="w-full block">
      {/* Bank arc at top */}
      {BANK_ARC_TICKS.map((deg) => {
        const angle = ((deg - 90) * Math.PI) / 180;
        const arcR = 44;
        const inner = arcR - 4;
        const outer = arcR + 1;
        return (
          <line
            key={deg}
            x1={cx + Math.cos(angle) * inner}
            y1={cy - 36 + Math.sin(angle) * inner}
            x2={cx + Math.cos(angle) * outer}
            y2={cy - 36 + Math.sin(angle) * outer}
            className={deg === 0 ? "stroke-emerald-400" : "stroke-stone-500/60"}
            strokeWidth={deg === 0 ? 1.5 : 1}
          />
        );
      })}
      {/* Bank pointer (rotates with angular) */}
      <g transform={`rotate(${bankDeg}, ${cx}, ${cy - 36})`}>
        <polygon
          points={`${cx},${cy - 36 - 40} ${cx - 3},${cy - 36 - 34} ${cx + 3},${cy - 36 - 34}`}
          className="fill-emerald-400"
        />
      </g>

      {/* Pitch ladder (shifts with linear) */}
      {[-1, 1].map((dir) => {
        const yOff = cy + ladderShift + dir * 16;
        return (
          <g key={dir}>
            <line x1={cx - 18} y1={yOff} x2={cx - 6} y2={yOff} className="stroke-stone-500/50" strokeWidth={1} />
            <line x1={cx + 6} y1={yOff} x2={cx + 18} y2={yOff} className="stroke-stone-500/50" strokeWidth={1} />
          </g>
        );
      })}

      {/* Boresight cross (fixed reference at center) */}
      <line x1={cx - 8} y1={cy} x2={cx - 3} y2={cy} className="stroke-stone-400" strokeWidth={1} />
      <line x1={cx + 3} y1={cy} x2={cx + 8} y2={cy} className="stroke-stone-400" strokeWidth={1} />
      <line x1={cx} y1={cy - 8} x2={cx} y2={cy - 3} className="stroke-stone-400" strokeWidth={1} />
      <line x1={cx} y1={cy + 3} x2={cx} y2={cy + 8} className="stroke-stone-400" strokeWidth={1} />

      {/* Flight path marker */}
      <circle cx={fpmX} cy={fpmY} r={5} fill="none" className="stroke-emerald-400" strokeWidth={1.5} />
      <line x1={fpmX - 10} y1={fpmY} x2={fpmX - 5} y2={fpmY} className="stroke-emerald-400" strokeWidth={1.5} />
      <line x1={fpmX + 5} y1={fpmY} x2={fpmX + 10} y2={fpmY} className="stroke-emerald-400" strokeWidth={1.5} />
      <line x1={fpmX} y1={fpmY - 10} x2={fpmX} y2={fpmY - 5} className="stroke-emerald-400" strokeWidth={1.5} />

      {/* Tool bar (right edge, fills from center) */}
      <line x1={toolBarX} y1={cy - toolBarH} x2={toolBarX} y2={cy + toolBarH} className="stroke-stone-600" strokeWidth={3} strokeLinecap="round" />
      {toolFill !== 0 && (
        <line
          x1={toolBarX}
          y1={cy}
          x2={toolBarX}
          y2={cy - toolFill}
          className="stroke-emerald-400"
          strokeWidth={3}
          strokeLinecap="round"
        />
      )}
      <line x1={toolBarX - 3} y1={cy} x2={toolBarX + 3} y2={cy} className="stroke-stone-400" strokeWidth={1} />

      {/* Readouts */}
      <text x={16} y={cy + 2} className="fill-stone-400" fontSize={10} fontFamily="monospace" textAnchor="start">
        {fmt(linear)}
      </text>
      <text x={cx} y={h - 6} className="fill-stone-400" fontSize={10} fontFamily="monospace" textAnchor="middle">
        {fmt(angular)}
      </text>
      <text x={toolBarX} y={cy + toolBarH + 12} className="fill-stone-400" fontSize={9} fontFamily="monospace" textAnchor="middle">
        {fmt(toolAxis)}
      </text>

      {/* Source icon */}
      <text x={w - 6} y={14} className="fill-stone-500" fontSize={11} textAnchor="end">
        {sourceGlyph}
      </text>
    </svg>
  );
});

const defaultInput: GamepadInput = {
  linear: 0, angular: 0, toolAxis: 0,
  actionA: false, actionB: false, estop: false,
  enable: false, disable: false, boost: false,
  cameraYaw: 0, cameraPitch: 0,
};

export function InputPanel() {
  const inputSource = useConsoleStore((s) => s.inputSource);

  // Poll input at 10Hz instead of subscribing (input updates at 60Hz from keyboard/gamepad)
  const [input, setInput] = useState<GamepadInput>(defaultInput);
  useEffect(() => {
    const id = setInterval(() => {
      setInput(useConsoleStore.getState().input);
    }, 100);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="p-2 space-y-1.5">
      <CommandHud
        linear={input.linear}
        angular={input.angular}
        toolAxis={input.toolAxis}
        inputSource={inputSource}
      />

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
  );
}
