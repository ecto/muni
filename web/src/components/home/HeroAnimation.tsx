"use client";

import { useEffect, useRef } from "react";

/**
 * Full-viewport animated dot-grid canvas.
 *
 * Two product motifs emerge from the grid:
 *   Left  — Rover: a bright dot traces a meandering path, leaving a
 *           warm trail of "cleared" dots in its wake.
 *   Right — vcad: dots connect to form a wireframe cube that slowly
 *           rotates, morphing between primitives.
 *
 * Ambient waves ripple across the field. Mouse creates local activation.
 * All neighbors precomputed — render loop is O(n).
 */

const GRID = 28;
const DOT_R = 1;
const BASE_ALPHA = 0.035;
const WAVE_SPEED = 100;
const WAVE_WIDTH = 160;
const DECAY = 2.0;

const OR = 255, OG = 106, OB = 0; // orange
const WR = 200, WG = 200, WB = 200; // white

// ─── Rover path ──────────────────────────────────────────────────────────
// A dot meanders through the left zone, leaving a warm trail.
// Path is a series of waypoints; the dot lerps between them.

function makeRoverWaypoints(cx: number, cy: number, radius: number): { x: number; y: number }[] {
  // Meandering path that loops — like a rover clearing a sidewalk grid
  const pts: { x: number; y: number }[] = [];
  const rows = 5;
  const rowH = (radius * 1.6) / rows;
  const startY = cy - radius * 0.8;
  for (let r = 0; r < rows; r++) {
    const y = startY + r * rowH;
    if (r % 2 === 0) {
      pts.push({ x: cx - radius * 0.6, y });
      pts.push({ x: cx + radius * 0.6, y });
    } else {
      pts.push({ x: cx + radius * 0.6, y });
      pts.push({ x: cx - radius * 0.6, y });
    }
  }
  // Loop back to start
  pts.push({ x: cx - radius * 0.6, y: startY });
  return pts;
}

// ─── vcad wireframe ──────────────────────────────────────────────────────
// Rotating 3D wireframe projected to 2D. Morphs between shapes.

interface Vec3 { x: number; y: number; z: number }

const CUBE_VERTS: Vec3[] = [
  { x: -1, y: -1, z: -1 }, { x: 1, y: -1, z: -1 },
  { x: 1, y: 1, z: -1 },  { x: -1, y: 1, z: -1 },
  { x: -1, y: -1, z: 1 },  { x: 1, y: -1, z: 1 },
  { x: 1, y: 1, z: 1 },   { x: -1, y: 1, z: 1 },
];
const CUBE_EDGES: [number, number][] = [
  [0, 1], [1, 2], [2, 3], [3, 0],
  [4, 5], [5, 6], [6, 7], [7, 4],
  [0, 4], [1, 5], [2, 6], [3, 7],
];

// Cylinder: 8 top + 8 bottom verts
function makeCylinder(n: number): { verts: Vec3[]; edges: [number, number][] } {
  const verts: Vec3[] = [];
  const edges: [number, number][] = [];
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2;
    verts.push({ x: Math.cos(a), y: -1, z: Math.sin(a) });
  }
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2;
    verts.push({ x: Math.cos(a), y: 1, z: Math.sin(a) });
  }
  // Top ring
  for (let i = 0; i < n; i++) edges.push([i, (i + 1) % n]);
  // Bottom ring
  for (let i = 0; i < n; i++) edges.push([n + i, n + (i + 1) % n]);
  // Verticals
  for (let i = 0; i < n; i++) edges.push([i, n + i]);
  return { verts, edges };
}

const CYL = makeCylinder(8);

// Lerp between two vert sets (pad shorter with center)
function lerpShapes(a: Vec3[], b: Vec3[], t: number): Vec3[] {
  const len = Math.max(a.length, b.length);
  const result: Vec3[] = [];
  for (let i = 0; i < len; i++) {
    const va = a[i % a.length];
    const vb = b[i % b.length];
    result.push({
      x: va.x + (vb.x - va.x) * t,
      y: va.y + (vb.y - va.y) * t,
      z: va.z + (vb.z - va.z) * t,
    });
  }
  return result;
}

function rotateY(v: Vec3, a: number): Vec3 {
  const c = Math.cos(a), s = Math.sin(a);
  return { x: v.x * c + v.z * s, y: v.y, z: -v.x * s + v.z * c };
}

function rotateX(v: Vec3, a: number): Vec3 {
  const c = Math.cos(a), s = Math.sin(a);
  return { x: v.x, y: v.y * c - v.z * s, z: v.y * s + v.z * c };
}

