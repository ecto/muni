"use client";

import { useEffect, useRef } from "react";

// The PDF cover, live: the inner solar system in a tilted plan view, Earth's
// Lagrange points, and a trajectory that leaves Earth, swings past Mars and
// heads out. Schematic, not to scale. Same geometry as the Typst cover, set in
// motion: every Earth–Mars launch window is one stage of the plan, Desk to Star.
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
const BELT = [4.6, 6.6];
const GRAT = 4.3; // bearing ring, between Mars and the belt
const AU = 2.75; // Earth orbit, in scene units

// Orbital periods in seconds, compressed so the system visibly turns
const W_EARTH = (Math.PI * 2) / 10;
const W_MARS = (Math.PI * 2) / 18.8;
const SYNODIC = (Math.PI * 2) / (W_EARTH - W_MARS); // one launch window per cycle
const T_FLYBY = 6;
const T_EXIT = 10;
const K_FLYBY = (A1 - A0) / (A2 - A0); // fraction of the path at Mars

// One launch per stage of the plan, Desk to Star; the fleet resets when the climb reaches the star.
const STAGES = 8;

// Accelerates off Earth, reaches Mars at T_FLYBY, then leaves faster after the slingshot
function probeProgress(t: number) {
  if (t <= 0) return 0;
  if (t < T_FLYBY) return K_FLYBY * Math.pow(t / T_FLYBY, 1.6);
  return Math.min(1, K_FLYBY + ((1 - K_FLYBY) * (t - T_FLYBY)) / (T_EXIT - T_FLYBY));
}

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
    const path = trajectory(360);
    const rand = (i: number, k: number) => {
      const s = Math.sin(i * 12.9898 + k * 78.233) * 43758.5453;
      return s - Math.floor(s);
    };
    // three star layers; nearer ones are bigger, brighter and move more with the pointer
    const stars = Array.from({ length: 320 }, (_, i) => ({
      x: rand(i, 1),
      y: rand(i, 2),
      z: Math.floor(rand(i, 3) * 3),
      tw: rand(i, 4) * Math.PI * 2,
      sp: 0.6 + rand(i, 5) * 2.2,
    }));
    const belt = Array.from({ length: 420 }, (_, i) => ({
      r: BELT[0] + (BELT[1] - BELT[0]) * Math.pow(rand(i, 6), 0.8),
      a: rand(i, 7) * Math.PI * 2,
      s: 0.4 + rand(i, 8) * 1.1,
    }));
    const pointer = { x: 0, y: 0, tx: 0, ty: 0 };
    const onMove = (e: PointerEvent) => {
      pointer.tx = (e.clientX / window.innerWidth) * 2 - 1;
      pointer.ty = (e.clientY / window.innerHeight) * 2 - 1;
    };
    window.addEventListener("pointermove", onMove);
    let meteor: { x: number; y: number; vx: number; vy: number; life: number } | null = null;
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
      ctx.globalCompositeOperation = "source-over";
      ctx.fillStyle = C.bg;
      ctx.fillRect(0, 0, w, h);

      const time = reduce ? SYNODIC * 0.42 : (now - start) / 1000 + SYNODIC * 0.97;
      pointer.x += (pointer.tx - pointer.x) * 0.04;
      pointer.y += (pointer.ty - pointer.y) * 0.04;

      // unit: scale so Mars' orbit spans most of the width; system sits low
      // on a phone the system is drawn bigger (Mars' orbit runs off the sides) and only the key bodies are labelled
      const narrow = w < 640;
      const u = Math.min(w / (narrow ? 6.4 : 9.5), h / 5.2);
      const cx = w * 0.5 - pointer.x * 10;
      const cy = h * 0.66 - pointer.y * 6;
      const tilt = TILT + pointer.y * 0.06;
      const yaw = pointer.x * 0.08;
      const P = (r: number, a: number): [number, number] => [cx + u * r * Math.cos(a + yaw), cy - u * tilt * r * Math.sin(a + yaw)];
      const depthOf = (a: number) => (1 - Math.sin(a + yaw)) / 2; // 0 far, 1 near

      // stars: twinkle and parallax
      for (const s of stars) {
        const z = s.z + 1;
        const x = ((s.x * w - pointer.x * z * 6) % w + w) % w;
        const y = ((s.y * h - pointer.y * z * 4) % h + h) % h;
        const tw = reduce ? 1 : 0.55 + 0.45 * Math.sin(time * s.sp + s.tw);
        ctx.fillStyle = s.z === 2 && s.tw < 0.5 ? "rgba(200,225,255,1)" : "rgba(232,215,190,1)";
        ctx.globalAlpha = (0.18 + 0.2 * z) * tw;
        ctx.beginPath();
        ctx.arc(x, y, 0.35 + 0.3 * z, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.globalAlpha = 1;

      // the occasional meteor
      if (!reduce) {
        if (!meteor && Math.random() < 0.004) {
          meteor = { x: Math.random() * w, y: Math.random() * h * 0.4, vx: -(3 + Math.random() * 3), vy: 1.4 + Math.random() * 1.2, life: 1 };
        }
        if (meteor) {
          const g = ctx.createLinearGradient(meteor.x, meteor.y, meteor.x - meteor.vx * 22, meteor.y - meteor.vy * 22);
          g.addColorStop(0, `rgba(255,244,232,${0.8 * meteor.life})`);
          g.addColorStop(1, "rgba(255,244,232,0)");
          ctx.strokeStyle = g;
          ctx.lineWidth = 1.2;
          ctx.beginPath();
          ctx.moveTo(meteor.x, meteor.y);
          ctx.lineTo(meteor.x - meteor.vx * 22, meteor.y - meteor.vy * 22);
          ctx.stroke();
          meteor.x += meteor.vx;
          meteor.y += meteor.vy;
          meteor.life -= 0.012;
          if (meteor.life <= 0) meteor = null;
        }
      }

      // orbits, split by depth so the near halves pass in front of the sun
      const orbit = (r: number, far: boolean) => {
        for (let k = 0; k < 160; k++) {
          const a = (k / 160) * Math.PI * 2;
          const b = ((k + 1) / 160) * Math.PI * 2;
          const mid = (a + b) / 2;
          if (Math.sin(mid + yaw) > 0 !== far) continue;
          const depth = depthOf(mid);
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
      const beltPass = (far: boolean) => {
        ctx.fillStyle = C.label;
        for (const b of belt) {
          const a = b.a + time * 0.9 * Math.pow(b.r, -1.5);
          if (Math.sin(a + yaw) > 0 !== far) continue;
          const [x, y] = P(b.r, a);
          ctx.globalAlpha = 0.15 + 0.45 * depthOf(a);
          ctx.fillRect(x, y, b.s, b.s);
        }
        ctx.globalAlpha = 1;
      };
      ORBITS.forEach((r) => orbit(r, true));
      beltPass(true);

      // bearing ring: ticks every 10°, labels at the cardinal bearings
      ctx.strokeStyle = C.line;
      ctx.fillStyle = C.line;
      ctx.font = "500 8px 'IBM Plex Mono', ui-monospace, monospace";
      ctx.textAlign = "center";
      for (let d = 0; d < 360; d += 10) {
        const a = (d * Math.PI) / 180;
        const long = d % 30 === 0;
        ctx.globalAlpha = 0.35 + 0.5 * depthOf(a);
        ctx.lineWidth = 0.8;
        ctx.beginPath();
        ctx.moveTo(...P(GRAT, a));
        ctx.lineTo(...P(GRAT + (long ? 0.22 : 0.1), a));
        ctx.stroke();
        if (d % 90 === 0) ctx.fillText(String(d).padStart(3, "0"), ...P(GRAT + 0.45, a));
      }
      ctx.textAlign = "start";
      ctx.globalAlpha = 1;

      // the sun: a breathing corona, drawn additively
      ctx.globalCompositeOperation = "lighter";
      const flick = reduce ? 1 : 1 + 0.05 * Math.sin(time * 1.7) + 0.03 * Math.sin(time * 4.3 + 1) + 0.02 * Math.sin(time * 9.1);
      const R = u * 0.95 * flick;
      const corona = ctx.createRadialGradient(cx, cy, 0, cx, cy, R * 2.4);
      corona.addColorStop(0, "rgba(255,95,0,0.35)");
      corona.addColorStop(0.35, "rgba(255,70,0,0.12)");
      corona.addColorStop(1, "rgba(255,50,0,0)");
      ctx.fillStyle = corona;
      ctx.beginPath();
      ctx.arc(cx, cy, R * 2.4, 0, Math.PI * 2);
      ctx.fill();
      const sun = ctx.createRadialGradient(cx, cy, 0, cx, cy, R);
      sun.addColorStop(0, "rgba(255,236,200,1)");
      sun.addColorStop(0.12, "rgba(255,140,40,1)");
      sun.addColorStop(0.3, "rgba(255,95,0,0.85)");
      sun.addColorStop(0.5, "rgba(255,95,0,0.25)");
      sun.addColorStop(1, "rgba(255,95,0,0)");
      ctx.fillStyle = sun;
      ctx.beginPath();
      ctx.arc(cx, cy, R, 0, Math.PI * 2);
      ctx.fill();
      ctx.globalCompositeOperation = "source-over";

      ORBITS.forEach((r) => orbit(r, false));
      beltPass(false);

      // planets, lit from the sun
      const planet = (r: number, a: number, rad: number, col: string) => {
        const [x, y] = P(r, a);
        ctx.globalCompositeOperation = "lighter";
        const halo = ctx.createRadialGradient(x, y, 0, x, y, rad * 4);
        halo.addColorStop(0, col + "55");
        halo.addColorStop(1, col + "00");
        ctx.fillStyle = halo;
        ctx.beginPath();
        ctx.arc(x, y, rad * 4, 0, Math.PI * 2);
        ctx.fill();
        ctx.globalCompositeOperation = "source-over";
        const dx = cx - x;
        const dy = cy - y;
        const d = Math.hypot(dx, dy) || 1;
        const body = ctx.createRadialGradient(x + (dx / d) * rad * 0.5, y + (dy / d) * rad * 0.5, 0, x, y, rad * 1.2);
        body.addColorStop(0, "#fff4e8");
        body.addColorStop(0.35, col);
        body.addColorStop(1, "#0b0907");
        ctx.fillStyle = body;
        ctx.beginPath();
        ctx.arc(x, y, rad, 0, Math.PI * 2);
        ctx.fill();
        return [x, y] as [number, number];
      };

      // Earth and Mars repeat their geometry every synodic period, so each cycle is a launch window
      const cycle = Math.floor(time / SYNODIC);
      const tau = time - cycle * SYNODIC;
      const earthA = (t: number) => A0 + W_EARTH * t;
      const marsA = (t: number) => A1 - W_MARS * T_FLYBY + W_MARS * t;
      const offset = earthA(cycle * SYNODIC) - A0;
      const stage = (((cycle - 1) % STAGES) + STAGES) % STAGES; // the first launch after load is Desk

      const me = planet(1.15, 2.1 + (Math.PI * 2 * time) / 5, 2.2, C.label);
      const ve = planet(1.85, 1.05 + (Math.PI * 2 * time) / 7.5, 3.2, C.text);
      const ju = planet(7.9, 2.06 + (Math.PI * 2 * time) / 90, 6.5, C.warn);
      const e = planet(2.75, earthA(time), 3.6, C.info);
      const m = planet(3.85, marsA(time), 3.0, C.fault);

      // Sun–Earth Lagrange points ride with Earth
      const ea = earthA(time);
      const lp = ([x, y]: [number, number]) => {
        ctx.strokeStyle = C.info;
        ctx.globalAlpha = 0.85;
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(x - 4, y);
        ctx.lineTo(x, y - 4);
        ctx.lineTo(x + 4, y);
        ctx.lineTo(x, y + 4);
        ctx.closePath();
        ctx.stroke();
        ctx.globalAlpha = 1;
      };
      lp(P(2.4, ea));
      lp(P(3.1, ea));
      lp(P(2.75, ea + Math.PI));
      lp(P(2.75, ea + Math.PI / 3));
      lp(P(2.75, ea - Math.PI / 3));

      // the fleet: each stage's route stays behind until the climb starts over
      const route = (off: number) => {
        ctx.beginPath();
        path.forEach((p, i) => {
          const [x, y] = P(Math.hypot(p[0], p[1]), Math.atan2(p[1], p[0]) + off);
          if (i === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        });
        ctx.stroke();
      };
      if (!reduce) {
        const tauNow = time - cycle * SYNODIC;
        const ghostNow = tauNow < T_EXIT ? 0 : Math.min(1, (tauNow - T_EXIT) / 3);
        ctx.strokeStyle = C.sel;
        ctx.lineWidth = 1;
        for (let c = cycle - stage; c <= cycle; c++) {
          const age = cycle - c;
          const a = 0.34 * Math.pow(0.9, age) * (c === cycle ? ghostNow : 1);
          if (a < 0.01) continue;
          ctx.globalAlpha = a;
          route(earthA(c * SYNODIC) - A0);
        }
        ctx.globalAlpha = 1;
      }

      // the probe: launch, slingshot past Mars, out of frame; a fading wake behind it
      const Q = ([x, y]: [number, number]) => {
        const r = Math.hypot(x, y);
        return P(r, Math.atan2(y, x) + offset);
      };
      const progress = reduce ? 1 : probeProgress(tau);
      const fade = reduce ? 1 : tau < T_EXIT ? 1 : Math.max(0, 1 - (tau - T_EXIT) / 3);
      const n = Math.floor((path.length - 1) * progress);
      if (fade > 0 && n > 1) {
        // the wake, brightest at the probe
        ctx.lineCap = "round";
        ctx.globalCompositeOperation = "lighter";
        for (let i = 1; i <= n; i++) {
          const k = i / n;
          const a = Math.pow(k, 2.2) * fade;
          const [x0, y0] = Q(path[i - 1]);
          const [x1, y1] = Q(path[i]);
          ctx.strokeStyle = C.sel;
          ctx.globalAlpha = a * 0.12;
          ctx.lineWidth = 9;
          ctx.beginPath();
          ctx.moveTo(x0, y0);
          ctx.lineTo(x1, y1);
          ctx.stroke();
          ctx.globalAlpha = 0.25 + a * 0.75 * fade;
          ctx.lineWidth = 1.6;
          ctx.stroke();
        }
        ctx.globalCompositeOperation = "source-over";
        ctx.globalAlpha = 1;
        if (progress < 1) {
          const [hx, hy] = Q(path[n]);
          ctx.globalCompositeOperation = "lighter";
          const glow = ctx.createRadialGradient(hx, hy, 0, hx, hy, 14);
          glow.addColorStop(0, "rgba(255,244,232,0.9)");
          glow.addColorStop(0.3, "rgba(255,95,0,0.4)");
          glow.addColorStop(1, "rgba(255,95,0,0)");
          ctx.fillStyle = glow;
          ctx.beginPath();
          ctx.arc(hx, hy, 14, 0, Math.PI * 2);
          ctx.fill();
          ctx.globalCompositeOperation = "source-over";
          ctx.fillStyle = "#fff4e8";
          ctx.beginPath();
          ctx.arc(hx, hy, 2.4, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      // shockwave rings at launch and at the flyby
      const ring = ([x, y]: [number, number], since: number, col: string) => {
        if (reduce || since < 0 || since > 1.6) return;
        const k = since / 1.6;
        ctx.strokeStyle = col;
        ctx.globalAlpha = (1 - k) * 0.8;
        ctx.lineWidth = 1.2;
        ctx.beginPath();
        ctx.ellipse(x, y, 4 + k * 36, (4 + k * 36) * tilt, 0, 0, Math.PI * 2);
        ctx.stroke();
        ctx.globalAlpha = 1;
      };
      ring(Q(path[0]), tau, C.info);
      ring(m, tau - T_FLYBY, C.fault);

      // flight-control annotations: a reticle on each body, a leader, and a callout
      const mono = "'IBM Plex Mono', ui-monospace, monospace";
      const callout = ([x, y]: [number, number], lines: string[], col: string, big = 7) => {
        const side = x > w - 170 ? -1 : 1;
        ctx.strokeStyle = col;
        ctx.lineWidth = 1;
        ctx.globalAlpha = 0.9;
        for (const [sx, sy] of [[-1, -1], [1, -1], [1, 1], [-1, 1]] as const) {
          ctx.beginPath();
          ctx.moveTo(x + sx * big, y + sy * (big - 3));
          ctx.lineTo(x + sx * big, y + sy * big);
          ctx.lineTo(x + sx * (big - 3), y + sy * big);
          ctx.stroke();
        }
        const lx = x + side * (big + 14);
        const ly = y - big - 14;
        ctx.globalAlpha = 0.55;
        ctx.beginPath();
        ctx.moveTo(x + side * (big + 2), y - big - 2);
        ctx.lineTo(lx, ly);
        ctx.lineTo(lx + side * 10, ly);
        ctx.stroke();
        ctx.textAlign = side > 0 ? "left" : "right";
        lines.forEach((t, i) => {
          ctx.font = `${i === 0 ? 500 : 400} ${i === 0 ? 10 : 9}px ${mono}`;
          ctx.fillStyle = i === 0 ? C.text : col;
          ctx.globalAlpha = i === 0 ? 0.95 : 0.8;
          ctx.fillText(t, lx + side * 14, ly + 3 + i * 12);
        });
        ctx.textAlign = "start";
        ctx.globalAlpha = 1;
      };
      if (!narrow) {
        callout(me, ["MERCURY", "0.39 AU"], C.label, 6);
        callout(ve, ["VENUS", "0.72 AU"], C.label, 6);
      }
      callout(e, ["EARTH", "1.00 AU"], C.info);
      callout(m, ["MARS", "1.52 AU"], C.fault);
      if (!narrow && ju[0] > 0 && ju[0] < w && ju[1] > 0 && ju[1] < h) callout(ju, ["JUPITER", "5.20 AU"], C.warn, 10);

      // Lagrange point designators (too small to read on a phone)
      if (!narrow) {
        ctx.font = `500 8px ${mono}`;
        ctx.fillStyle = C.info;
        ctx.globalAlpha = 0.6;
        ([[2.4, 0, "L1"], [3.1, 0, "L2"], [2.75, Math.PI, "L3"], [2.75, Math.PI / 3, "L4"], [2.75, -Math.PI / 3, "L5"]] as const).forEach(([r, da, t]) => {
          const [x, y] = P(r, ea + da);
          ctx.fillText(t, x + 6, y + 10);
        });
        ctx.globalAlpha = 1;
      }

      // the probe: mission, clock, range and speed, live
      if (!reduce && fade > 0 && progress > 0 && progress < 1) {
        const at = (pr: number) => path[Math.floor((path.length - 1) * pr)];
        const [px, py] = at(progress);
        const [qx, qy] = at(Math.max(0, probeProgress(tau - 0.2)));
        const speed = (Math.hypot(px - qx, py - qy) / 0.2) * (30 / (AU * W_EARTH)); // km/s, Earth = 30
        callout(Q([px, py]), [`MSN-${String(stage + 1).padStart(2, "0")}`, `T+${tau.toFixed(1).padStart(4, "0")} S`, `R ${(Math.hypot(px, py) / AU).toFixed(2)} AU`, `V ${speed.toFixed(1)} KM/S`], C.sel, 9);
      }

      if (!reduce) raf = requestAnimationFrame(draw);
    }
    raf = requestAnimationFrame(draw);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("pointermove", onMove);
    };
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
