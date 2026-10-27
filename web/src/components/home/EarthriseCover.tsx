"use client";

import { useEffect, useRef } from "react";

// Earthrise, drawn like a flight controller's display rather than a photo: Earth as a
// wireframe globe with its terminator and city lights, the Moon's limb as one lit line,
// reticles and callouts on what matters, and one orange light: a seed waiting on the Moon.
// Same palette and annotation style as the plan's starmap (PostCover).
const C = {
  bg: "#0b0907",
  surface: "#15110d",
  line: "#2a2119",
  lineHi: "#4a3a2a",
  label: "#a08d70",
  text: "#e8e1d2",
  sel: "#ff5f00",
  info: "#00f0ff",
  warn: "#ffd400",
};

const ORBIT = 72; // seconds per lap of the Moon at rest; Earth rises and sets once a lap
const GROUND = 0.4; // how far the surface slides under us per radian of orbit, in screen widths
const SUN = { x: 0.95, y: -0.15, z: 0.27 }; // from the right: a waxing Earth
const MONO = "'IBM Plex Mono', ui-monospace, monospace";
// Land at 5° resolution, rows from 90°N to 90°S, columns from 180°W to 180°E. Coarse on purpose:
// drawn as a dot matrix it reads as a navigation display, not a photo.
const LAND = [
  "........................................................................",
  "..................####..########........................................",
  "............####################.......##..............##...............",
  "...#####..##########.....#######........##......#######################.",
  "...#####################.#########....##################################",
  "...##############...###...##.........###########################..######",
  "....#############....###...........#.###.#######################...##...",
  "..........###############.........##############################...#....",
  "...........##############..........######...#####################.......",
  "...........###########............#######...###################.#.......",
  "...........##########.............##.....########################.......",
  "............########..............###########################.#.........",
  ".............####..#.............############################...........",
  "..............###...#...........################.############...........",
  "...............####.............###############...###..###..#...........",
  "..................#..###........###############....#...###..#...........",
  "....................#####........#############......#..##....#..........",
  "....................######...........########..........#####............",
  "....................#########........#######............#####.####......",
  "....................#########.........######.............##.....##......",
  ".....................########.........########................###.......",
  "......................######..........########..............######......",
  "......................######..........#####..#............#########.....",
  ".....................######............####...............#########.....",
  ".....................#####.............###.................###..##......",
  ".....................####.......................................##....#.",
  ".....................###.........................................#...##.",
  ".....................##..............................................#..",
  ".....................##.................................................",
  "........................................................................",
  ".......................##...............................................",
  "......######################....########################################",
  ".....#######################...#########################################",
  "########################################################################",
  "########################################################################",
  "########################################################################",
];
const LAND_DOTS: [number, number][] = [];
LAND.forEach((row, r) =>
  [...row].forEach((ch, c) => {
    if (ch !== "#") return;
    for (const [dr, dc] of [[0.25, 0.25], [0.25, 0.75], [0.75, 0.25], [0.75, 0.75]]) {
      LAND_DOTS.push([((90 - (r + dr) * 5) * Math.PI) / 180, ((-180 + (c + dc) * 5) * Math.PI) / 180]);
    }
  }),
);
const CLE = { lat: (41.5 * Math.PI) / 180, lon: (-81.69 * Math.PI) / 180 }; // Cleveland, Ohio

