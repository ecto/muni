import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import {
  BatteryFull,
  BatteryLow,
  BatteryWarning,
  Lightning,
  Gauge,
} from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { Mode, ModeLabels } from "@/lib/types";

function getModeVariant(
  mode: Mode
): "default" | "secondary" | "destructive" | "outline" {
  switch (mode) {
    case Mode.Teleop:
    case Mode.Autonomous:
      return "default";
    case Mode.EStop:
    case Mode.Fault:
      return "destructive";
    case Mode.Idle:
      return "secondary";
    default:
      return "outline";
  }
}

function BatteryIcon({ voltage }: { voltage: number }) {
  if (voltage < 42)
    return (
      <BatteryWarning className="h-4 w-4 text-destructive" weight="fill" />
    );
  if (voltage < 45)
    return <BatteryLow className="h-4 w-4 text-orange-500" weight="fill" />;
  return <BatteryFull className="h-4 w-4 text-green-500" weight="fill" />;
}

export function TelemetryPanel() {
  const { telemetry, connected } = useConsoleStore();

  // Battery percentage (48V system: 39V empty, 54.6V full)
  const batteryPercent = Math.max(
    0,
    Math.min(100, ((telemetry.power.battery_voltage - 39) / (54.6 - 39)) * 100)
  );

  return (
    <Card className="w-64 bg-card/90 backdrop-blur">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium flex items-center justify-between">
          Telemetry
          <Badge variant={connected ? "default" : "destructive"}>
            {connected ? "Connected" : "Disconnected"}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Mode */}
        <div className="flex items-center justify-between">
          <span className="text-sm text-muted-foreground">Mode</span>
          <Badge variant={getModeVariant(telemetry.mode)}>
            {ModeLabels[telemetry.mode]}
          </Badge>
        </div>

        <Separator />

        {/* Battery */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="flex items-center gap-2 text-muted-foreground">
              <BatteryIcon voltage={telemetry.power.battery_voltage} />
              Battery
            </span>
            <span className="font-mono">
              {telemetry.power.battery_voltage.toFixed(1)}V
            </span>
          </div>
          <Progress value={batteryPercent} className="h-2" />
        </div>

        {/* Current */}
        <div className="flex items-center justify-between text-sm">
          <span className="flex items-center gap-2 text-muted-foreground">
            <Lightning className="h-4 w-4" weight="fill" />
            Current
          </span>
          <span className="font-mono">
            {telemetry.power.system_current.toFixed(1)}A
          </span>
        </div>

        <Separator />

        {/* Velocity */}
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Gauge className="h-4 w-4" weight="fill" />
            Velocity
          </div>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Linear</span>
              <span className="font-mono">
                {telemetry.velocity.linear.toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Angular</span>
              <span className="font-mono">
                {telemetry.velocity.angular.toFixed(2)}
              </span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
