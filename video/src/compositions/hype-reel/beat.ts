// Beat system — BPM to frame conversion + master cut point list
import { REEL_CONFIG } from "./config";

const { bpm, fps, firstBeatOffsetFrames } = REEL_CONFIG;

/** Frames per beat at current BPM/FPS */
export const FRAMES_PER_BEAT = (60 / bpm) * fps; // ~12.857 at 140bpm/30fps

/** Convert a beat number (0-indexed) to a frame number */
export function beatToFrame(beat: number): number {
  return Math.round(firstBeatOffsetFrames + beat * FRAMES_PER_BEAT);
}

/** Convert frame to nearest beat */
export function frameToBeat(frame: number): number {
  return (frame - firstBeatOffsetFrames) / FRAMES_PER_BEAT;
}

export interface CutPoint {
  beat: number;
  frame: number;
  label: string;
  section: string;
}

// Master cut list — every visual cut in the video, indexed by beat
// Labels match shot registry keys in shots.ts
const RAW_CUTS: Array<{ beat: number; label: string; section: string }> = [
  // === INTRO BUILD (beats 0–27) === 11 shots
  { beat: 0, label: "snow-problem-1", section: "introBuild" },   // rapid cuts override
  { beat: 4, label: "stat-cities", section: "introBuild" },
  { beat: 6, label: "stat-injuries", section: "introBuild" },
  { beat: 8, label: "dirty-sidewalk", section: "introBuild" },
  { beat: 10, label: "fuck-snow", section: "introBuild" },       // 4 beats, dirty sidewalk behind
  { beat: 14, label: "what-if", section: "introBuild" },
  { beat: 18, label: "text-autonomous", section: "introBuild" },
  { beat: 20, label: "text-electric", section: "introBuild" },
  { beat: 22, label: "text-open-source", section: "introBuild" },
  { beat: 24, label: "text-sidewalk-roombas", section: "introBuild" },

  // === THE DROP / REVEAL (beats 28–41) === 9 shots
  { beat: 28, label: "bvr1-hero-front", section: "theDropReveal" },
  { beat: 31, label: "bvr1-hero-side", section: "theDropReveal" },
  { beat: 32, label: "bvr1-hero-rear", section: "theDropReveal" },
  { beat: 33, label: "no-gas-48v", section: "theDropReveal" },
  { beat: 34, label: "no-gas-vesc", section: "theDropReveal" },
  { beat: 35, label: "no-gas-12kw", section: "theDropReveal" },
  { beat: 36, label: "no-gas-zero", section: "theDropReveal" },
  { beat: 37, label: "bvr1-3d-spin", section: "theDropReveal" },
  { beat: 40, label: "stat-cost", section: "theDropReveal" },

  // === THE FLEX (beats 42–53) === 6 shots
  { beat: 42, label: "flex-snow-action-1", section: "theFlex" },
  { beat: 44, label: "flex-debris-action", section: "theFlex" },
  { beat: 46, label: "flex-snow-action-2", section: "theFlex" },
  { beat: 48, label: "flex-text-24-7", section: "theFlex" },
  { beat: 50, label: "flex-text-no-driver", section: "theFlex" },
  { beat: 52, label: "flex-text-one-operator", section: "theFlex" },

  // === THE TECH (beats 54–65) === 6 shots
  { beat: 54, label: "tech-text-orin", section: "theTech" },
  { beat: 56, label: "tech-lidar", section: "theTech" },
  { beat: 58, label: "tech-text-rtk", section: "theTech" },
  { beat: 60, label: "tech-console-ui", section: "theTech" },
  { beat: 62, label: "tech-text-teleop", section: "theTech" },
  { beat: 64, label: "tech-text-rust", section: "theTech" },

  // === THE BUSINESS (beats 66–75) === 5 shots
  { beat: 66, label: "biz-tam-map", section: "theBusiness" },
  { beat: 68, label: "biz-text-32b", section: "theBusiness" },
  { beat: 70, label: "biz-text-raas", section: "theBusiness" },
  { beat: 72, label: "biz-text-margins", section: "theBusiness" },
  { beat: 74, label: "biz-text-community", section: "theBusiness" },

  // === THE CLOSE (beats 76–104) === 7 shots
  { beat: 76, label: "close-checklist", section: "theClose" },
  { beat: 84, label: "close-founder", section: "theClose" },
  { beat: 88, label: "pink-bvr1", section: "theClose" },       // 0:40 — "pussy whip"
  { beat: 90, label: "close-bvr1-beauty", section: "theClose" },
  { beat: 94, label: "close-cta", section: "theClose" },
  { beat: 98, label: "close-url", section: "theClose" },
  { beat: 100, label: "close-event", section: "theClose" },
];

/** All cut points with computed frame numbers */
export const CUT_POINTS: CutPoint[] = RAW_CUTS.map((c) => ({
  ...c,
  frame: beatToFrame(c.beat),
}));

/** Get all cuts within a frame range (inclusive start, exclusive end) */
export function getCutsInRange(startFrame: number, endFrame: number): CutPoint[] {
  return CUT_POINTS.filter((c) => c.frame >= startFrame && c.frame < endFrame);
}

/** Get duration in frames from one cut to the next (or to endFrame) */
export function getCutDuration(cut: CutPoint, endFrame: number): number {
  const idx = CUT_POINTS.indexOf(cut);
  const next = CUT_POINTS[idx + 1];
  return (next ? Math.min(next.frame, endFrame) : endFrame) - cut.frame;
}
