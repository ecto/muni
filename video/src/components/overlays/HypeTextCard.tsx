// Full-screen text card — Inter Black, all caps, hard cuts
import { useCurrentFrame } from "remotion";
import { interpolate } from "remotion";
import { WHITE, MUNI_ORANGE, DARK_BG } from "../../lib/brand";

const FONT_DISPLAY = '"Inter", "Helvetica Neue", Arial, sans-serif';

interface HypeTextCardProps {
  text: string;
  tag?: string;
  bgColor?: string;
  fontSize?: number;
  color?: string;
}

export function HypeTextCard({
  text,
  tag,
  bgColor = DARK_BG,
  fontSize = 96,
  color = WHITE,
}: HypeTextCardProps) {
  const frame = useCurrentFrame();

  // Hard cut in — no fade, just appears. Slight scale punch on entry.
  const scale = interpolate(frame, [0, 4], [1.05, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const lines = text.split("\n");

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        backgroundColor: bgColor,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        transform: `scale(${scale})`,
      }}
    >
      {tag && (
        <div
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize: 24,
            fontWeight: 900,
            color: MUNI_ORANGE,
            letterSpacing: 8,
            textTransform: "uppercase",
            marginBottom: 24,
          }}
        >
          {tag}
        </div>
      )}
      {lines.map((line, i) => (
        <div
          key={i}
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize,
            fontWeight: 900,
            color,
            letterSpacing: -2,
            lineHeight: 1.05,
            textTransform: "uppercase",
            textAlign: "center",
            textShadow: `0 4px 40px rgba(0,0,0,0.8)`,
          }}
        >
          {line}
        </div>
      ))}
    </div>
  );
}
