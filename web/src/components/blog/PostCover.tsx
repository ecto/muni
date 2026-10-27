"use client";

import { useEffect, useRef } from "react";

// The PDF cover, live: the inner solar system in a tilted plan view, Earth's
// Lagrange points, and a trajectory that leaves Earth, swings past Mars and
// heads out. Schematic, not to scale. Same geometry as the Typst cover.
const C = {
  bg: "#0b0907",
  line: "#4a3a2a",
  label: "#a08d70",
  text: "#e8e1d2",
  sel: "#ff5f00",
  info: "#00f0ff",
  warn: "#ffd400",
  fault: "#ff2052",
};

const TILT = 0.5;
const ORBITS = [1.15, 1.85, 2.75, 3.85, 7.9];
const A0 = (165 * Math.PI) / 180; // Earth
const A1 = (300 * Math.PI) / 180; // Mars flyby
const A2 = (392 * Math.PI) / 180; // out of frame

function smooth(t: number) {
  return t * t * (3 - 2 * t);
}

function trajectory(n: number) {
  const pts: [number, number][] = [];
  for (let k = 0; k <= n; k++) {
    const a = A0 + ((A2 - A0) * k) / n;
    const r = a <= A1 ? 2.75 + (3.85 - 2.75) * smooth((a - A0) / (A1 - A0)) : 3.85 + (9.6 - 3.85) * Math.pow((a - A1) / (A2 - A1), 1.35);
    pts.push([r * Math.cos(a), r * Math.sin(a)]);
  }
  return pts;
}

