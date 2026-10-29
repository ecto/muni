"use client";

import { useEffect, useRef } from "react";

// The zero-context-memory cover, live: one Gated DeltaNet head's state matrix in
// a tilted plan view (Fig 1's delta grid, set in perspective). The field is a
// trained head forgetting: every cell decays by g < 1 each step and is re-excited
// at random, so it breathes and smears. A one-hot key walks down the left edge;
// where it lands, a row is written and pinned (g = 1, β = 1) and then holds its
// value exactly while everything around it fades. The pinned rows are the post's
// title, stored as token ids, one id per row, its bits as lit and unlit cells.
const C = {
  bg: "#0b0907",
  surface: "#15110d",
  line: "#2a2119",
  lineHi: "#4a3a2a",
  label: "#a08d70",
  text: "#e8e1d2",
  sel: "#ff5f00",
  ok: "#39ff14",
  info: "#00f0ff",
  fault: "#ff2052",
};

// "Gate-Pinned State Memory" in the Qwen3.5 tokenizer: produced on mew with
// transformers 5.22 AutoTokenizer.from_pretrained("~/gpsm-models/Qwen3.5-4B")
// .encode(title, add_special_tokens=False) → 'Gate' '-P' 'inned' ' State' ' Memory'.
const TITLE = "Gate-Pinned State Memory";
const IDS = [40912, 9023, 19936, 3130, 13453];
const BITS = 17; // every id < 2^17; a row repeats its id in 17-bit words, one dark cell between words
const WORD = BITS + 1;
const ROW_AT = [0.2, 0.34, 0.48, 0.62, 0.76]; // where each token lands, as a fraction of the rows

const T0 = 0.9; // first write
const T_STEP = 1.35; // between writes
const T_TRAVEL = 0.85; // the key's walk to its row
const T_FLASH = 0.45;
const T_DONE = T0 + T_STEP * (IDS.length - 1) + T_TRAVEL + T_FLASH;
const K = 1.7; // perspective: the far edge is 1 / (1 + K) the width of the near one
const LEVELS = 32;

function mix(a: string, b: string, t: number) {
  const p = (s: string, i: number) => parseInt(s.slice(1 + i * 2, 3 + i * 2), 16);
  const c = [0, 1, 2].map((i) => Math.round(p(a, i) + (p(b, i) - p(a, i)) * t));
  return `rgb(${c[0]},${c[1]},${c[2]})`;
}

// the field's palette, surface → line-hi → label, built once
const FIELD = Array.from({ length: LEVELS }, (_, i) => {
  const t = i / (LEVELS - 1);
  return t < 0.7 ? mix(C.surface, C.lineHi, t / 0.7) : mix(C.lineHi, C.label, (t - 0.7) / 0.3);
});

function smooth(t: number) {
  const k = Math.min(1, Math.max(0, t));
  return k * k * (3 - 2 * k);
}

