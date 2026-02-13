// Scene 5: Scale + business model (0:46–0:58)
import { Sequence } from "remotion";
import { REEL_CONFIG } from "../config";
import { getCutsInRange, getCutDuration } from "../beat";
import { getShot } from "../shots";
import { ShotFrame } from "../../../components/overlays/ShotFrame";
import { HypeTextCard } from "../../../components/overlays/HypeTextCard";
import { SnowBeltMap } from "../components/SnowBeltMap";

const CODE_RENDERED: Record<string, React.FC> = {
  "biz-tam-map": SnowBeltMap,
};

const section = REEL_CONFIG.sections.theBusiness;

export function TheBusiness() {
  const cuts = getCutsInRange(section.start, section.end);

  return (
    <>
      {cuts.map((cut) => {
        const localFrom = cut.frame - section.start;
        const duration = getCutDuration(cut, section.end);
        const shot = getShot(cut.label);

        // Code-rendered biz graphics
        const CodeComponent = CODE_RENDERED[cut.label];
        if (CodeComponent) {
          return (
            <Sequence key={cut.label} from={localFrom} durationInFrames={duration}>
              <CodeComponent />
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
