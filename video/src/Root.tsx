import { Composition, Folder } from "remotion";
import { BVR1Showcase } from "./compositions/bvr1-showcase";
import { SHOWCASE_CONFIG } from "./compositions/bvr1-showcase/config";
import { HypeReel } from "./compositions/hype-reel";
import { REEL_CONFIG } from "./compositions/hype-reel/config";
import "./styles/global.css";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Folder name="BVR1">
        <Composition
          id="BVR1Showcase"
          component={BVR1Showcase}
          durationInFrames={SHOWCASE_CONFIG.totalFrames}
          fps={SHOWCASE_CONFIG.fps}
          width={SHOWCASE_CONFIG.width}
          height={SHOWCASE_CONFIG.height}
          defaultProps={{}}
        />
      </Folder>

      <Folder name="Marketing">
        <Composition
          id="HypeReel"
          component={HypeReel}
          durationInFrames={REEL_CONFIG.totalFrames}
          fps={REEL_CONFIG.fps}
          width={REEL_CONFIG.width}
          height={REEL_CONFIG.height}
          defaultProps={{}}
        />
      </Folder>
    </>
  );
};
