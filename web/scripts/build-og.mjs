// Social preview cards: renders public/og/** from the hidden /og-capture/<slug> route.
//
//   npm run dev            # (or any server for this tree) on :3030
//   npm run og             # all cards; OG_BASE=http://localhost:3000 npm run og to point elsewhere
//   npm run og -- vision   # just the named cards
//
// Each card is a cover in og mode (components/blog/covers/ogClock.ts): it draws only
// when we call window.__ogRender(t), stepping its simulation at 30 fps from t = 0,
// so every frame is deterministic. Stills are one frame; videos are FRAMES frames
// plus a FADE-frame crossfade of the tail into the head, so the loop closes. Needs
// Google Chrome (CHROME_PATH to override) and ffmpeg with libx264 (FFMPEG). The
// outputs are committed: the site is a static export and nothing renders at deploy.
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer-core";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const BASE = process.env.OG_BASE || "http://localhost:3030";
const CHROME = process.env.CHROME_PATH || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const FFMPEG = process.env.FFMPEG || "ffmpeg";
const FPS = 30;
const FADE = 24; // frames of crossfade closing each loop

// still: the frame (seconds of cover time) for the PNG. video: start and length of the loop.
const CARDS = [
  // all five title rows written and the readback held; the loop shows each write land
  { slug: "zero-context-memory", out: "og/research/zero-context-memory", still: 9.5, video: { from: 0.4, seconds: 8 } },
  // the probe mid-flight to the Mars flyby, its callout live; the loop is one launch
  { slug: "vision", out: "og/research/vision", still: 4.2, video: { from: 0.2, seconds: 8 } },
  // mid-write: the key walking down to its row
  { slug: "research", out: "og/research", still: 5.0 },
  { slug: "site", out: "og/site", still: 4.2 },
];

const only = process.argv.slice(2);
const cards = only.length ? CARDS.filter((c) => only.includes(c.slug)) : CARDS;

const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--hide-scrollbars", "--force-color-profile=srgb"] });
try {
  for (const card of cards) {
    const page = await browser.newPage();
    await page.setViewport({ width: 1200, height: 630, deviceScaleFactor: 1 });
    // keep analytics out of the capture
    await page.setRequestInterception(true);
    page.on("request", (r) => (/posthog/.test(r.url()) ? r.abort() : r.continue()));
    await page.goto(`${BASE}/og-capture/${card.slug}`, { waitUntil: "networkidle0" });
    await page.waitForFunction("window.__ogReady === true && document.fonts.status === 'loaded'");
    await page.evaluate(() => document.fonts.ready);
    const shot = async (t) => {
      await page.evaluate((t) => window.__ogRender(t), t);
      return page.screenshot({ type: "png" });
    };

    const png = join(ROOT, "public", `${card.out}.png`);
    mkdirSync(dirname(png), { recursive: true });
    writeFileSync(png, await shot(card.still));
    console.log(`${card.out}.png  ${(statSync(png).size / 1024).toFixed(0)} KB`);

    if (card.video) {
      const n = Math.round(card.video.seconds * FPS);
      const dir = mkdtempSync(join(tmpdir(), "muni-og-"));
      try {
        // raw frames 0 .. n + FADE - 1; the loop plays raw[FADE .. n + FADE - 1] and
        // blends its last FADE frames toward raw[0 .. FADE - 1], which lead into raw[FADE]
        const raw = (i) => join(dir, `raw-${String(i).padStart(4, "0")}.png`);
        const outf = (j) => join(dir, `out-${String(j).padStart(4, "0")}.png`);
        for (let i = 0; i < n + FADE; i++) writeFileSync(raw(i), await shot(card.video.from + i / FPS));
        for (let j = 0; j < n; j++) {
          const i = j - (n - FADE);
          if (i < 0) {
            execFileSync("cp", [raw(FADE + j), outf(j)]);
            continue;
          }
          const a = ((i + 1) / (FADE + 1)).toFixed(4);
          execFileSync(FFMPEG, ["-v", "error", "-y", "-i", raw(n + i), "-i", raw(i), "-filter_complex", `blend=all_expr='A*(1-${a})+B*${a}'`, outf(j)]);
        }
        const mp4 = join(ROOT, "public", `${card.out}.mp4`);
        execFileSync(FFMPEG, [
          "-v", "error", "-y", "-framerate", String(FPS), "-i", join(dir, "out-%04d.png"),
          "-c:v", "libx264", "-preset", "slow", "-crf", process.env.OG_CRF || "26", "-tune", "animation",
          "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-an", mp4,
        ]);
        console.log(`${card.out}.mp4  ${(statSync(mp4).size / 1024).toFixed(0)} KB`);
      } finally {
        rmSync(dir, { recursive: true, force: true });
      }
    }
    await page.close();
  }
} finally {
  await browser.close();
}
