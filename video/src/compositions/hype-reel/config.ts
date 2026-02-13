// Hype Reel — timing config, section boundaries, grade constants

export const REEL_CONFIG = {
  fps: 30,
  width: 1920,
  height: 1080,
  totalFrames: 1440, // ~48 seconds

  // Audio — verified via librosa beat detection
  bpm: 134,
  firstBeatOffsetFrames: 20, // first beat at 0.651s ≈ frame 20

  // Color grading
  grade: {
    tealShadows: "rgba(0, 180, 180, 0.15)",
    orangeHighlights: "rgba(255, 102, 0, 0.1)",
    vignette: "radial-gradient(ellipse at center, transparent 50%, rgba(0,0,0,0.7) 100%)",
  },

  // Section boundaries (in frames, from beatToFrame at 134 BPM + 20f offset)
  sections: {
    introBuild: { start: 0, end: 396 },        // 0:00–0:13  (beats 0–27)
    theDropReveal: { start: 396, end: 584 },    // 0:13–0:19  (beats 28–41)
    theFlex: { start: 584, end: 745 },          // 0:19–0:25  (beats 42–53)
    theTech: { start: 745, end: 907 },          // 0:25–0:30  (beats 54–65)
    theBusiness: { start: 907, end: 1041 },     // 0:30–0:35  (beats 66–75)
    theClose: { start: 1041, end: 1440 },       // 0:35–0:48  (beats 76–104)
  },
} as const;

export type SectionName = keyof typeof REEL_CONFIG.sections;