export function PostCover({ title, subtitle, children, scrollTo }: { title: string; subtitle: string; children?: React.ReactNode; scrollTo?: string }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const path = trajectory(240);
    const stars = Array.from({ length: 220 }, (_, i) => {
      const s = Math.sin(i * 12.9898) * 43758.5453;
      const t = Math.sin(i * 78.233) * 12345.678;
      return [s - Math.floor(s), t - Math.floor(t), (i % 3) * 0.5 + 0.5] as const;
    });
    let raf = 0;
    const start = performance.now();

    function draw(now: number) {
      if (!canvas || !ctx) return;
      const dpr = window.devicePixelRatio || 1;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;
      if (canvas.width !== w * dpr) {
        canvas.width = w * dpr;
        canvas.height = h * dpr;
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.fillStyle = C.bg;
      ctx.fillRect(0, 0, w, h);

      // unit: scale so Mars' orbit spans most of the width; system sits low
      const u = Math.min(w / 9.5, h / 5.2);
      const cx = w * 0.5;
      const cy = h * 0.66;
      const P = (r: number, a: number): [number, number] => [cx + u * r * Math.cos(a), cy - u * TILT * r * Math.sin(a)];
      const Q = ([x, y]: [number, number]): [number, number] => [cx + u * x, cy - u * TILT * y];
      const t = reduce ? 1 : Math.min(1, (now - start) / 5200);
      const spin = reduce ? 0 : (now - start) / 60000;

      for (const [sx, sy, r] of stars) {
        ctx.fillStyle = "rgba(160,141,112,0.55)";
        ctx.beginPath();
        ctx.arc(sx * w, sy * h, r * 0.6, 0, Math.PI * 2);
        ctx.fill();
      }

      // far halves of the orbits, behind the sun
      const orbit = (r: number, far: boolean) => {
        for (let k = 0; k < 120; k++) {
          const a = (k / 120) * Math.PI * 2;
          const b = ((k + 1) / 120) * Math.PI * 2;
          const mid = (a + b) / 2;
          if (Math.sin(mid) > 0 !== far) continue;
          const depth = (1 - Math.sin(mid)) / 2;
          ctx.strokeStyle = r === 2.75 ? C.label : C.line;
          ctx.globalAlpha = 0.25 + 0.65 * depth;
          ctx.lineWidth = (r === 2.75 ? 1.1 : 0.8) * (0.6 + 0.8 * depth);
          ctx.beginPath();
          ctx.moveTo(...P(r, a));
          ctx.lineTo(...P(r, b));
          ctx.stroke();
        }
        ctx.globalAlpha = 1;
      };
      ORBITS.forEach((r) => orbit(r, true));

      // the sun
      const sun = ctx.createRadialGradient(cx, cy, 0, cx, cy, u * 0.95);
      sun.addColorStop(0, "rgba(255,95,0,1)");
      sun.addColorStop(0.28, "rgba(255,95,0,0.85)");
      sun.addColorStop(0.45, "rgba(255,95,0,0.25)");
      sun.addColorStop(1, "rgba(255,95,0,0)");
      ctx.fillStyle = sun;
      ctx.beginPath();
      ctx.arc(cx, cy, u * 0.95, 0, Math.PI * 2);
      ctx.fill();

      ORBITS.forEach((r) => orbit(r, false));

      // planets drift slowly; Earth and Mars stay where the trajectory needs them
      const planet = (r: number, a: number, rad: number, col: string) => {
        const [x, y] = P(r, a);
        ctx.fillStyle = col;
        ctx.globalAlpha = 0.15;
        ctx.beginPath();
        ctx.arc(x, y, rad * 2.6, 0, Math.PI * 2);
        ctx.fill();
        ctx.globalAlpha = 1;
        ctx.beginPath();
        ctx.arc(x, y, rad, 0, Math.PI * 2);
        ctx.fill();
      };
      planet(1.15, 2.1 + spin * 4, 2.2, C.label);
      planet(1.85, 1.05 + spin * 1.6, 3.2, C.text);
      planet(7.9, 2.06 + spin * 0.08, 6.5, C.warn);
      planet(2.75, A0, 3.6, C.info);
      planet(3.85, A1, 3.0, C.fault);

      // Sun–Earth Lagrange points
      const lp = ([x, y]: [number, number]) => {
        ctx.strokeStyle = C.info;
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(x - 4, y);
        ctx.lineTo(x, y - 4);
        ctx.lineTo(x + 4, y);
        ctx.lineTo(x, y + 4);
        ctx.closePath();
        ctx.stroke();
      };
      lp(P(2.4, A0));
      lp(P(3.1, A0));
      lp(P(2.75, A0 + Math.PI));
      lp(P(2.75, A0 + Math.PI / 3));
      lp(P(2.75, A0 - Math.PI / 3));

      // the trajectory draws itself, then a probe rides it
      const n = Math.max(2, Math.floor(path.length * t));
      for (const [lw, a] of [
        [10, 0.08],
        [5, 0.16],
        [1.6, 1],
      ] as const) {
        ctx.strokeStyle = C.sel;
        ctx.globalAlpha = a;
        ctx.lineWidth = lw;
        ctx.lineCap = "round";
        ctx.lineJoin = "round";
        ctx.beginPath();
        path.slice(0, n).forEach((p, i) => (i === 0 ? ctx.moveTo(...Q(p)) : ctx.lineTo(...Q(p))));
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
      const head = Q(path[n - 1]);
      ctx.fillStyle = "#fff4e8";
      ctx.beginPath();
      ctx.arc(head[0], head[1], 2.6, 0, Math.PI * 2);
      ctx.fill();

      ctx.font = "500 10px 'IBM Plex Mono', ui-monospace, monospace";
      ctx.fillStyle = C.label;
      const e = P(2.75, A0);
      ctx.fillText("EARTH", e[0] - 44, e[1] - 10);
      const m = P(3.85, A1);
      ctx.fillText("MARS", m[0] - 12, m[1] - 12);

      if (!reduce) raf = requestAnimationFrame(draw);
    }
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  }, []);

  return (
    <header className="post-cover">
      <canvas ref={ref} className="post-cover-canvas" aria-hidden="true" />
      <div className="post-cover-text">
        <h1>{title}</h1>
        <p className="post-subtitle">{subtitle}</p>
        {children}
      </div>
      {scrollTo && (
        <a className="post-cover-scroll" href={`#${scrollTo}`} aria-label="Scroll to the text">
          <span>Scroll</span>
          <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 4.5 6 8l3.5-3.5" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" /></svg>
        </a>
      )}
    </header>
  );
}
