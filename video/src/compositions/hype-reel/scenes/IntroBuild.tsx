// Scene 1: Problem montage + stats (0:00–0:12)
import { Sequence, useCurrentFrame, Img, staticFile } from "remotion";
import { interpolate } from "remotion";
import { REEL_CONFIG } from "../config";
import { getCutsInRange, getCutDuration } from "../beat";
import { getShot } from "../shots";
import { ShotFrame } from "../../../components/overlays/ShotFrame";
import { HypeTextCard } from "../../../components/overlays/HypeTextCard";
import { WHITE, DARK_BG } from "../../../lib/brand";

const section = REEL_CONFIG.sections.introBuild;

/** Rapid-fire problem montage — 3 unique images, no pan/zoom, fills until stat-cities */
const RAPID_CUTS = [
  { src: "reel/shots/snow-problem-1.jpg", frames: 25 },
  { src: "reel/shots/snow-problem-2.jpg", frames: 25 },
  { src: "reel/shots/snow-problem-3.jpg", frames: 24 },
]; // 74 frames total = beat 4 (where stat-cities starts)

export function IntroBuild() {
  const frame = useCurrentFrame();
  const cuts = getCutsInRange(section.start, section.end);

  return (
    <>
      {/* Ultra-fast problem montage — first 2 seconds */}
      {RAPID_CUTS.reduce<{ elements: React.ReactNode[]; offset: number }>(
        (acc, cut, i) => {
          acc.elements.push(
            <Sequence key={`rapid-${i}`} from={acc.offset} durationInFrames={cut.frames}>
              <div style={{ position: "absolute", inset: 0, overflow: "hidden", backgroundColor: "#000" }}>
                <Img
                  src={staticFile(cut.src)}
                  style={{
                    width: "100%",
                    height: "100%",
                    objectFit: "contain",
                  }}
                />
              </div>
            </Sequence>,
          );
          acc.offset += cut.frames;
          return acc;
        },
        { elements: [], offset: 0 },
      ).elements}

      {cuts.map((cut, idx) => {
        const localFrom = cut.frame - section.start;
        const duration = getCutDuration(cut, section.end);
        const shot = getShot(cut.label);

        // Skip snow-problem-1 — replaced by rapid cuts above
        if (cut.label === "snow-problem-1") return null;

        // Extend dirty-sidewalk image behind fuck-snow text
        if (cut.label === "dirty-sidewalk") {
          const nextCut = cuts[idx + 1];
          const extendedDuration = nextCut
            ? (nextCut.frame - section.start) + getCutDuration(nextCut, section.end) - localFrom
            : duration;
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={extendedDuration}>
              <ShotFrame shot={shot} durationInFrames={extendedDuration} />
            </Sequence>
          );
        }

        // fuck-snow: semi-transparent overlay so dirty sidewalk shows through
        if (cut.label === "fuck-snow") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <HypeTextCard
                text={shot.textOverlay!}
                bgColor="rgba(10, 10, 10, 0.65)"
              />
            </Sequence>
          );
        }

        return (
          <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
            {/* Render shot frame (image/video/placeholder) */}
            <ShotFrame shot={shot} durationInFrames={duration} />

            {/* Text overlay if present */}
            {shot.textOverlay && (
              <HypeTextCard
                text={shot.textOverlay}
                tag={shot.tag}
                bgColor={shot.bgColor}
              />
            )}

            {/* Logo flash */}
            {cut.label === "muni-logo-flash" && (
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
                <Img
                  src={staticFile("images/muni-logo-light.svg")}
                  style={{ width: 400, opacity: 0.9 }}
                />
              </div>
            )}
          </Sequence>
        );
      })}

    </>
  );
}
