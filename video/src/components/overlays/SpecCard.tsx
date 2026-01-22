import { MUNI_ORANGE, FONT_MONO, WHITE, DARK_BG } from "../../lib/brand";

interface SpecItem {
  label: string;
  value: string;
}

interface SpecCardProps {
  title: string;
  specs: SpecItem[];
  opacity?: number;
  position?: "left" | "right" | "center";
}

export function SpecCard({
  title,
  specs,
  opacity = 1,
  position = "right",
}: SpecCardProps) {
  const positionStyles: Record<string, React.CSSProperties> = {
    left: { left: 60, top: "50%", transform: "translateY(-50%)" },
    right: { right: 60, top: "50%", transform: "translateY(-50%)" },
    center: { left: "50%", top: "50%", transform: "translate(-50%, -50%)" },
  };

  return (
    <div
      style={{
        position: "absolute",
        ...positionStyles[position],
        opacity,
        background: `linear-gradient(135deg, ${DARK_BG} 0%, rgba(26, 26, 26, 0.95) 100%)`,
        border: `2px solid ${MUNI_ORANGE}`,
        borderRadius: 16,
        padding: "32px 40px",
        fontFamily: FONT_MONO,
        color: WHITE,
        minWidth: 320,
        backdropFilter: "blur(10px)",
      }}
    >
      <div
        style={{
          fontSize: 28,
          fontWeight: 700,
          color: MUNI_ORANGE,
          marginBottom: 24,
          textTransform: "uppercase",
          letterSpacing: 2,
        }}
      >
        {title}
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
        {specs.map((spec, i) => (
          <div key={i} style={{ display: "flex", justifyContent: "space-between", gap: 40 }}>
            <span style={{ fontSize: 16, opacity: 0.7 }}>{spec.label}</span>
            <span style={{ fontSize: 16, fontWeight: 700 }}>{spec.value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
