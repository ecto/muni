interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  color?: string;
  showDot?: boolean;
}

export function Sparkline({
  data,
  width = 48,
  height = 14,
  color = "#22c55e",
  showDot = true,
}: SparklineProps) {
  if (data.length === 0) {
    return (
      <svg width={width} height={height} className="inline-block">
        <line
          x1={0}
          y1={height / 2}
          x2={width}
          y2={height / 2}
          stroke="#6b7280"
          strokeWidth={1}
          strokeDasharray="2,2"
        />
      </svg>
    );
  }

  // Normalize data to fit within height
  const max = Math.max(...data, 1); // At least 1 to avoid division by zero
  const padding = 1;
  const effectiveHeight = height - padding * 2;

  // Calculate points
  const step = data.length > 1 ? width / (data.length - 1) : width;
  const points = data.map((value, i) => {
    const x = data.length > 1 ? i * step : width / 2;
    const y = padding + effectiveHeight - (value / max) * effectiveHeight;
    return `${x},${y}`;
  });

  const polylinePoints = points.join(" ");

  // Create fill area (closed polygon)
  const fillPoints = [
    `0,${height}`,
    ...points,
    `${width},${height}`,
  ].join(" ");

  const currentValue = data[data.length - 1];
  const lastX = data.length > 1 ? (data.length - 1) * step : width / 2;
  const lastY = padding + effectiveHeight - (currentValue / max) * effectiveHeight;

  return (
    <svg width={width} height={height} className="inline-block">
      {/* Filled area under the line */}
      <polygon
        points={fillPoints}
        fill={color}
        fillOpacity={0.2}
      />
      {/* Line */}
      <polyline
        points={polylinePoints}
        fill="none"
        stroke={color}
        strokeWidth={1}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      {/* Current value dot */}
      {showDot && data.length > 0 && (
        <circle
          cx={lastX}
          cy={lastY}
          r={2}
          fill={color}
        />
      )}
    </svg>
  );
}
