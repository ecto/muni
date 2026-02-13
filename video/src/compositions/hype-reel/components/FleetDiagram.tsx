// Animated fleet operations diagram — depot + rovers
import { useCurrentFrame, interpolate } from "remotion";
import { MUNI_ORANGE, WHITE, DARK_BG, FONT_MONO } from "../../../lib/brand";

const FONT_DISPLAY = '"Inter", "Helvetica Neue", Arial, sans-serif';

interface Node {
  label: string;
  sublabel?: string;
  x: number;
  y: number;
  delay: number;
  color?: string;
}

const DEPOT: Node = {
  label: "DEPOT",
  sublabel: "Fleet Ops",
  x: 960,
  y: 340,
  delay: 0,
  color: MUNI_ORANGE,
};

const ROVERS: Node[] = [
  { label: "BVR-01", sublabel: "Zone A", x: 400, y: 180, delay: 6 },
  { label: "BVR-02", sublabel: "Zone B", x: 400, y: 340, delay: 9 },
  { label: "BVR-03", sublabel: "Zone C", x: 400, y: 500, delay: 12 },
  { label: "BVR-04", sublabel: "Zone D", x: 580, y: 120, delay: 15 },
  { label: "BVR-05", sublabel: "Zone E", x: 580, y: 560, delay: 18 },
];

const SERVICES = [
  { label: "Console", x: 1200, y: 180, delay: 4 },
  { label: "Metrics", x: 1200, y: 340, delay: 5 },
  { label: "Dispatch", x: 1200, y: 500, delay: 6 },
];

function AnimatedNode({ node, frame }: { node: Node; frame: number }) {
  const opacity = interpolate(frame, [node.delay, node.delay + 4], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const scale = interpolate(frame, [node.delay, node.delay + 3], [0.8, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const color = node.color ?? WHITE;
  const borderColor = node.color ?? "rgba(255,255,255,0.3)";

  return (
    <div
      style={{
        position: "absolute",
        left: node.x - 70,
        top: node.y - 30,
        width: 140,
        height: 60,
        border: `2px solid ${borderColor}`,
        borderRadius: 8,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        opacity,
        transform: `scale(${scale})`,
        backgroundColor: "rgba(0,0,0,0.6)",
      }}
    >
      <div
        style={{
          fontFamily: FONT_MONO,
          fontSize: 18,
          fontWeight: 700,
          color,
          letterSpacing: 2,
        }}
      >
        {node.label}
      </div>
      {node.sublabel && (
        <div
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize: 12,
            color: "rgba(255,255,255,0.5)",
            marginTop: 2,
          }}
        >
          {node.sublabel}
        </div>
      )}
    </div>
  );
}

export function FleetDiagram() {
  const frame = useCurrentFrame();

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: DARK_BG,
        overflow: "hidden",
      }}
    >
      {/* Connection lines */}
      <svg
        style={{ position: "absolute", inset: 0 }}
        viewBox="0 0 1920 1080"
        preserveAspectRatio="none"
      >
        {ROVERS.map((rover) => {
          const lineOpacity = interpolate(
            frame,
            [rover.delay + 2, rover.delay + 6],
            [0, 0.4],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
          );
          // Animated dash offset
          const dashOffset = interpolate(frame, [0, 60], [100, 0], {
            extrapolateRight: "extend",
          });

          return (
            <line
              key={rover.label}
              x1={rover.x}
              y1={rover.y}
              x2={DEPOT.x}
              y2={DEPOT.y}
              stroke={MUNI_ORANGE}
              strokeWidth={1.5}
              opacity={lineOpacity}
              strokeDasharray="8,6"
              strokeDashoffset={dashOffset}
            />
          );
        })}
        {SERVICES.map((svc) => {
          const lineOpacity = interpolate(
            frame,
            [svc.delay + 2, svc.delay + 6],
            [0, 0.3],
            { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
          );
          return (
            <line
              key={svc.label}
              x1={DEPOT.x}
              y1={DEPOT.y}
              x2={svc.x}
              y2={svc.y}
              stroke="rgba(255,255,255,0.3)"
              strokeWidth={1}
              opacity={lineOpacity}
              strokeDasharray="4,4"
            />
          );
        })}
      </svg>

      {/* Nodes */}
      <AnimatedNode node={DEPOT} frame={frame} />
      {ROVERS.map((r) => (
        <AnimatedNode key={r.label} node={r} frame={frame} />
      ))}
      {SERVICES.map((s) => (
        <AnimatedNode
          key={s.label}
          node={{ ...s, sublabel: undefined, delay: s.delay }}
          frame={frame}
        />
      ))}

      {/* "1 Operator → 10 Rovers" label */}
      <div
        style={{
          position: "absolute",
          bottom: 80,
          left: 0,
          right: 0,
          textAlign: "center",
          fontFamily: FONT_MONO,
          fontSize: 18,
          color: MUNI_ORANGE,
          letterSpacing: 6,
          textTransform: "uppercase",
          opacity: interpolate(frame, [20, 28], [0, 0.7], {
            extrapolateRight: "clamp",
          }),
        }}
      >
        Fleet Operations Architecture
      </div>
    </div>
  );
}
