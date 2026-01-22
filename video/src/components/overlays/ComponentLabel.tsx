import { MUNI_ORANGE, FONT_MONO, WHITE } from "../../lib/brand";
import type { ComponentInfo } from "../../lib/bvr1-components";

interface ComponentLabelProps {
  component: ComponentInfo;
  opacity?: number;
  // Screen position (calculated from 3D projection)
  x: number;
  y: number;
}

export function ComponentLabel({
  component,
  opacity = 1,
  x,
  y,
}: ComponentLabelProps) {
  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y,
        transform: "translate(-50%, -100%)",
        opacity,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 8,
      }}
    >
      {/* Label card */}
      <div
        style={{
          background: "rgba(0, 0, 0, 0.85)",
          border: `2px solid ${MUNI_ORANGE}`,
          borderRadius: 8,
          padding: "12px 20px",
          fontFamily: FONT_MONO,
          color: WHITE,
          textAlign: "center",
          minWidth: 180,
        }}
      >
        <div
          style={{
            fontSize: 18,
            fontWeight: 700,
            color: MUNI_ORANGE,
            marginBottom: 4,
          }}
        >
          {component.name}
        </div>
        <div style={{ fontSize: 13, opacity: 0.8, marginBottom: 4 }}>
          {component.desc}
        </div>
        <div style={{ fontSize: 12, opacity: 0.6 }}>{component.specs}</div>
      </div>

      {/* Connector line */}
      <div
        style={{
          width: 2,
          height: 40,
          background: MUNI_ORANGE,
        }}
      />

      {/* Anchor dot */}
      <div
        style={{
          width: 12,
          height: 12,
          borderRadius: "50%",
          background: MUNI_ORANGE,
          border: `2px solid ${WHITE}`,
        }}
      />
    </div>
  );
}
