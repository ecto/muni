import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import { GameController, Keyboard, Circle } from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { InputSource } from "@/lib/types";

function InputSourceIcon({ source }: { source: InputSource }) {
  switch (source) {
    case InputSource.Gamepad:
      return <GameController className="h-4 w-4" weight="fill" />;
    case InputSource.Keyboard:
      return <Keyboard className="h-4 w-4" weight="fill" />;
    default:
      return <Circle className="h-4 w-4" weight="regular" />;
  }
}

function AxisBar({ value, label }: { value: number; label: string }) {
  // Map -1..1 to 0..100
  const percent = ((value + 1) / 2) * 100;

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between text-sm">
        <span className="text-muted-foreground">{label}</span>
        <span className="font-mono w-12 text-right">
          {value >= 0 ? "+" : ""}
          {value.toFixed(2)}
        </span>
      </div>
      <Progress value={percent} className="h-2" />
    </div>
  );
}

export function InputPanel() {
  const { input, inputSource } = useConsoleStore();

  const sourceLabel = {
    [InputSource.None]: "No Input",
    [InputSource.Keyboard]: "Keyboard",
    [InputSource.Gamepad]: "Gamepad",
  }[inputSource];

  return (
    <Card className="w-64 bg-card/90 backdrop-blur">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium flex items-center justify-between">
          Input
          <Badge
            variant={inputSource === InputSource.None ? "outline" : "secondary"}
            className="flex items-center gap-1"
          >
            <InputSourceIcon source={inputSource} />
            {sourceLabel}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <AxisBar value={input.linear} label="Linear" />
        <AxisBar value={input.angular} label="Angular" />
        <AxisBar value={input.toolAxis} label="Tool" />

        <Separator />

        {/* Buttons */}
        <div className="flex items-center gap-2 text-sm">
          <Badge
            variant={input.actionA ? "default" : "outline"}
            className="text-xs"
          >
            A
          </Badge>
          <Badge
            variant={input.actionB ? "default" : "outline"}
            className="text-xs"
          >
            B
          </Badge>
          <Badge
            variant={input.estop ? "destructive" : "outline"}
            className="text-xs"
          >
            STOP
          </Badge>
          <Badge
            variant={input.enable ? "default" : "outline"}
            className="text-xs"
          >
            EN
          </Badge>
        </div>
      </CardContent>
    </Card>
  );
}
