// Animated product roadmap timeline
import { useCurrentFrame, interpolate } from "remotion";
import { MUNI_ORANGE, WHITE, DARK_BG, FONT_MONO } from "../../../lib/brand";

const FONT_DISPLAY = '"Inter", "Helvetica Neue", Arial, sans-serif';

interface Milestone {
  quarter: string;
  title: string;
  items: string[];
  active?: boolean;
}

const MILESTONES: Milestone[] = [
  {
    quarter: "NOW",
    title: "Prototype",
    items: ["BVR1 hardware", "Teleop validated", "First snow removal"],
    active: true,
  },
  {
    quarter: "Q3 2026",
    title: "Pilot",
    items: ["3 rovers deployed", "Municipal partner", "Autonomy v1"],
  },
  {
    quarter: "Q1 2027",
    title: "Scale",
    items: ["Fleet of 10+", "RaaS revenue", "Multi-city"],
  },
  {
    quarter: "2028+",
    title: "Expand",
    items: ["Year-round ops", "Trash + sweeping", "100+ units"],
  },
];

const TIMELINE_Y = 440;
const START_X = 200;
const SPACING = 420;

export function RoadmapTimeline() {
  const frame = useCurrentFrame();

  // Line draws left to right
  const lineProgress = interpolate(frame, [0, 30], [0, 1], {
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: DARK_BG,
        overflow: "hidden",
      }}
    >
      {/* Title */}
      <div
        style={{
          position: "absolute",
          top: 120,
          left: 0,
          right: 0,
          textAlign: "center",
          fontFamily: FONT_MONO,
          fontSize: 20,
          color: MUNI_ORANGE,
          letterSpacing: 6,
          textTransform: "uppercase",
          opacity: interpolate(frame, [0, 8], [0, 0.7], {
            extrapolateRight: "clamp",
          }),
        }}
      >
        Product Roadmap
      </div>

      {/* Timeline line */}
      <svg
        style={{ position: "absolute", inset: 0 }}
        viewBox="0 0 1920 1080"
        preserveAspectRatio="none"
      >
        <line
          x1={START_X}
          y1={TIMELINE_Y}
          x2={START_X + (MILESTONES.length - 1) * SPACING * lineProgress}
          y2={TIMELINE_Y}
          stroke="rgba(255,255,255,0.2)"
          strokeWidth={2}
        />
      </svg>

      {/* Milestones */}
      {MILESTONES.map((ms, i) => {
        const delay = i * 6 + 4;
        const opacity = interpolate(frame, [delay, delay + 5], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });
        const slideUp = interpolate(frame, [delay, delay + 5], [20, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });
        const x = START_X + i * SPACING;

        return (
          <div
            key={ms.quarter}
            style={{
              position: "absolute",
              left: x - 80,
              top: TIMELINE_Y - 200,
              width: 200,
              opacity,
              transform: `translateY(${slideUp}px)`,
            }}
          >
            {/* Quarter label */}
            <div
              style={{
                fontFamily: FONT_MONO,
                fontSize: 16,
                color: ms.active ? MUNI_ORANGE : "rgba(255,255,255,0.4)",
                letterSpacing: 4,
                marginBottom: 8,
              }}
            >
              {ms.quarter}
            </div>

            {/* Title */}
            <div
              style={{
                fontFamily: FONT_DISPLAY,
                fontSize: 32,
                fontWeight: 900,
                color: ms.active ? WHITE : "rgba(255,255,255,0.7)",
                marginBottom: 16,
                textTransform: "uppercase",
              }}
            >
              {ms.title}
            </div>

            {/* Items */}
            {ms.items.map((item, j) => (
              <div
                key={j}
                style={{
                  fontFamily: FONT_DISPLAY,
                  fontSize: 16,
                  color: "rgba(255,255,255,0.5)",
                  lineHeight: 1.6,
                }}
              >
                {item}
              </div>
            ))}

            {/* Dot on timeline */}
            <div
              style={{
                position: "absolute",
                left: 80 - 6,
                top: 200 - 6,
                width: 12,
                height: 12,
                borderRadius: "50%",
                backgroundColor: ms.active ? MUNI_ORANGE : "rgba(255,255,255,0.3)",
                border: ms.active ? `2px solid ${MUNI_ORANGE}` : "none",
              }}
            />
          </div>
        );
      })}
    </div>
  );
}
