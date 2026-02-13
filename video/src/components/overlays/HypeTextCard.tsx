// Full-screen text card — Helvetica Neue, all caps, hard cuts
import { WHITE, MUNI_ORANGE, DARK_BG, FONT_DISPLAY } from "../../lib/brand";

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
