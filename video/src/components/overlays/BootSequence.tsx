// Terminal-style checklist for closing sequence
import { useCurrentFrame } from "remotion";
import { MUNI_ORANGE, FONT_MONO, WHITE, DARK_BG } from "../../lib/brand";

const CHECKLIST_ITEMS = [
  "Autonomous navigation",
  "Electric drivetrain",
  "Fleet management",
  "Teleop fallback",
  "3D mapping",
  "Behavioral cloning",
  "Open source",
  "Sidewalk-scale",
  "Revenue model",
  "Municipal pilot imminent",
];

interface BootSequenceProps {
  /** How many frames between each line appearing */
  staggerFrames?: number;
}

export function BootSequence({ staggerFrames = 8 }: BootSequenceProps) {
  const frame = useCurrentFrame();

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: DARK_BG,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {CHECKLIST_ITEMS.map((item, i) => {
          const appearFrame = i * staggerFrames;
          const visible = frame >= appearFrame;
          const checkFrame = appearFrame + 4; // checkmark appears slightly after
          const checked = frame >= checkFrame;

          if (!visible) return null;

          return (
            <div
              key={i}
              style={{
                fontFamily: FONT_MONO,
                fontSize: 32,
                color: checked ? WHITE : "rgba(255,255,255,0.3)",
                display: "flex",
                alignItems: "center",
                gap: 16,
                transition: "color 0.1s",
              }}
            >
              <span
                style={{
                  color: checked ? MUNI_ORANGE : "rgba(255,255,255,0.15)",
                  fontSize: 28,
                  width: 32,
                  textAlign: "center",
                }}
              >
                {checked ? "\u2713" : "\u25CB"}
              </span>
              {item}
            </div>
          );
        })}
      </div>
    </div>
  );
}
