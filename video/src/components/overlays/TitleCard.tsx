import { MUNI_ORANGE, FONT_MONO, WHITE } from "../../lib/brand";

interface TitleCardProps {
  title: string;
  subtitle?: string;
  opacity?: number;
  position?: "center" | "bottom" | "top";
}

export function TitleCard({
  title,
  subtitle,
  opacity = 1,
  position = "center",
}: TitleCardProps) {
  const positionStyles: Record<string, React.CSSProperties> = {
    center: {
      left: "50%",
      top: "50%",
      transform: "translate(-50%, -50%)",
    },
    bottom: {
      left: "50%",
      bottom: 100,
      transform: "translateX(-50%)",
    },
    top: {
      left: "50%",
      top: 100,
      transform: "translateX(-50%)",
    },
  };

  return (
    <div
      style={{
        position: "absolute",
        ...positionStyles[position],
        opacity,
        fontFamily: FONT_MONO,
        textAlign: "center",
      }}
    >
      <div
        style={{
          fontSize: 72,
          fontWeight: 700,
          color: WHITE,
          letterSpacing: -2,
          textShadow: `0 4px 20px rgba(0, 0, 0, 0.8), 0 0 40px ${MUNI_ORANGE}40`,
        }}
      >
        {title}
      </div>
      {subtitle && (
        <div
          style={{
            fontSize: 28,
            color: MUNI_ORANGE,
            marginTop: 16,
            letterSpacing: 4,
            textTransform: "uppercase",
          }}
        >
          {subtitle}
        </div>
      )}
    </div>
  );
}
