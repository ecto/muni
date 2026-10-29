// Deterministic capture for social cards (see scripts/build-og.mjs). In og mode a
// cover draws nothing on its own: the capture script calls window.__ogRender(t)
// and the cover steps its simulation at 30 fps from 0 to t, so frame t is the
// same on every run.
export const OG_FPS = 30;

declare global {
  interface Window {
    __ogRender?: (t: number) => void;
    __ogReady?: boolean;
  }
}

/** a seeded PRNG (mulberry32), standing in for Math.random in og mode */
export function seeded(seed = 1) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** expose window.__ogRender: reset() rewinds the cover to t = 0, step(ms) draws one frame at start + ms */
export function installOgClock(reset: () => void, step: (ms: number) => void) {
  let frame = -1;
  window.__ogRender = (t: number) => {
    const target = Math.round(t * OG_FPS);
    if (target < frame) frame = -1;
    if (frame < 0) reset();
    while (frame < target) {
      frame++;
      step((frame * 1000) / OG_FPS);
    }
  };
  window.__ogReady = true;
  return () => {
    delete window.__ogRender;
    delete window.__ogReady;
  };
}
