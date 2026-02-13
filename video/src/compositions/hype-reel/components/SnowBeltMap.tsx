// Animated US snow belt map — highlights target cities
import { useCurrentFrame, interpolate } from "remotion";
import { MUNI_ORANGE, WHITE, DARK_BG, FONT_MONO } from "../../../lib/brand";

const SNOW_BELT_CITIES = [
  { name: "Minneapolis", x: 690, y: 240, delay: 0 },
  { name: "Chicago", x: 750, y: 310, delay: 3 },
  { name: "Detroit", x: 810, y: 280, delay: 6 },
  { name: "Cleveland", x: 830, y: 300, delay: 9 },
  { name: "Buffalo", x: 880, y: 260, delay: 12 },
  { name: "Boston", x: 960, y: 250, delay: 15 },
  { name: "Denver", x: 500, y: 340, delay: 18 },
  { name: "Milwaukee", x: 720, y: 270, delay: 21 },
  { name: "Pittsburgh", x: 850, y: 320, delay: 24 },
  { name: "Salt Lake City", x: 420, y: 320, delay: 27 },
];

// Simplified US outline as SVG path (continental)
const US_PATH =
  "M200,350 L220,280 L280,250 L350,230 L400,200 L430,180 L480,170 L530,175 L560,190 L600,200 L650,210 L700,200 L750,190 L800,180 L850,190 L900,200 L950,220 L980,250 L990,290 L970,340 L950,370 L920,390 L880,400 L840,410 L800,430 L760,440 L720,445 L680,440 L640,430 L600,420 L560,430 L520,440 L480,450 L440,440 L400,420 L360,410 L320,400 L280,390 L240,380 L210,370 Z";

// Snow belt region (northern states arc)
const SNOW_BELT_PATH =
  "M380,180 L420,170 L480,165 L540,170 L600,180 L660,190 L720,185 L780,180 L840,185 L900,195 L960,230 L950,280 L900,300 L840,310 L780,300 L720,290 L660,295 L600,300 L540,290 L480,280 L420,270 L380,250 Z";

export function SnowBeltMap() {
  const frame = useCurrentFrame();

  const mapOpacity = interpolate(frame, [0, 10], [0, 1], { extrapolateRight: "clamp" });
  const beltOpacity = interpolate(frame, [5, 15], [0, 0.3], { extrapolateRight: "clamp" });

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: DARK_BG,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        overflow: "hidden",
      }}
    >
      <svg
        viewBox="100 100 1000 450"
        style={{ width: "85%", height: "85%", opacity: mapOpacity }}
      >
        {/* US outline */}
        <path
          d={US_PATH}
          fill="none"
          stroke="rgba(255,255,255,0.15)"
          strokeWidth={2}
        />

        {/* Snow belt highlight */}
        <path
          d={SNOW_BELT_PATH}
          fill={MUNI_ORANGE}
          opacity={beltOpacity}
          stroke="none"
        />

        {/* City dots + labels */}
        {SNOW_BELT_CITIES.map((city) => {
          const dotOpacity = interpolate(
            frame,
            [city.delay, city.delay + 4],
            [0, 1],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
          );
          const dotScale = interpolate(
            frame,
            [city.delay, city.delay + 3],
            [0, 1],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
          );
          // Pulse ring
          const ringScale = interpolate(
            frame,
            [city.delay + 2, city.delay + 12],
            [1, 2.5],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
          );
          const ringOpacity = interpolate(
            frame,
            [city.delay + 2, city.delay + 12],
            [0.6, 0],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
          );

          return (
            <g key={city.name} opacity={dotOpacity}>
              {/* Pulse ring */}
              <circle
                cx={city.x}
                cy={city.y}
                r={6 * ringScale}
                fill="none"
                stroke={MUNI_ORANGE}
                strokeWidth={1.5}
                opacity={ringOpacity}
              />
              {/* Dot */}
              <circle
                cx={city.x}
                cy={city.y}
                r={5 * dotScale}
                fill={MUNI_ORANGE}
              />
              {/* Label */}
              <text
                x={city.x + 10}
                y={city.y + 4}
                fill={WHITE}
                fontSize={13}
                fontFamily="Inter, sans-serif"
                fontWeight={700}
                opacity={0.8}
              >
                {city.name}
              </text>
            </g>
          );
        })}
      </svg>

      {/* Title */}
      <div
        style={{
          position: "absolute",
          top: 80,
          left: 100,
          fontFamily: FONT_MONO,
          fontSize: 20,
          color: MUNI_ORANGE,
          letterSpacing: 6,
          textTransform: "uppercase",
          opacity: interpolate(frame, [0, 8], [0, 0.7], { extrapolateRight: "clamp" }),
        }}
      >
        Snow Belt Target Markets
      </div>
    </div>
  );
}
