import { AbsoluteFill, Sequence } from "remotion";
import { ThreeCanvas } from "@remotion/three";
import { SHOWCASE_CONFIG } from "./config";
import { HeroReveal } from "./scenes/HeroReveal";
import { OrbitalRotation } from "./scenes/OrbitalRotation";
import { ScaleComparison } from "./scenes/ScaleComparison";
import { ComponentCallouts } from "./scenes/ComponentCallouts";
import { SpecsClose } from "./scenes/SpecsClose";
import { DARK_BG } from "../../lib/brand";

export const BVR1Showcase: React.FC = () => {
  const { scenes, width, height } = SHOWCASE_CONFIG;

  return (
    <AbsoluteFill style={{ backgroundColor: DARK_BG }}>
      {/* 3D Canvas - spans all scenes */}
      <ThreeCanvas
        width={width}
        height={height}
        camera={{
          fov: SHOWCASE_CONFIG.camera.fov,
          near: 0.1,
          far: 5000,
          position: [600, 400, 600],
        }}
        style={{ position: "absolute", top: 0, left: 0 }}
      >
        {/* Scene 1: Hero Reveal */}
        <Sequence
          from={scenes.heroReveal.start}
          durationInFrames={scenes.heroReveal.end - scenes.heroReveal.start}
          layout="none"
        >
          <HeroReveal />
        </Sequence>

        {/* Scene 2: Orbital Rotation */}
        <Sequence
          from={scenes.orbitalRotation.start}
          durationInFrames={scenes.orbitalRotation.end - scenes.orbitalRotation.start}
          layout="none"
        >
          <OrbitalRotation />
        </Sequence>

        {/* Scene 3: Scale Comparison */}
        <Sequence
          from={scenes.scaleComparison.start}
          durationInFrames={scenes.scaleComparison.end - scenes.scaleComparison.start}
          layout="none"
        >
          <ScaleComparison />
        </Sequence>

        {/* Scene 4: Component Callouts */}
        <Sequence
          from={scenes.componentCallouts.start}
          durationInFrames={scenes.componentCallouts.end - scenes.componentCallouts.start}
          layout="none"
        >
          <ComponentCallouts />
        </Sequence>

        {/* Scene 5: Specs & Close */}
        <Sequence
          from={scenes.specsClose.start}
          durationInFrames={scenes.specsClose.end - scenes.specsClose.start}
          layout="none"
        >
          <SpecsClose />
        </Sequence>
      </ThreeCanvas>

      {/* 2D Overlays - handled within each scene as HTML overlays */}
    </AbsoluteFill>
  );
};
