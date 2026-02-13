// Full-frame image/video/placeholder with Ken Burns + color grade
import { useCurrentFrame, Img, staticFile, Video } from "remotion";
import { interpolate } from "remotion";
import type { Shot } from "../../compositions/hype-reel/shots";
import { REEL_CONFIG } from "../../compositions/hype-reel/config";
import { DARK_BG, MUNI_ORANGE, FONT_MONO, WHITE } from "../../lib/brand";

const FONT_DISPLAY = '"Inter", "Helvetica Neue", Arial, sans-serif';

interface ShotFrameProps {
  shot: Shot;
  durationInFrames: number;
}

export function ShotFrame({ shot, durationInFrames }: ShotFrameProps) {
  const frame = useCurrentFrame();

  // Ken Burns animation
  const kb = shot.kenBurns;
  const scale = kb
    ? interpolate(frame, [0, durationInFrames], [kb.startScale, kb.endScale], {
        extrapolateRight: "clamp",
      })
    : 1;
  const translateX = kb
    ? interpolate(frame, [0, durationInFrames], [kb.startX, kb.endX], {
        extrapolateRight: "clamp",
      })
    : 0;
  const translateY = kb
    ? interpolate(frame, [0, durationInFrames], [kb.startY, kb.endY], {
        extrapolateRight: "clamp",
      })
    : 0;

  // Placeholder frame when no src
  if (!shot.src && shot.type !== "text" && shot.type !== "black") {
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
          [{shot.type}]
        </div>
        <div
          style={{
            fontFamily: FONT_DISPLAY,
            fontSize: 28,
            color: WHITE,
            opacity: 0.5,
            textAlign: "center",
            maxWidth: 800,
            lineHeight: 1.4,
          }}
        >
          {shot.description}
        </div>
        {shot.tag && (
          <div
            style={{
              fontFamily: FONT_MONO,
              fontSize: 14,
              color: MUNI_ORANGE,
              opacity: 0.4,
              marginTop: 16,
              letterSpacing: 6,
            }}
          >
            {shot.tag}
          </div>
        )}
      </div>
    );
  }

  // Black frame
  if (shot.type === "black") {
    return (
      <div style={{ position: "absolute", inset: 0, backgroundColor: "#000" }} />
    );
  }

  // Text-only shot (handled by HypeTextCard in parent — this is fallback)
  if (shot.type === "text" && !shot.src) {
    return null; // parent renders HypeTextCard
  }

  // Image shot
  if (shot.type === "image" && shot.src) {
    return (
      <div style={{ position: "absolute", inset: 0, overflow: "hidden", backgroundColor: "#000" }}>
        <Img
          src={staticFile(shot.src)}
          style={{
            width: "100%",
            height: "100%",
            objectFit: "contain",
            transform: `scale(${scale}) translate(${translateX}%, ${translateY}%)`,
          }}
        />
      </div>
    );
  }

  // Video shot
  if (shot.type === "video" && shot.src) {
    return (
      <div style={{ position: "absolute", inset: 0, overflow: "hidden", backgroundColor: "#000" }}>
        <Video
          src={staticFile(shot.src)}
          style={{
            width: "100%",
            height: "100%",
            objectFit: "contain",
            transform: `scale(${scale}) translate(${translateX}%, ${translateY}%)`,
          }}
        />
      </div>
    );
  }

  // Fallback for unhandled types (3d, splat handled by parent scene)
  return null;
}
