// Error boundary for 3D model loading failures
// Falls back to a placeholder when GLB files can't be loaded
import React from "react";
import { MUNI_ORANGE, FONT_MONO, WHITE } from "../../lib/brand";

interface Props {
  children: React.ReactNode;
  label?: string;
}

interface State {
  hasError: boolean;
}

export class ModelErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            position: "absolute",
            inset: 0,
            backgroundColor: "#111",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            border: `4px dashed ${MUNI_ORANGE}40`,
          }}
        >
          <div
            style={{
              fontFamily: FONT_MONO,
              fontSize: 18,
              color: MUNI_ORANGE,
              opacity: 0.6,
              letterSpacing: 4,
              textTransform: "uppercase",
              marginBottom: 16,
            }}
          >
            [3D MODEL]
          </div>
          <div
            style={{
              fontFamily: '"Inter", "Helvetica Neue", Arial, sans-serif',
              fontSize: 28,
              color: WHITE,
              opacity: 0.5,
              textAlign: "center",
            }}
          >
            {this.props.label ?? "3D model unavailable"}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
