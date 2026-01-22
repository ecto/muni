import { Composition, Folder } from "remotion";
import { BVR1Showcase } from "./compositions/bvr1-showcase";
import { SHOWCASE_CONFIG } from "./compositions/bvr1-showcase/config";
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

      {/* Future video compositions */}
      {/* <Folder name="Marketing">
        <Composition id="InvestorPitch" ... />
        <Composition id="TechExplainer" ... />
      </Folder> */}
      {/* <Folder name="Social">
        <Composition id="SocialClips" ... />
      </Folder> */}
    </>
  );
};