export function EarthriseCover({ title, subtitle, children }: { title: string; subtitle: string; children?: React.ReactNode }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const rand = (i: number, k: number) => {
      const s = Math.sin(i * 12.9898 + k * 78.233) * 43758.5453;
      return s - Math.floor(s);
    };
    const stars = Array.from({ length: 240 }, (_, i) => ({ x: rand(i, 1), y: rand(i, 2), z: Math.floor(rand(i, 3) * 3), tw: rand(i, 4) * 6.28, sp: 0.5 + rand(i, 5) * 2 }));
    // city lights cluster into a few bands, the way coasts and rivers do
    // night lights from real metros (lat, lon in degrees, weight), each a small jittered cluster,
    // so the night side shows recognizable coasts rather than noise
    const METROS: [number, number, number][] = [
      [41.5, -81.7, 3], [40.7, -74, 5], [42.4, -71.1, 3], [39, -77, 3], [41.9, -87.6, 4], [42.3, -83, 3], [43.7, -79.4, 3], [45.5, -73.6, 2],
      [33.7, -84.4, 3], [25.8, -80.2, 3], [29.8, -95.4, 3], [32.8, -96.8, 3], [39.7, -105, 2], [33.4, -112, 2], [34, -118.2, 5], [37.8, -122.4, 3],
      [47.6, -122.3, 2], [19.4, -99.1, 5], [20.7, -103.3, 2], [4.7, -74.1, 3], [-12, -77, 3], [-23.5, -46.6, 5], [-22.9, -43.2, 4], [-34.6, -58.4, 4],
      [-33.4, -70.6, 3], [51.5, -0.1, 5], [48.9, 2.4, 5], [52.5, 13.4, 3], [40.4, -3.7, 3], [41.9, 12.5, 3], [45.5, 9.2, 3], [52.4, 4.9, 3],
      [50.1, 8.7, 2], [55.8, 37.6, 4], [59.9, 30.3, 2], [41, 29, 4], [30, 31.2, 5], [6.5, 3.4, 4], [-26.2, 28, 3], [-1.3, 36.8, 2],
      [24.7, 46.7, 2], [25.2, 55.3, 3], [35.7, 51.4, 3], [24.9, 67, 4], [28.6, 77.2, 6], [19.1, 72.9, 6], [13, 77.6, 4], [22.6, 88.4, 5],
      [23.8, 90.4, 5], [13.8, 100.5, 4], [1.3, 103.8, 3], [-6.2, 106.8, 5], [14.6, 121, 4], [22.3, 114.2, 4], [23.1, 113.3, 5], [31.2, 121.5, 6],
      [39.9, 116.4, 5], [30.6, 104.1, 3], [37.6, 127, 5], [35.7, 139.7, 6], [34.7, 135.5, 4], [-33.9, 151.2, 3], [-37.8, 145, 3], [21, 105.8, 3],
    ];
    const cities = METROS.flatMap(([la, lo, wgt], m) =>
      Array.from({ length: wgt * 4 }, (_, i) => ({
        lat: ((la + (rand(m * 31 + i, 6) - 0.5) * wgt * 1.1) * Math.PI) / 180,
        lon: ((lo + (rand(m * 31 + i, 7) - 0.5) * wgt * 1.4) * Math.PI) / 180,
      })),
    );
    const craters = Array.from({ length: 34 }, (_, i) => ({ x: rand(i, 11), d: Math.pow(rand(i, 12), 1.6), r: 0.006 + Math.pow(rand(i, 13), 2.5) * 0.045 }));
    // the mouse only reveals annotations; nothing moves with it
    const mouse = { x: -1e4, y: -1e4 };
    const hover = { earth: 0, seed: 0, eagle: 0 };
    const onMove = (e: PointerEvent) => {
      const r = canvas.getBoundingClientRect();
      mouse.x = e.clientX - r.left;
      mouse.y = e.clientY - r.top;
    };
    window.addEventListener("pointermove", onMove);
    // scrolling pushes the orbit along (or back); the push decays and the lap carries on at its own pace
    let orb = -Math.PI / 2 + 0.25; // start with Earth just under the horizon, about to rise
    let push = 0;
    const onWheel = (e: WheelEvent) => {
      push = Math.max(-3, Math.min(3, push + e.deltaY * 0.0025));
    };
    let touchY = 0;
    const onTouchStart = (e: TouchEvent) => {
      touchY = e.touches[0].clientY;
    };
    const onTouchMove = (e: TouchEvent) => {
      const y = e.touches[0].clientY;
      push = Math.max(-3, Math.min(3, push + (touchY - y) * 0.006));
      touchY = y;
    };
    window.addEventListener("wheel", onWheel, { passive: true });
    window.addEventListener("touchstart", onTouchStart, { passive: true });
    window.addEventListener("touchmove", onTouchMove, { passive: true });
    let typed = "";
    let frogAt = -1;
    const onKey = (e: KeyboardEvent) => {
      typed = (typed + e.key.toLowerCase()).slice(-4);
      if (typed === "frog") frogAt = performance.now();
    };
    window.addEventListener("keydown", onKey);
    let last = performance.now();
    const sn = Math.hypot(SUN.x, SUN.y, SUN.z);
    const L = [SUN.x / sn, SUN.y / sn, SUN.z / sn];
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
      const t = reduce ? 60 : (now - start) / 1000;
      const dt = Math.min(0.05, (now - last) / 1000);
      last = now;
      if (reduce) orb = 0.9;
      else {
        orb += ((Math.PI * 2) / ORBIT) * (1 + push * 6) * dt;
        push *= Math.pow(0.12, dt); // most of a push is spent within a second
      }
      const rise = Math.sin(orb); // Earth's height over the limb: -1 hidden, 1 highest
      const gx = orb * w * GROUND;

      for (const s of stars) {
        const z = s.z + 1;
        const x = (((s.x * w - gx * 0.05) % w) + w) % w;
        const y = (((s.y * h) % h) + h) % h;
        ctx.globalAlpha = (0.14 + 0.16 * z) * (reduce ? 1 : 0.6 + 0.4 * Math.sin(t * s.sp + s.tw));
        ctx.fillStyle = C.label;
        ctx.fillRect(x, y, 0.5 + 0.35 * z, 0.5 + 0.35 * z);
      }
      ctx.globalAlpha = 1;

      const horizon = h * 0.9;
      const R = Math.max(54, Math.min(w, h) * 0.12);
      const ex = w * (0.5 - 0.1 * Math.cos(orb));
      const ey = horizon + R * 1.1 - ((rise + 1) / 2) * R * 2.05;
      const spin = -CLE.lon - 0.35 + t * 0.11; // one turn a minute; Cleveland faces us at load
      const tilt = 0.41;

      // project a point on the globe (lat, lon) to screen, with depth (z > 0 faces us)
      const proj = (lat: number, lon: number) => {
        const cl = Math.cos(lat);
        let x = cl * Math.sin(lon + spin);
        let y = -Math.sin(lat);
        const z = cl * Math.cos(lon + spin);
        const xt = x * Math.cos(tilt) - y * Math.sin(tilt);
        y = x * Math.sin(tilt) + y * Math.cos(tilt);
        x = xt;
        return { x: ex + x * R, y: ey + y * R, z, lit: x * L[0] + y * L[1] + z * L[2] };
      };

      // the disc and its day side, flat: surface for night, a faint cyan wash for day
      ctx.save();
      ctx.beginPath();
      ctx.arc(ex, ey, R, 0, Math.PI * 2);
      ctx.fillStyle = C.surface;
      ctx.fill();
      ctx.clip();
      const terminator = Math.atan2(L[2], L[0]); // how far the lit side wraps toward us
      ctx.beginPath();
      ctx.arc(ex, ey, R, -Math.PI / 2, Math.PI / 2);
      ctx.ellipse(ex, ey, Math.abs(Math.sin(terminator)) * R, R, 0, Math.PI / 2, -Math.PI / 2, Math.sin(terminator) > 0);
      ctx.fillStyle = "rgba(0,240,255,0.07)";
      ctx.fill();

      // land, as a dot matrix: bright on the day side, faint on the night side
      for (const [la, lo] of LAND_DOTS) {
        const p = proj(la, lo);
        if (p.z < 0.02) continue;
        const day = p.lit > 0;
        ctx.fillStyle = day ? C.info : C.label;
        ctx.globalAlpha = (day ? 0.25 + 0.6 * Math.min(1, p.lit * 2) : 0.18) * (0.4 + 0.6 * p.z);
        ctx.fillRect(p.x - 0.7, p.y - 0.7, 1.4, 1.4);
      }
      ctx.globalAlpha = 1;

      // graticule: parallels and meridians, brighter on the day side
      ctx.lineWidth = 0.8;
      const seg = (pts: { x: number; y: number; z: number; lit: number }[], gain = 1) => {
        for (let i = 1; i < pts.length; i++) {
          const a = pts[i - 1];
          const b = pts[i];
          if (a.z < 0 || b.z < 0) continue;
          ctx.strokeStyle = C.info;
          ctx.globalAlpha = (a.lit > 0 ? 0.32 : 0.1) * gain;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.stroke();
        }
      };
      const parallel = (lat: number) => Array.from({ length: 97 }, (_, i) => proj((lat * Math.PI) / 180, (i / 96) * Math.PI * 2));
      const meridian = (lon: number) => Array.from({ length: 49 }, (_, i) => proj(((i / 48) * 180 - 90) * (Math.PI / 180), (lon * Math.PI) / 180));
      ctx.lineWidth = 0.5;
      for (let lat = -75; lat <= 75; lat += 15) if (lat % 30 !== 0) seg(parallel(lat), 0.45);
      for (let lon = 15; lon < 360; lon += 30) seg(meridian(lon), 0.45);
      ctx.lineWidth = 0.8;
      for (let lat = -60; lat <= 60; lat += 30) seg(parallel(lat), lat === 0 ? 1.8 : 1);
      for (let lon = 0; lon < 360; lon += 30) seg(meridian(lon), 1);
      // polar circles and tropics, dashed, as on a navigator's globe
      ctx.setLineDash([2, 3]);
      for (const lat of [66.56, 23.44, -23.44, -66.56]) seg(parallel(lat), 0.8);
      ctx.setLineDash([]);
      ctx.globalAlpha = 1;
      // the terminator: where day meets night, a thin line along the edge of the lit side
      ctx.strokeStyle = C.info;
      ctx.globalAlpha = 0.5;
      ctx.lineWidth = 0.8;
      ctx.beginPath();
      ctx.ellipse(ex, ey, Math.abs(Math.sin(terminator)) * R, R, 0, -Math.PI / 2, Math.PI / 2, Math.sin(terminator) < 0);
      ctx.stroke();
      ctx.globalAlpha = 1;

      // city lights on the night side
      ctx.fillStyle = C.warn;
      for (const c of cities) {
        const p = proj(c.lat, c.lon);
        if (p.z < 0.05 || p.lit > -0.02) continue;
        ctx.globalAlpha = Math.min(0.85, 0.25 + p.z * 0.7);
        ctx.fillRect(p.x, p.y, 1.2, 1.2);
      }
      ctx.globalAlpha = 1;
      ctx.restore();

      // atmosphere: one cyan line, with a little bloom on the sunward limb
      ctx.globalCompositeOperation = "lighter";
      for (const [lw, a] of [[6, 0.06], [2.5, 0.18], [1.1, 0.9]] as const) {
        ctx.strokeStyle = C.info;
        ctx.globalAlpha = a;
        ctx.lineWidth = lw;
        ctx.beginPath();
        ctx.arc(ex, ey, R + 0.5, -Math.PI * 0.55, Math.PI * 0.55);
        ctx.stroke();
      }
      ctx.globalAlpha = 0.35;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(ex, ey, R + 0.5, Math.PI * 0.55, Math.PI * 1.45);
      ctx.stroke();
      ctx.globalAlpha = 1;
      ctx.globalCompositeOperation = "source-over";

      // the Moon: a dark mass under one lit limb, craters as outlines
      const Rm = w * 2.4 + h;
      const mcx = w * 0.5;
      const mcy = horizon + Rm;
      const ground = ctx.createLinearGradient(0, horizon, 0, h);
      ground.addColorStop(0, C.surface);
      ground.addColorStop(1, C.bg);
      ctx.fillStyle = ground;
      ctx.beginPath();
      ctx.arc(mcx, mcy, Rm, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = C.label;
      ctx.globalAlpha = 0.7;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(mcx, mcy, Rm, Math.PI * 1.2, Math.PI * 1.8);
      ctx.stroke();
      const depth = h - horizon;
      ctx.save();
      ctx.beginPath();
      ctx.arc(mcx, mcy, Rm - 3, 0, Math.PI * 2);
      ctx.clip();
      ctx.strokeStyle = C.lineHi;
      for (const c of craters) {
        const x = ((((c.x * w - gx * (0.6 + c.d)) % (w * 1.2)) + w * 1.2) % (w * 1.2)) - w * 0.1;
        const surface = mcy - Math.sqrt(Math.max(0, Rm * Rm - (x - mcx) * (x - mcx))); // the limb curves down at the edges
        const y = surface + 8 + c.d * depth * 0.85;
        const rx = c.r * w * (0.35 + c.d);
        const ry = rx * (0.1 + c.d * 0.28);
        const a = 0.3 + 0.55 * (1 - c.d);
        // floor in shadow
        ctx.globalAlpha = a * 0.55;
        ctx.fillStyle = C.bg;
        ctx.beginPath();
        ctx.ellipse(x, y, rx, ry, 0, 0, Math.PI * 2);
        ctx.fill();
        // rim
        ctx.globalAlpha = a * 0.7;
        ctx.strokeStyle = C.lineHi;
        ctx.lineWidth = 1;
        ctx.stroke();
        // the far inner wall faces the sun (from the right) and catches it
        ctx.globalAlpha = a;
        ctx.strokeStyle = C.label;
        ctx.beginPath();
        ctx.ellipse(x, y, rx * 0.94, ry * 0.86, 0, Math.PI * 1.05, Math.PI * 1.75);
        ctx.stroke();
        // a central peak in the large ones
        if (c.r > 0.03) {
          ctx.globalAlpha = a * 0.8;
          ctx.beginPath();
          ctx.moveTo(x - rx * 0.12, y + ry * 0.15);
          ctx.lineTo(x, y - ry * 0.35);
          ctx.lineTo(x + rx * 0.12, y + ry * 0.15);
          ctx.stroke();
        }
      }
      ctx.restore();
      ctx.globalAlpha = 1;

      // one flame: a seed on the Moon, waiting, with rings like the paper's beacon
      const bx = ((((w * 0.7 - gx * 0.6) % (w * 1.6)) + w * 1.6) % (w * 1.6)) - w * 0.3; // the seed slides past once in a while
      const by = horizon + ((bx - mcx) * (bx - mcx)) / (2 * Rm);
      const pulse = reduce ? 0.5 : (t * 0.45) % 1;
      for (let i = 0; i < 3; i++) {
        const r = 5 + i * 8 + pulse * 8;
        ctx.fillStyle = C.sel;
        ctx.globalAlpha = 0.16 * (1 - (i + pulse) / 3);
        ctx.beginPath();
        ctx.arc(bx, by, r, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.globalAlpha = 1;
      ctx.fillStyle = C.sel;
      ctx.beginPath();
      ctx.arc(bx, by, 2.4, 0, Math.PI * 2);
      ctx.fill();

      // easter egg: Eagle's descent stage, still at Tranquility Base, slides past once a lap or so
      const surfaceAt = (x: number) => mcy - Math.sqrt(Math.max(0, Rm * Rm - (x - mcx) * (x - mcx)));
      const lx = ((((w * 0.2 - gx * 0.6) % (w * 2.3)) + w * 2.3) % (w * 2.3)) - w * 0.4;
      const ly = surfaceAt(lx) + 5;
      if (lx > -20 && lx < w + 20) {
        ctx.strokeStyle = C.label;
        ctx.lineWidth = 1;
        ctx.globalAlpha = 0.85;
        ctx.beginPath();
        ctx.moveTo(lx - 5, ly - 4);
        ctx.lineTo(lx + 5, ly - 4);
        ctx.lineTo(lx + 4, ly - 8);
        ctx.lineTo(lx - 4, ly - 8);
        ctx.closePath();
        for (const sx of [-1, 1]) {
          ctx.moveTo(lx + sx * 4, ly - 5);
          ctx.lineTo(lx + sx * 8, ly);
          ctx.moveTo(lx + sx * 9.5, ly);
          ctx.lineTo(lx + sx * 6.5, ly);
        }
        ctx.stroke();
        ctx.globalAlpha = 1;
      }

      // easter egg: type "frog" and one hops across the Moon, low gravity and all
      if (frogAt >= 0) {
        const ft = (now - frogAt) / 1000;
        if (ft > 7) frogAt = -1;
        else {
          const fx = -20 + (ft / 7) * (w + 40);
          const hop = (ft * 0.9) % 1; // slow hops: a sixth of a g
          const fy = surfaceAt(fx) + 4 - Math.sin(hop * Math.PI) * 38;
          ctx.fillStyle = "#39ff14";
          ctx.beginPath();
          ctx.ellipse(fx, fy - 3, 5, 3.5, 0, 0, Math.PI * 2);
          ctx.arc(fx - 2.2, fy - 6.5, 1.5, 0, Math.PI * 2);
          ctx.arc(fx + 2.2, fy - 6.5, 1.5, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      // callouts, as on the starmap: reticle, leader, name, then data
      const callout = (x: number, y: number, lines: string[], col: string, big: number, side: 1 | -1, fade = 1) => {
        ctx.strokeStyle = col;
        ctx.lineWidth = 1;
        ctx.globalAlpha = 0.9 * fade;
        for (const [sx, sy] of [[-1, -1], [1, -1], [1, 1], [-1, 1]] as const) {
          ctx.beginPath();
          ctx.moveTo(x + sx * big, y + sy * (big - 4));
          ctx.lineTo(x + sx * big, y + sy * big);
          ctx.lineTo(x + sx * (big - 4), y + sy * big);
          ctx.stroke();
        }
        const lx = x + side * (big + 16);
        const ly = y - big - 16;
        ctx.globalAlpha = 0.55 * fade;
        ctx.beginPath();
        ctx.moveTo(x + side * (big + 2), y - big - 2);
        ctx.lineTo(lx, ly);
        ctx.lineTo(lx + side * 10, ly);
        ctx.stroke();
        ctx.textAlign = side > 0 ? "left" : "right";
        lines.forEach((s, i) => {
          ctx.font = `${i === 0 ? 500 : 400} ${i === 0 ? 10 : 9}px ${MONO}`;
          ctx.fillStyle = i === 0 ? C.text : col;
          ctx.globalAlpha = (i === 0 ? 0.95 : 0.8) * fade;
          ctx.fillText(s, lx + side * 14, ly + 3 + i * 12);
        });
        ctx.textAlign = "start";
        ctx.globalAlpha = 1;
      };
      // annotations appear only under the mouse, fading in and out
      const near = (x: number, y: number, r: number) => Math.hypot(mouse.x - x, mouse.y - y) < r;
      hover.earth += ((near(ex, ey, R + 24) ? 1 : 0) - hover.earth) * 0.15;
      hover.seed += ((near(bx, by, 28) ? 1 : 0) - hover.seed) * 0.15;
      if (hover.seed > 0.01) callout(bx, by, ["SEED-01", "MOON · WAITING"], C.sel, 9, 1, hover.seed);
      hover.eagle += ((near(lx, ly - 4, 22) ? 1 : 0) - hover.eagle) * 0.15;
      if (hover.eagle > 0.01) callout(lx, ly - 4, ["EAGLE", "TRANQUILITY BASE · 1969"], C.label, 10, lx > w - 200 ? -1 : 1, hover.eagle);
      const show = Math.min(1, Math.max(0, (rise + 0.1) * 2.5)) * hover.earth;
      if (show > 0.01) {
        callout(ex, ey, ["EARTH", "384,400 KM", `PHASE ${Math.round(((1 + L[2]) / 2) * 100)}%`], C.info, R + 8, -1, show);
        const cle = proj(CLE.lat, CLE.lon);
        if (cle.z > 0.15) {
          const f = show * Math.min(1, (cle.z - 0.15) * 4);
          ctx.globalAlpha = f;
          ctx.fillStyle = C.text;
          ctx.fillRect(cle.x - 1.5, cle.y - 1.5, 3, 3);
          ctx.globalAlpha = 1;
          callout(cle.x, cle.y, ["CLEVELAND, OH", "41.50°N 81.69°W"], C.text, 6, 1, f);
        }
      }

      if (!reduce) raf = requestAnimationFrame(draw);
    }
    raf = requestAnimationFrame(draw);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("wheel", onWheel);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("touchstart", onTouchStart);
      window.removeEventListener("touchmove", onTouchMove);
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
    </header>
  );
}
