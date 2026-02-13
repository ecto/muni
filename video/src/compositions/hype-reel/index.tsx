// Hype Reel — main composition
// 70-80 second hype reel for f.inc Artifact Demo Day
// Synced to A$AP Rocky's HELICOPTER at 140 BPM
import { AbsoluteFill, Sequence, Audio, staticFile, continueRender, delayRender, getStaticFiles } from "remotion";
import { useEffect, useState } from "react";
import { REEL_CONFIG } from "./config";
import { IntroBuild } from "./scenes/IntroBuild";
import { TheDropReveal } from "./scenes/TheDropReveal";
import { TheFlex } from "./scenes/TheFlex";
import { TheTech } from "./scenes/TheTech";
import { TheBusiness } from "./scenes/TheBusiness";
import { TheClose } from "./scenes/TheClose";
import { DARK_BG } from "../../lib/brand";

const { sections } = REEL_CONFIG;

function sectionDuration(s: { start: number; end: number }) {
  return s.end - s.start;
}

/** Load Inter Black font face before rendering */
function useInterFont() {
  const [handle] = useState(() => delayRender("Loading Inter font"));

  useEffect(() => {
    const font = new FontFace(
      "Inter",
      `url(${staticFile("reel/fonts/Inter-Black.woff2")})`,
      { weight: "900", style: "normal" }
    );
    font
      .load()
      .then((loaded) => {
        document.fonts.add(loaded);
        continueRender(handle);
      })
      .catch(() => {
        // Font not available — fall back to Helvetica
        continueRender(handle);
      });
  }, [handle]);
}

export const HypeReel: React.FC = () => {
  useInterFont();

  // Audio is optional — gitignored copyrighted music
  const hasAudio = getStaticFiles().some((f) => f.name === "reel/audio/helicopter.mp3");

  return (
    <AbsoluteFill style={{ backgroundColor: DARK_BG }}>
      {/* Audio track (optional — gitignored) */}
      {hasAudio && <Audio src={staticFile("reel/audio/helicopter.mp3")} volume={1} />}

      {/* Scene 1: Problem montage + stats */}
      <Sequence from={sections.introBuild.start} durationInFrames={sectionDuration(sections.introBuild)}>
        <IntroBuild />
      </Sequence>

      {/* Scene 2: Beat drop, BVR1 reveal, "no gas" stats */}
      <Sequence from={sections.theDropReveal.start} durationInFrames={sectionDuration(sections.theDropReveal)}>
        <TheDropReveal />
      </Sequence>

      {/* Scene 3: Use case fast cuts */}
      <Sequence from={sections.theFlex.start} durationInFrames={sectionDuration(sections.theFlex)}>
        <TheFlex />
      </Sequence>

      {/* Scene 4: Hardware + autonomy stack */}
      <Sequence from={sections.theTech.start} durationInFrames={sectionDuration(sections.theTech)}>
        <TheTech />
      </Sequence>

      {/* Scene 5: Scale + business model */}
      <Sequence from={sections.theBusiness.start} durationInFrames={sectionDuration(sections.theBusiness)}>
        <TheBusiness />
      </Sequence>

      {/* Scene 6: Checklist, founder, CTA */}
      <Sequence from={sections.theClose.start} durationInFrames={sectionDuration(sections.theClose)}>
        <TheClose />
      </Sequence>
    </AbsoluteFill>
  );
};