export function PinnedRows() {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const mono = "'IBM Plex Mono', ui-monospace, monospace";

    // geometry and state, rebuilt only on resize
    let rows = 0;
    let cols = 0;
    let vals = new Float32Array(0);
    let rowY = new Float32Array(0);
    let rowS = new Float32Array(0);
    let rowH = new Float32Array(0);
    let colX = new Float32Array(0);
    let target = new Int16Array(0); // row index of each token
    const pinned = new Uint8Array(IDS.length);
    let pinOf = new Int8Array(0); // token pinned at each row, -1 if none
    let cx = 0;
    let halfW = 0;
    let cw = 0;
    let drift = 0.62; // the unpinned head's readout value

    const seed = (i: number) => {
      const s = Math.sin(i * 12.9898 + 78.233) * 43758.5453;
      return s - Math.floor(s);
    };

    function layout(w: number, h: number) {
      const narrow = w < 640;
      const yTop = h * (narrow ? 0.66 : 0.58);
      const yBot = h * 1.02;
      halfW = w * (narrow ? 0.47 : 0.36);
      cx = w * 0.5;
      cols = Math.max(36, Math.min(WORD * 8, Math.floor((2 * halfW) / 9)));
      rows = Math.max(40, Math.min(64, Math.floor((yBot - yTop) / 8)));
      vals = new Float32Array(rows * cols);
      for (let i = 0; i < vals.length; i++) vals[i] = Math.pow(seed(i), 3) * 0.7;
      rowY = new Float32Array(rows);
      rowS = new Float32Array(rows);
      rowH = new Float32Array(rows);
      const s0 = 1 / (1 + K);
      for (let r = 0; r < rows; r++) {
        const z = r / (rows - 1);
        const s = 1 / (1 + K * (1 - z));
        rowS[r] = s;
        rowY[r] = yTop + (yBot - yTop) * ((s - s0) / (1 - s0));
      }
      for (let r = 0; r < rows; r++) rowH[r] = (r < rows - 1 ? rowY[r + 1] - rowY[r] : rowY[r] - rowY[r - 1]) * 0.8;
      colX = new Float32Array(cols);
      for (let c = 0; c < cols; c++) colX[c] = -1 + (2 * (c + 0.5)) / cols;
      cw = ((2 * halfW) / cols) * 0.8;
      target = new Int16Array(IDS.length);
      pinOf = new Int8Array(rows).fill(-1);
      IDS.forEach((_, i) => (target[i] = Math.round(ROW_AT[i] * (rows - 1))));
      pinned.fill(0);
    }

    // the bit of token k shown at column c: MSB first, a dark spacer between words
    const bitAt = (k: number, c: number) => {
      const j = c % WORD;
      return j < BITS ? (IDS[k] >> (BITS - 1 - j)) & 1 : 0;
    };

    const pointer = { x: 0, y: 0, tx: 0, ty: 0 };
    const onMove = (e: PointerEvent) => {
      pointer.tx = (e.clientX / window.innerWidth) * 2 - 1;
      pointer.ty = (e.clientY / window.innerHeight) * 2 - 1;
    };
    window.addEventListener("pointermove", onMove);

    // scrolling the cover raises g toward 1; at the bottom the field stops fading
    let gate = 0;
    const onScroll = () => {
      gate = smooth(window.scrollY / Math.max(1, canvas.clientHeight * 0.85));
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });

    let raf = 0;
    let last = performance.now();
    const start = last;
    let onScreen = true;
    let tabOn = document.visibilityState === "visible";
    let running = false;

    // a readout in the clear margin beside the matrix, joined to its row by a
    // leader and a small tick at the row end (the solar cover's MSN-01 style).
    // Each line is a list of [text, color] runs. side -1: text left of the row,
    // right-aligned at tx; side 1: text right of the row, left-aligned at tx.
    type Run = [string, string];
    const FONT0 = `500 11px ${mono}`;
    const FONT1 = `400 10px ${mono}`;
    const width = (line: Run[], i: number) => {
      ctx.font = i === 0 ? FONT0 : FONT1;
      let m = 0;
      for (const [t] of line) m += ctx.measureText(t).width;
      return m;
    };
    const readout = (rx: number, ry: number, tx: number, ty: number, lines: Run[][], side: 1 | -1, lead: string, alpha = 1) => {
      // the tick at the row end, and the leader out to the text
      ctx.strokeStyle = lead;
      ctx.lineWidth = 1;
      ctx.globalAlpha = 0.9 * alpha;
      ctx.beginPath();
      ctx.moveTo(rx, ry - 4);
      ctx.lineTo(rx, ry + 4);
      ctx.stroke();
      ctx.globalAlpha = 0.5 * alpha;
      ctx.beginPath();
      if (ty === ry) {
        ctx.moveTo(rx + side * 4, ry);
        ctx.lineTo(tx - side * 6, ry);
      } else {
        // slot above the matrix: straight up from the row end to under the text
        ctx.moveTo(rx, ry - 6);
        ctx.lineTo(rx, ty + 8 + 14 * (lines.length - 1));
      }
      ctx.stroke();
      ctx.textAlign = "left";
      for (let i = 0; i < lines.length; i++) {
        const m = width(lines[i], i);
        let x = side > 0 ? tx : tx - m;
        const y = ty + 4 + i * 14;
        for (const [t, col] of lines[i]) {
          ctx.fillStyle = col;
          ctx.globalAlpha = (i === 0 ? 0.95 : 0.85) * alpha;
          ctx.fillText(t, x, y);
          x += ctx.measureText(t).width;
        }
      }
      ctx.globalAlpha = 1;
    };

    function draw(now: number) {
      if (!canvas || !ctx) return;
      const dpr = window.devicePixelRatio || 1;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;
      if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr) || rows === 0) {
        canvas.width = Math.round(w * dpr);
        canvas.height = Math.round(h * dpr);
        layout(w, h);
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.globalCompositeOperation = "source-over";
      ctx.fillStyle = C.bg;
      ctx.fillRect(0, 0, w, h);

      const dt = Math.min(0.05, (now - last) / 1000);
      last = now;
      const time = reduce ? T_DONE + 2 : (now - start) / 1000;
      pointer.x += (pointer.tx - pointer.x) * 0.04;
      pointer.y += (pointer.ty - pointer.y) * 0.04;
      const narrow = w < 640;

      // g for the unpinned field: 0.9 at the top of the page, 1 once the cover has scrolled away
      const g = 0.9 + 0.1 * gate;
      const leak = 1 - g; // 0.1 → 0
      if (!reduce) {
        const keep = Math.exp(-leak * 16 * dt);
        for (let i = 0; i < vals.length; i++) vals[i] *= keep;
        // re-excitation: random writes the head didn't pin, each smeared into its neighbours
        const n = Math.floor(vals.length * 0.0035 * (leak / 0.1) + Math.random());
        for (let k = 0; k < n; k++) {
          const i = Math.floor(Math.random() * vals.length);
          const v = 0.35 + Math.random() * 0.65;
          if (vals[i] < v) vals[i] = v;
          const c = i % cols;
          if (c + 1 < cols && vals[i + 1] < v * 0.55) vals[i + 1] = v * 0.55;
          if (c > 0 && vals[i - 1] < v * 0.3) vals[i - 1] = v * 0.3;
        }
        drift = drift * keep;
        if (leak > 0 && Math.random() < dt * 0.35) drift = 0.4 + Math.random() * 0.5;
      }

      // writes: the key walks to row target[k], arrives at tw, the row pins
      let active = -1;
      for (let k = 0; k < IDS.length; k++) {
        const tw = T0 + T_STEP * k + T_TRAVEL;
        if (time >= tw && !pinned[k]) {
          pinned[k] = 1;
          pinOf[target[k]] = k;
        }
        if (time >= tw - T_TRAVEL && time < tw + T_FLASH + 0.5) active = k;
      }
      const done = time >= T_DONE;

      const ox = cx - pointer.x * 10;
      const oy = -pointer.y * 6;
      const edgeX = (r: number) => ox + colX[0] * halfW * rowS[r] - cw * rowS[r] * 1.6;

      // the field, far rows first
      for (let r = 0; r < rows; r++) {
        const s = rowS[r];
        const y = rowY[r] + oy * s;
        const hh = Math.max(0.8, rowH[r]);
        const ww = Math.max(0.8, cw * s);
        const base = ox - halfW * s;
        const step = ((2 * halfW) / cols) * s;
        const depth = 0.45 + 0.55 * s;
        const k = pinOf[r];
        if (k >= 0) {
          const since = time - (T0 + T_STEP * k + T_TRAVEL);
          const flash = reduce ? 0 : Math.max(0, 1 - since / T_FLASH);
          // a soft band of flame under the pinned row
          ctx.globalCompositeOperation = "lighter";
          ctx.fillStyle = C.sel;
          ctx.globalAlpha = 0.07 + 0.25 * flash;
          ctx.fillRect(base - step, y - hh * 0.9, step * (cols + 2), hh * 2.6);
          ctx.globalCompositeOperation = "source-over";
          for (let c = 0; c < cols; c++) {
            const x = base + step * c;
            if (bitAt(k, c)) {
              ctx.fillStyle = C.sel;
              ctx.globalAlpha = 1;
            } else {
              ctx.fillStyle = C.sel;
              ctx.globalAlpha = 0.16;
            }
            ctx.fillRect(x, y, ww, hh);
          }
          if (flash > 0) {
            ctx.globalCompositeOperation = "lighter";
            ctx.fillStyle = "#fff4e8";
            ctx.globalAlpha = flash * 0.85;
            for (let c = 0; c < cols; c++) if (bitAt(k, c)) ctx.fillRect(base + step * c, y, ww, hh);
            ctx.globalCompositeOperation = "source-over";
          }
          continue;
        }
        ctx.globalAlpha = depth;
        const row = r * cols;
        for (let c = 0; c < cols; c++) {
          const v = vals[row + c];
          ctx.fillStyle = FIELD[Math.min(LEVELS - 1, (v * LEVELS) | 0)];
          ctx.fillRect(base + step * c, y, ww, hh);
        }
      }
      ctx.globalAlpha = 1;

      // the key: a one-hot cyan pulse walking down the left edge
      if (!reduce && active >= 0 && !pinned[active]) {
        const from = active === 0 ? 0 : target[active - 1];
        const to = target[active];
        const p = smooth((time - (T0 + T_STEP * active)) / T_TRAVEL);
        const rf = from + (to - from) * p;
        const r0 = Math.min(rows - 1, Math.floor(rf));
        const r1 = Math.min(rows - 1, r0 + 1);
        const f = rf - r0;
        const x = edgeX(r0) + (edgeX(r1) - edgeX(r0)) * f;
        const y = rowY[r0] + (rowY[r1] - rowY[r0]) * f + oy * rowS[r0] + rowH[r0] / 2;
        ctx.globalCompositeOperation = "lighter";
        const glow = ctx.createRadialGradient(x, y, 0, x, y, 12);
        glow.addColorStop(0, "rgba(0,240,255,0.7)");
        glow.addColorStop(1, "rgba(0,240,255,0)");
        ctx.fillStyle = glow;
        ctx.beginPath();
        ctx.arc(x, y, 12, 0, Math.PI * 2);
        ctx.fill();
        ctx.globalCompositeOperation = "source-over";
        ctx.fillStyle = C.info;
        ctx.fillRect(x - 2.5, y - 2.5, 5, 5);
      }
      // the edge itself: a faint cyan tick at every written row
      ctx.fillStyle = C.info;
      for (let k = 0; k < IDS.length; k++) {
        if (!pinned[k]) continue;
        const r = target[k];
        ctx.globalAlpha = 0.55;
        ctx.fillRect(edgeX(r) - 1.5, rowY[r] + oy * rowS[r] + rowH[r] / 2 - 1.5, 3, 3);
      }
      ctx.globalAlpha = 1;

      // HUD, in the left margin: the head being written, then the readback, held for good
      const rowLabel = (r: number) => String(r).padStart(3, "0");
      const k = done ? IDS.length - 1 : active;
      if (k >= 0) {
        const r = target[k];
        const rx = edgeX(r) - 6;
        const ry = rowY[r] + oy * rowS[r] + rowH[r] / 2;
        const ids = narrow ? `[${IDS[0]}, ${IDS[1]}, …]` : `[${IDS.join(", ")}]`;
        const lines: Run[][] = done
          ? [[[`get("title") → ${ids} · `, C.text], ["EXACT", C.sel]], [[`→ ${TITLE}`, C.text]]]
          : [[["HEAD 12 · g 1.000000 · β 1.000000", C.text]], [[`ROW ${rowLabel(r)}`, C.sel], [` ${pinned[k] ? "WRITE" : "SEEK"} · ID ${IDS[k]}`, C.label]]];
        const m = Math.max(width(lines[0], 0), width(lines[1], 1));
        const tx = rx - 34;
        if (tx - m >= 14) {
          // room beside the row
          readout(rx, ry, tx, ry, lines, -1, C.sel);
        } else {
          // no side margin (phones): the slot sits just above the matrix, leader down to the row
          const top = rowY[0] + oy * rowS[0];
          // left-aligned just past the leader, kept inside the cover
          const sx = Math.min(Math.max(rx - 8, 16), w - m - 16);
          readout(rx, ry, sx, top - 22 - 14, lines, 1, C.sel);
        }
      }

      // the unpinned head, on the right, for contrast: its value just drifts away
      {
        const r = Math.round(rows * 0.44);
        const s = rowS[r];
        const rx = ox + halfW * s + 6;
        const ry = rowY[r] + oy * s + rowH[r] / 2;
        const lines: Run[][] = [[[`HEAD 33 · g ${g.toFixed(6)} · ${leak > 1e-4 ? "DRIFT" : "HOLD"}`, C.text]], [[`v ${drift.toFixed(6)}`, C.label]]];
        const tx = rx + 34;
        const m = Math.max(width(lines[0], 0), width(lines[1], 1));
        if (tx + m <= w - 56) readout(rx, ry, tx, ry, lines, 1, C.label, 0.8);
      }

      if (!reduce && onScreen && tabOn) raf = requestAnimationFrame(draw);
      else running = false;
    }
    const resume = () => {
      if (reduce || running || !onScreen || !tabOn) return;
      running = true;
      last = performance.now();
      raf = requestAnimationFrame(draw);
    };

    // pause when the cover is offscreen or the tab is hidden
    const io = new IntersectionObserver(([e]) => {
      onScreen = e.isIntersecting;
      resume();
    });
    io.observe(canvas);
    const onVis = () => {
      tabOn = document.visibilityState === "visible";
      resume();
    };
    document.addEventListener("visibilitychange", onVis);
    const onResize = () => reduce && requestAnimationFrame(draw);
    window.addEventListener("resize", onResize);

    running = true;
    raf = requestAnimationFrame(draw);
    return () => {
      cancelAnimationFrame(raf);
      io.disconnect();
      document.removeEventListener("visibilitychange", onVis);
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onResize);
    };
  }, []);

  return <canvas ref={ref} className="post-cover-canvas" aria-hidden="true" />;
}
