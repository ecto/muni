import {
  CloudSnow,
  Wind,
  Drop,
  Sun,
  Cloud,
  CloudRain,
} from "@phosphor-icons/react";
import type { WeatherResponse } from "@/lib/types";

function WeatherIcon({ forecast }: { forecast: string }) {
  const lower = forecast.toLowerCase();
  if (lower.includes("snow")) return <CloudSnow className="h-8 w-8 text-blue-300" weight="fill" />;
  if (lower.includes("rain") || lower.includes("shower"))
    return <CloudRain className="h-8 w-8 text-blue-400" weight="fill" />;
  if (lower.includes("cloud") || lower.includes("overcast"))
    return <Cloud className="h-8 w-8 text-zinc-400" weight="fill" />;
  return <Sun className="h-8 w-8 text-yellow-400" weight="fill" />;
}

function ForecastBar({ periods }: { periods: WeatherResponse["forecast"] }) {
  // Show next 8 hours
  const next8 = periods.slice(0, 8);
  if (next8.length === 0) return null;

  return (
    <div className="flex gap-1 mt-2">
      {next8.map((p, i) => {
        const hour = new Date(p.startTime).getHours();
        const label = hour === 0 ? "12a" : hour < 12 ? `${hour}a` : hour === 12 ? "12p" : `${hour - 12}p`;
        return (
          <div key={i} className="flex-1 text-center">
            <div className="text-[10px] text-muted-foreground">{label}</div>
            <div className="text-xs font-medium">{Math.round(p.temperatureF)}</div>
            {p.isSnow && <CloudSnow className="h-3 w-3 mx-auto text-blue-300" weight="fill" />}
          </div>
        );
      })}
    </div>
  );
}

export function WeatherCard({ weather }: { weather: WeatherResponse | null }) {
  if (!weather?.current) {
    return (
      <div className="p-3">
        <div className="text-xs text-muted-foreground text-center py-4">
          <CloudSnow className="h-8 w-8 mx-auto mb-2 opacity-30" />
          Weather data unavailable
        </div>
      </div>
    );
  }

  const { current, forecast, updatedAt } = weather;

  const snowPeriods = forecast.filter((p) => p.isSnow);
  const snowSummary = snowPeriods.length > 0
    ? `Snow expected in ${snowPeriods.length} of next ${forecast.length}h`
    : null;

  return (
    <div className="p-3 space-y-2">
      {/* Current conditions */}
      <div className="flex items-center gap-3">
        <WeatherIcon forecast={current.shortForecast} />
        <div className="flex-1">
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold">
              {Math.round(current.temperatureF)}°
            </span>
            <span className="text-sm text-muted-foreground">
              {current.shortForecast}
            </span>
          </div>
          <div className="flex items-center gap-3 text-xs text-muted-foreground mt-0.5">
            <span className="flex items-center gap-1">
              <Wind className="h-3 w-3" />
              {Math.round(current.windSpeedMph)} mph {current.windDirection}
            </span>
            {current.humidityPct != null && (
              <span className="flex items-center gap-1">
                <Drop className="h-3 w-3" />
                {Math.round(current.humidityPct)}%
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Snow alert */}
      {snowSummary && (
        <div className="flex items-center gap-2 px-2 py-1.5 bg-blue-500/10 border border-blue-500/20 rounded text-xs text-blue-400">
          <CloudSnow className="h-3.5 w-3.5" weight="fill" />
          {snowSummary}
        </div>
      )}

      {current.isSnowing && (
        <div className="flex items-center gap-2 px-2 py-1.5 bg-blue-500/10 border border-blue-500/20 rounded text-xs text-blue-300 font-medium">
          <CloudSnow className="h-3.5 w-3.5 animate-pulse" weight="fill" />
          Currently snowing
        </div>
      )}

      {/* Hourly forecast */}
      <ForecastBar periods={forecast} />

      {/* Updated timestamp */}
      {updatedAt && (
        <div className="text-[10px] text-muted-foreground/50 text-right">
          Updated {new Date(updatedAt).toLocaleTimeString()}
        </div>
      )}
    </div>
  );
}