function project(v: Vec3, cx: number, cy: number, scale: number): { x: number; y: number } {
  const perspective = 4 / (4 + v.z);
  return { x: cx + v.x * scale * perspective, y: cy + v.y * scale * perspective };
}

export function HeroAnimation() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;

    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    let w = 0, h = 0, cols = 0, rows = 0, count = 0;
    let xs: Float32Array, ys: Float32Array, activations: Float32Array;
    let neighborPairs: Uint32Array;
    let pairCount = 0;

    // Rover state
    let roverWaypoints: { x: number; y: number }[] = [];
    let roverPos = { x: 0, y: 0 };
    let roverWpIdx = 0;
    let roverProgress = 0;
    const ROVER_SPEED = 60; // px/sec
    let roverTrail: { x: number; y: number; age: number }[] = [];

    // Waves
    interface Wave { cx: number; cy: number; radius: number }
    let waves: Wave[] = [];
    let lastEdgeWave = 0;

    let mouse = { x: -9999, y: -9999 };
    let animId = 0, lastTime = 0;

    function resize() {
      const dpr = Math.min(window.devicePixelRatio, 2);
      w = window.innerWidth;
      h = window.innerHeight;
      canvas!.width = w * dpr;
      canvas!.height = h * dpr;
      canvas!.style.width = w + "px";
      canvas!.style.height = h + "px";
      ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);

      cols = Math.ceil(w / GRID) + 1;
      rows = Math.ceil(h / GRID) + 1;
      count = cols * rows;
      const ox = (w - (cols - 1) * GRID) / 2;
      const oy = (h - (rows - 1) * GRID) / 2;

      xs = new Float32Array(count);
      ys = new Float32Array(count);
      activations = new Float32Array(count);

      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const i = r * cols + c;
          xs[i] = ox + c * GRID;
          ys[i] = oy + r * GRID;
        }
      }

      // Precompute neighbor pairs into flat typed array
      const tmpPairs: number[] = [];
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const i = r * cols + c;
          if (c + 1 < cols) { tmpPairs.push(i, i + 1); }
          if (r + 1 < rows) { tmpPairs.push(i, i + cols); }
          if (c + 1 < cols && r + 1 < rows) { tmpPairs.push(i, i + cols + 1); }
          if (c > 0 && r + 1 < rows) { tmpPairs.push(i, i + cols - 1); }
        }
      }
      neighborPairs = new Uint32Array(tmpPairs);
      pairCount = tmpPairs.length / 2;

      // Init rover
      const roverCx = w * 0.28, roverCy = h * 0.5;
      const roverR = Math.min(w, h) * 0.18;
      roverWaypoints = makeRoverWaypoints(roverCx, roverCy, roverR);
      roverPos = { ...roverWaypoints[0] };
      roverWpIdx = 0;
      roverProgress = 0;
      roverTrail = [];

      waves = [];
      lastEdgeWave = 0;
    }

    function spawnEdgeWave() {
      const side = Math.floor(Math.random() * 4);
      let cx: number, cy: number;
      switch (side) {
        case 0: cx = Math.random() * w; cy = -50; break;
        case 1: cx = w + 50; cy = Math.random() * h; break;
        case 2: cx = Math.random() * w; cy = h + 50; break;
        default: cx = -50; cy = Math.random() * h; break;
      }
      waves.push({ cx, cy, radius: 0 });
    }

    function tick(now: number) {
      if (!lastTime) { lastTime = now; lastEdgeWave = now; }
      const dt = Math.min((now - lastTime) / 1000, 0.05);
      lastTime = now;
      const t = now / 1000;

      // --- Edge waves ---
      if (now - lastEdgeWave > 4500) { spawnEdgeWave(); lastEdgeWave = now; }
      const maxDist = Math.sqrt(w * w + h * h) + WAVE_WIDTH;
      for (let i = waves.length - 1; i >= 0; i--) {
        waves[i].radius += WAVE_SPEED * dt;
        if (waves[i].radius > maxDist) waves.splice(i, 1);
      }

      // --- Rover movement ---
      if (roverWaypoints.length > 1) {
        const from = roverWaypoints[roverWpIdx];
        const to = roverWaypoints[(roverWpIdx + 1) % roverWaypoints.length];
        const dx = to.x - from.x, dy = to.y - from.y;
        const segLen = Math.sqrt(dx * dx + dy * dy);
        roverProgress += (ROVER_SPEED * dt) / segLen;

        if (roverProgress >= 1) {
          roverProgress -= 1;
          roverWpIdx = (roverWpIdx + 1) % roverWaypoints.length;
        }

        const f = roverWaypoints[roverWpIdx];
        const t2 = roverWaypoints[(roverWpIdx + 1) % roverWaypoints.length];
        roverPos.x = f.x + (t2.x - f.x) * roverProgress;
        roverPos.y = f.y + (t2.y - f.y) * roverProgress;

        // Add trail point
        roverTrail.push({ x: roverPos.x, y: roverPos.y, age: 0 });
        // Age & cull trail
        for (let i = roverTrail.length - 1; i >= 0; i--) {
          roverTrail[i].age += dt;
          if (roverTrail[i].age > 12) roverTrail.splice(i, 1);
        }
      }

      // --- vcad wireframe ---
      const vcadCx = w * 0.72, vcadCy = h * 0.5;
      const vcadScale = Math.min(w, h) * 0.1;
      // Morph between cube and cylinder over 10s cycle
      const morphCycle = t * 0.1;
      const morphT = (Math.sin(morphCycle * Math.PI * 2) + 1) / 2; // 0→1→0
      const morphedVerts = lerpShapes(CUBE_VERTS, CYL.verts, morphT);
      const edges = morphT < 0.5 ? CUBE_EDGES : CYL.edges;

      const rotY = t * 0.4;
      const rotX = 0.35;
      const projectedVerts = morphedVerts.map(v => {
        const r1 = rotateY(v, rotY);
        const r2 = rotateX(r1, rotX);
        return project(r2, vcadCx, vcadCy, vcadScale);
      });

      // --- Update dot activations ---
      const mouseR = 140, mouseR2 = mouseR * mouseR;
      const roverR2 = 50 * 50;
      const trailR2 = 30 * 30;

      for (let i = 0; i < count; i++) {
        activations[i] = Math.max(0, activations[i] - DECAY * dt);
        const x = xs[i], y = ys[i];

        // Gentle ambient sine field (very subtle)
        const ambient = Math.sin(x * 0.008 + t * 0.5) * Math.sin(y * 0.008 + t * 0.3);
        if (ambient > 0.5) {
          activations[i] = Math.min(1, activations[i] + (ambient - 0.5) * 0.08);
        }

        // Edge waves
        for (const wave of waves) {
          const dx = x - wave.cx, dy = y - wave.cy;
          const dist = Math.sqrt(dx * dx + dy * dy);
          const wd = Math.abs(dist - wave.radius);
          if (wd < WAVE_WIDTH) {
            activations[i] = Math.min(1, activations[i] + (1 - wd / WAVE_WIDTH) * 0.25);
          }
        }

        // Rover proximity — bright near the rover
        {
          const dx = x - roverPos.x, dy = y - roverPos.y;
          const d2 = dx * dx + dy * dy;
          if (d2 < roverR2) {
            activations[i] = Math.min(1, activations[i] + (1 - Math.sqrt(d2) / 50) * 0.8);
          }
        }

        // Rover trail — warm glow on cleared path
        for (const tp of roverTrail) {
          const dx = x - tp.x, dy = y - tp.y;
          const d2 = dx * dx + dy * dy;
          if (d2 < trailR2) {
            const fade = Math.max(0, 1 - tp.age / 12);
            activations[i] = Math.min(1, activations[i] + (1 - Math.sqrt(d2) / 30) * fade * 0.3);
          }
        }

        // vcad wireframe proximity — glow near edges
        for (const [a, b] of edges) {
          if (a >= projectedVerts.length || b >= projectedVerts.length) continue;
          const pa = projectedVerts[a], pb = projectedVerts[b];
          // Point-to-segment distance
          const ex = pb.x - pa.x, ey = pb.y - pa.y;
          const len2 = ex * ex + ey * ey;
          if (len2 === 0) continue;
          const tt = Math.max(0, Math.min(1, ((x - pa.x) * ex + (y - pa.y) * ey) / len2));
          const px = pa.x + tt * ex, py = pa.y + tt * ey;
          const dx = x - px, dy = y - py;
          const d = Math.sqrt(dx * dx + dy * dy);
          if (d < 40) {
            activations[i] = Math.min(1, activations[i] + (1 - d / 40) * 0.5);
          }
        }

        // Mouse
        if (!prefersReduced) {
          const dx = x - mouse.x, dy = y - mouse.y;
          const d2 = dx * dx + dy * dy;
          if (d2 < mouseR2) {
            activations[i] = Math.min(1, activations[i] + (1 - Math.sqrt(d2) / mouseR) * 0.5);
          }
        }
      }

      // --- Draw ---
      ctx!.clearRect(0, 0, w, h);

      // Connections
      for (let b = 1; b <= 4; b++) {
        const lo = (b - 1) * 0.125, hi = b * 0.125;
        ctx!.beginPath();
        let any = false;
        for (let p = 0; p < pairCount; p++) {
          const i = neighborPairs[p * 2], j = neighborPairs[p * 2 + 1];
          const ai = activations[i], aj = activations[j];
          if (ai < 0.1 || aj < 0.1) continue;
          const alpha = Math.min(ai, aj) * 0.15;
          if (alpha < lo || alpha >= hi) continue;
          ctx!.moveTo(xs[i], ys[i]);
          ctx!.lineTo(xs[j], ys[j]);
          any = true;
        }
        if (any) {
          const mid = (lo + hi) / 2;
          const tt = Math.min(1, mid / 0.1);
          const r = Math.round(WR + (OR - WR) * tt);
          const g = Math.round(WG + (OG - WG) * tt);
          const bv = Math.round(WB + (OB - WB) * tt);
          ctx!.strokeStyle = `rgba(${r},${g},${bv},${mid})`;
          ctx!.lineWidth = 0.5;
          ctx!.stroke();
        }
      }

      // Inactive dots
      ctx!.beginPath();
      for (let i = 0; i < count; i++) {
        if (activations[i] >= 0.08) continue;
        ctx!.moveTo(xs[i] + DOT_R, ys[i]);
        ctx!.arc(xs[i], ys[i], DOT_R, 0, Math.PI * 2);
      }
      ctx!.fillStyle = `rgba(${WR},${WG},${WB},${BASE_ALPHA})`;
      ctx!.fill();

      // Active dots
      for (let b = 1; b <= 4; b++) {
        const lo = (b - 1) * 0.25, hi = b * 0.25;
        ctx!.beginPath();
        let any = false;
        for (let i = 0; i < count; i++) {
          const a = activations[i];
          if (a < 0.08 || a < lo || a >= hi) continue;
          const radius = DOT_R + a * 1.0;
          ctx!.moveTo(xs[i] + radius, ys[i]);
          ctx!.arc(xs[i], ys[i], radius, 0, Math.PI * 2);
          any = true;
        }
        if (!any) continue;
        const mid = (lo + hi) / 2;
        const r = Math.round(WR + (OR - WR) * mid);
        const g = Math.round(WG + (OG - WG) * mid);
        const bv = Math.round(WB + (OB - WB) * mid);
        ctx!.fillStyle = `rgba(${r},${g},${bv},${BASE_ALPHA + mid * 0.55})`;
        ctx!.fill();
      }

      // vcad wireframe — draw actual edges (thin, orange, low opacity)
      ctx!.beginPath();
      for (const [a, b] of edges) {
        if (a >= projectedVerts.length || b >= projectedVerts.length) continue;
        ctx!.moveTo(projectedVerts[a].x, projectedVerts[a].y);
        ctx!.lineTo(projectedVerts[b].x, projectedVerts[b].y);
      }
      ctx!.strokeStyle = `rgba(${OR},${OG},${OB},0.08)`;
      ctx!.lineWidth = 1;
      ctx!.stroke();

      // Rover head — bright orange dot at rover position
      ctx!.beginPath();
      ctx!.arc(roverPos.x, roverPos.y, 3, 0, Math.PI * 2);
      ctx!.fillStyle = `rgba(${OR},${OG},${OB},0.5)`;
      ctx!.fill();

      animId = requestAnimationFrame(tick);
    }

    function onMouse(e: MouseEvent) { mouse.x = e.clientX; mouse.y = e.clientY; }
    function onMouseLeave() { mouse.x = -9999; mouse.y = -9999; }
    function onTouch(e: TouchEvent) {
      if (e.touches.length) { mouse.x = e.touches[0].clientX; mouse.y = e.touches[0].clientY; }
    }

    resize();
    window.addEventListener("resize", resize);
    window.addEventListener("mousemove", onMouse);
    window.addEventListener("mouseleave", onMouseLeave);
    window.addEventListener("touchmove", onTouch, { passive: true });

    if (prefersReduced) {
      tick(performance.now());
    } else {
      animId = requestAnimationFrame(tick);
    }

    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener("resize", resize);
      window.removeEventListener("mousemove", onMouse);
      window.removeEventListener("mouseleave", onMouseLeave);
      window.removeEventListener("touchmove", onTouch);
    };
  }, []);

  return <canvas ref={canvasRef} className="hero-canvas" aria-hidden="true" />;
}
