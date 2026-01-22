import { Img, staticFile } from "remotion";

interface LogoProps {
  opacity?: number;
  scale?: number;
  position?: "center" | "bottom-right" | "top-left";
}

export function Logo({
  opacity = 1,
  scale = 1,
  position = "center",
}: LogoProps) {
  const baseSize = 200;
  const size = baseSize * scale;

  const positionStyles: Record<string, React.CSSProperties> = {
    center: {
      left: "50%",
      top: "50%",
      transform: `translate(-50%, -50%) scale(${scale})`,
    },
    "bottom-right": {
      right: 40,
      bottom: 40,
      transform: `scale(${scale})`,
      transformOrigin: "bottom right",
    },
    "top-left": {
      left: 40,
      top: 40,
      transform: `scale(${scale})`,
      transformOrigin: "top left",
    },
  };

  return (
    <div
      style={{
        position: "absolute",
        ...positionStyles[position],
        opacity,
        width: size,
        height: size,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <Img
        src={staticFile("images/muni-logo-light.svg")}
        style={{
          width: "100%",
          height: "100%",
          objectFit: "contain",
        }}
      />
    </div>
  );
}
