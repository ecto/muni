// Scene 4: Hardware + autonomy stack
import { Sequence } from "remotion";
import { REEL_CONFIG } from "../config";
import { getCutsInRange, getCutDuration } from "../beat";
import { getShot } from "../shots";
import { ShotFrame } from "../../../components/overlays/ShotFrame";
import { HypeTextCard } from "../../../components/overlays/HypeTextCard";
import { BVR1LidarSafety } from "../components/BVR1HeroShot";

const section = REEL_CONFIG.sections.theTech;

export function TheTech() {
  const cuts = getCutsInRange(section.start, section.end);

  return (
    <>
      {cuts.map((cut) => {
        const localFrom = cut.frame - section.start;
        const duration = getCutDuration(cut, section.end);
        const shot = getShot(cut.label);

        if (cut.label === "tech-lidar") {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <BVR1LidarSafety durationInFrames={duration} />
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
