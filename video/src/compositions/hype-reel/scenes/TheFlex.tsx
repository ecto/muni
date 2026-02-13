// Scene 3: Use case fast cuts (0:22–0:36)
import { Sequence } from "remotion";
import { REEL_CONFIG } from "../config";
import { getCutsInRange, getCutDuration } from "../beat";
import { getShot } from "../shots";
import { ShotFrame } from "../../../components/overlays/ShotFrame";
import { HypeTextCard } from "../../../components/overlays/HypeTextCard";
import { BVR1SnowAction } from "../components/BVR1HeroShot";

const section = REEL_CONFIG.sections.theFlex;

export function TheFlex() {
  const cuts = getCutsInRange(section.start, section.end);

  return (
    <>
      {cuts.map((cut) => {
        const localFrom = cut.frame - section.start;
        const duration = getCutDuration(cut, section.end);
        const shot = getShot(cut.label);

        // 3D BVR1 action shots replacing video placeholders
        if (cut.label === "flex-snow-action-1") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BVR1SnowAction durationInFrames={duration} variant={1} />
            </Sequence>
          );
        }
        if (cut.label === "flex-snow-action-2") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BVR1SnowAction durationInFrames={duration} variant={2} />
            </Sequence>
          );
        }

        return (
          <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
            <ShotFrame shot={shot} durationInFrames={duration} />
            {shot.textOverlay && (
              <HypeTextCard
                text={shot.textOverlay}
                tag={shot.tag}
                bgColor={shot.bgColor}
              />
            )}
          </Sequence>
        );
      })}

    </>
  );
}
