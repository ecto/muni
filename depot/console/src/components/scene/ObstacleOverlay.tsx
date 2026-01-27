import { useRef, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";
import { getInterpolatedObstacles } from "@/lib/obstacleStore";
import type { InterpolatedObstacle } from "@/lib/obstacleInterpolation";

/**
 * Renders detected obstacles as class-specific 3D shapes with
 * floating labels.
 *
 * Obstacle classes (from firmware heuristics):
 *   0 = Unknown    -> wireframe box only (no solid)
 *   1 = Pole       -> cylinder
 *   2 = Vehicle    -> solid box
 *   3 = Pedestrian -> capsule (pill)
 *   4 = Wall       -> solid box (flat)
 *   5 = Debris     -> icosahedron (rock-like)
 *
 * Coordinate mapping (same as CostmapOverlay / RoverModel):
 *   Three.js X = -world_Y
 *   Three.js Z = -world_X
 *   Three.js Y = height (vertical)
 */

/** Maximum number of obstacle shapes to render. */
const MAX_BOXES = 64;

/** Class-specific colors. Index = ObstacleClass enum value. */
const CLASS_COLORS: number[] = [
  0x9ca3af, // 0: Unknown  - gray
  0x22d3ee, // 1: Pole     - cyan
  0xef4444, // 2: Vehicle  - red
  0xfacc15, // 3: Pedestrian - yellow
  0x3b82f6, // 4: Wall     - blue
  0xfb923c, // 5: Debris   - orange
];

/** Class names for labels. */
const CLASS_NAMES: string[] = [
  "?",
  "Pole",
  "Vehicle",
  "Person",
  "Wall",
  "Debris",
];

// Geometry type indices
const GEO_BOX = 0;
const GEO_CYLINDER = 1;
const GEO_CAPSULE = 2;
const GEO_ICOSAHEDRON = 3;

/** Map obstacle class -> geometry type. */
const CLASS_GEO: number[] = [
  GEO_BOX,         // 0: Unknown
  GEO_CYLINDER,    // 1: Pole
  GEO_BOX,         // 2: Vehicle
  GEO_CAPSULE,     // 3: Pedestrian
  GEO_BOX,         // 4: Wall
  GEO_ICOSAHEDRON, // 5: Debris
];

/**
 * Natural Y-axis size of each unit geometry (for correct height scaling).
 * Box=1, Cylinder=1, Capsule=1.5 (radius 0.5 + length 0.5), Icosahedron=1
 */
const GEO_NATURAL_Y: number[] = [1.0, 1.0, 1.5, 1.0];

function getClassColor(classId: number): number {
  return CLASS_COLORS[classId] ?? CLASS_COLORS[0];
}

function getClassName(classId: number): string {
  return CLASS_NAMES[classId] ?? CLASS_NAMES[0];
}

/** Estimated obstacle height when we only have 2D area. */
function estimateHeight(area: number): number {
  if (area > 2.0) return 1.8; // large obstacle (vehicle-sized)
  if (area > 0.5) return 1.2; // medium obstacle
  return 0.6; // small obstacle
}

/**
 * Recover principal/cross axis lengths from axis-aligned bbox and PCA rotation.
 *
 * The axis-aligned bbox of a rotated rectangle is:
 *   bboxW = L·|cos θ| + C·|sin θ|
 *   bboxH = L·|sin θ| + C·|cos θ|
 * Inverting this 2×2 system gives the true extents (L, C).
 */
function unrotateExtents(
  bboxW: number,
  bboxH: number,
  rotation: number,
  area: number,
): [number, number] {
  const cosA = Math.abs(Math.cos(rotation));
  const sinA = Math.abs(Math.sin(rotation));
  const cos2theta = cosA * cosA - sinA * sinA; // cos(2θ)

  if (Math.abs(cos2theta) > 0.1) {
    // Well-conditioned: solve linear system
    const principal = Math.max(0.1, (bboxW * cosA - bboxH * sinA) / cos2theta);
    const cross = Math.max(0.1, (bboxH * cosA - bboxW * sinA) / cos2theta);
    return [principal, cross];
  }

  // Near 45°: cos(2θ) ≈ 0, system is ill-conditioned.
  // Use L + C ≈ (bboxW + bboxH) / (|cosθ| + |sinθ|) and L·C ≈ area.
  const sum = (bboxW + bboxH) / (cosA + sinA);
  const disc = sum * sum - 4 * area;
  if (disc > 0) {
    const sqrtDisc = Math.sqrt(disc);
    return [
      Math.max(0.1, (sum + sqrtDisc) / 2),
      Math.max(0.1, (sum - sqrtDisc) / 2),
    ];
  }
  return [sum / 2, sum / 2];
}

// ============================================================================
// Label helpers
// ============================================================================

/** Create a canvas-based text sprite for a label. */
function createLabelSprite(text: string, color: number): THREE.Sprite {
  const canvas = document.createElement("canvas");
  canvas.width = 128;
  canvas.height = 32;
  const ctx = canvas.getContext("2d")!;

  ctx.fillStyle = "rgba(0, 0, 0, 0.6)";
  ctx.roundRect(0, 0, 128, 32, 4);
  ctx.fill();

  const hex = "#" + color.toString(16).padStart(6, "0");
  ctx.fillStyle = hex;
  ctx.font = "bold 18px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(text, 64, 16);

  const texture = new THREE.CanvasTexture(canvas);
  texture.minFilter = THREE.LinearFilter;
  const material = new THREE.SpriteMaterial({
    map: texture,
    transparent: true,
    depthTest: false,
  });
  const sprite = new THREE.Sprite(material);
  sprite.scale.set(0.8, 0.2, 1);
  return sprite;
}

/** Update label sprite text and color without allocating a new sprite. */
function updateLabelSprite(sprite: THREE.Sprite, text: string, color: number) {
  const material = sprite.material as THREE.SpriteMaterial;
  const texture = material.map as THREE.CanvasTexture;
  const canvas = texture.image as HTMLCanvasElement;
  const ctx = canvas.getContext("2d")!;

  ctx.clearRect(0, 0, 128, 32);
  ctx.fillStyle = "rgba(0, 0, 0, 0.6)";
  ctx.roundRect(0, 0, 128, 32, 4);
  ctx.fill();

  const hex = "#" + color.toString(16).padStart(6, "0");
  ctx.fillStyle = hex;
  ctx.font = "bold 18px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(text, 64, 16);

  texture.needsUpdate = true;
}

// ============================================================================
// Shared geometry / material resources (created once, shared across pool)
// ============================================================================

interface SharedResources {
  solidGeos: THREE.BufferGeometry[];
  edgesGeos: THREE.EdgesGeometry[];
  solidMats: THREE.MeshStandardMaterial[];
  edgeMats: THREE.LineBasicMaterial[];
}

function createSharedResources(): SharedResources {
  const box = new THREE.BoxGeometry(1, 1, 1);
  const cylinder = new THREE.CylinderGeometry(0.5, 0.5, 1, 16);
  const capsule = new THREE.CapsuleGeometry(0.5, 0.5, 4, 16);
  const icosahedron = new THREE.IcosahedronGeometry(0.5, 1);
  const solidGeos: THREE.BufferGeometry[] = [box, cylinder, capsule, icosahedron];

  const edgesGeos = solidGeos.map((g) => new THREE.EdgesGeometry(g));

  const solidMats = CLASS_COLORS.map(
    (c) =>
      new THREE.MeshStandardMaterial({
        color: c,
        emissive: c,
        emissiveIntensity: 0.3,
        transparent: true,
        opacity: 0.35,
        depthWrite: false,
        side: THREE.DoubleSide,
      }),
  );

  const edgeMats = CLASS_COLORS.map(
    (c) =>
      new THREE.LineBasicMaterial({
        color: c,
        transparent: true,
        opacity: 0.9,
      }),
  );

  return { solidGeos, edgesGeos, solidMats, edgeMats };
}

function disposeSharedResources(res: SharedResources) {
  for (const g of res.solidGeos) g.dispose();
  for (const g of res.edgesGeos) g.dispose();
  for (const m of res.solidMats) m.dispose();
  for (const m of res.edgeMats) m.dispose();
}

// ============================================================================
// Pool entry
// ============================================================================

interface PoolEntry {
  solidMesh: THREE.Mesh;
  wireframe: THREE.LineSegments;
  label: THREE.Sprite;
  /** Per-entry cloned materials for independent opacity. */
  solidMat: THREE.MeshStandardMaterial;
  edgeMat: THREE.LineBasicMaterial;
  /** Last class assigned (to avoid unnecessary geometry/material swaps). */
  lastClass: number;
  /** Track ID currently assigned to this pool entry. */
  lastTrackId: number;
}

// ============================================================================
// Component
// ============================================================================

export function ObstacleOverlay() {
  const obstaclesEnabled = useConsoleStore((s) => s.obstaclesEnabled);

  const groupRef = useRef<THREE.Group>(null);
  const poolRef = useRef<PoolEntry[]>([]);
  const sharedRef = useRef<SharedResources | null>(null);
  /** Map trackId -> pool index for stable assignment. */
  const trackPoolRef = useRef<Map<number, number>>(new Map());

  // Create shared resources on mount, dispose everything on unmount
  useEffect(() => {
    sharedRef.current = createSharedResources();
    return () => {
      if (sharedRef.current) {
        disposeSharedResources(sharedRef.current);
        sharedRef.current = null;
      }
      for (const entry of poolRef.current) {
        entry.solidMat.dispose();
        entry.edgeMat.dispose();
        (entry.label.material as THREE.SpriteMaterial).map?.dispose();
        (entry.label.material as THREE.SpriteMaterial).dispose();
      }
      poolRef.current = [];
      trackPoolRef.current.clear();
    };
  }, []);

  useFrame(() => {
    if (!obstaclesEnabled || !groupRef.current || !sharedRef.current) {
      if (groupRef.current) {
        /* eslint-disable react-hooks/immutability -- imperative Three.js pool updates in frame loop */
        for (const entry of poolRef.current) {
          entry.solidMesh.visible = false;
          entry.wireframe.visible = false;
          entry.label.visible = false;
        }
        /* eslint-enable react-hooks/immutability */
      }
      return;
    }

    // Interpolation produces different values each frame — no version check needed
    const obstacles: InterpolatedObstacle[] = getInterpolatedObstacles();
    const group = groupRef.current;
    const pool = poolRef.current;
    const shared = sharedRef.current;
    const trackPool = trackPoolRef.current;

    // Grow pool if needed
    while (pool.length < Math.min(obstacles.length, MAX_BOXES)) {
      // Clone materials per entry for independent opacity
      const solidMat = shared.solidMats[0].clone();
      const edgeMat = shared.edgeMats[0].clone();

      const solidMesh = new THREE.Mesh(
        shared.solidGeos[GEO_BOX],
        solidMat,
      );
      solidMesh.visible = false;

      const wireframe = new THREE.LineSegments(
        shared.edgesGeos[GEO_BOX],
        edgeMat,
      );
      wireframe.visible = false;
      wireframe.renderOrder = 1;

      const label = createLabelSprite("?", CLASS_COLORS[0]);
      label.visible = false;

      group.add(solidMesh);
      group.add(wireframe);
      group.add(label);
      pool.push({ solidMesh, wireframe, label, solidMat, edgeMat, lastClass: -1, lastTrackId: -1 });
    }

    // Build trackId -> obstacle index for stable pool assignment
    const usedPoolIndices = new Set<number>();
    const obstacleAssignment: number[] = new Array(obstacles.length);

    // First pass: reuse existing pool entries for known trackIds
    for (let i = 0; i < Math.min(obstacles.length, MAX_BOXES); i++) {
      const trackId = obstacles[i].trackId;
      const existingIdx = trackPool.get(trackId);
      if (existingIdx !== undefined && existingIdx < pool.length && !usedPoolIndices.has(existingIdx)) {
        obstacleAssignment[i] = existingIdx;
        usedPoolIndices.add(existingIdx);
      } else {
        obstacleAssignment[i] = -1; // needs assignment
      }
    }

    // Second pass: assign unmatched obstacles to unused pool entries
    let nextFreePool = 0;
    for (let i = 0; i < Math.min(obstacles.length, MAX_BOXES); i++) {
      if (obstacleAssignment[i] !== -1) continue;
      while (nextFreePool < pool.length && usedPoolIndices.has(nextFreePool)) {
        nextFreePool++;
      }
      if (nextFreePool < pool.length) {
        obstacleAssignment[i] = nextFreePool;
        usedPoolIndices.add(nextFreePool);
        nextFreePool++;
      }
    }

    // Update trackPool map
    trackPool.clear();
    const count = Math.min(obstacles.length, MAX_BOXES);
    for (let i = 0; i < count; i++) {
      const poolIdx = obstacleAssignment[i];
      if (poolIdx === -1 || poolIdx === undefined) continue;
      trackPool.set(obstacles[i].trackId, poolIdx);
    }

    // Update visible entries
    for (let i = 0; i < count; i++) {
      const obs = obstacles[i];
      const poolIdx = obstacleAssignment[i];
      if (poolIdx === -1 || poolIdx === undefined || poolIdx >= pool.length) continue;
      const entry = pool[poolIdx];

      const worldW = obs.bboxMaxX - obs.bboxMinX;
      const worldH = obs.bboxMaxY - obs.bboxMinY;
      const cx = obs.centroidX;
      const cy = obs.centroidY;

      // Use real height data when available, fall back to estimate
      const hasHeight = obs.maxZ > obs.minZ + 0.01;
      const rawHeight = hasHeight
        ? obs.maxZ - obs.minZ
        : estimateHeight(obs.area);
      const baseZ = hasHeight ? obs.minZ : 0;

      // Swap geometry/material on class change
      const classId = obs.obstacleClass;
      if (entry.lastClass !== classId) {
        const geoType = CLASS_GEO[classId] ?? GEO_BOX;
        entry.solidMesh.geometry = shared.solidGeos[geoType];
        entry.wireframe.geometry = shared.edgesGeos[geoType];

        // Update cloned material colors
        const color = getClassColor(classId);
        entry.solidMat.color.setHex(color);
        entry.solidMat.emissive.setHex(color);
        entry.edgeMat.color.setHex(color);

        updateLabelSprite(
          entry.label,
          getClassName(classId),
          color,
        );
        entry.lastClass = classId;
      }

      // Apply interpolated opacity
      const opacity = obs.opacity;
      entry.solidMat.opacity = 0.35 * opacity;
      entry.edgeMat.opacity = 0.9 * opacity;

      // Class-specific scaling
      const geoType = CLASS_GEO[classId] ?? GEO_BOX;
      const natY = GEO_NATURAL_Y[geoType];
      let sx: number, sy: number, sz: number, renderHeight: number;
      // PCA rotation for box-shaped obstacles (world angle → Three.js Y rotation)
      let rotY = 0;

      switch (classId) {
        case 1: {
          // Pole - cylinder; diameter from smaller bbox dimension
          const diameter = Math.min(worldW, worldH);
          renderHeight = Math.max(rawHeight, 1.5);
          sx = diameter;
          sy = renderHeight / natY;
          sz = diameter;
          break;
        }
        case 2: {
          // Vehicle - solid box; height >= 1.4m
          const [vLen, vCross] = unrotateExtents(worldW, worldH, obs.rotation, obs.area);
          renderHeight = Math.max(rawHeight, 1.4);
          sx = vCross;
          sy = renderHeight / natY;
          sz = vLen;
          rotY = obs.rotation;
          break;
        }
        case 3: {
          // Pedestrian - capsule; height >= 1.0m
          const bodyWidth = Math.max(worldW, worldH);
          renderHeight = Math.max(rawHeight, 1.0);
          sx = bodyWidth;
          sy = renderHeight / natY;
          sz = bodyWidth;
          break;
        }
        case 4: {
          // Wall - solid box; height <= 1.0m
          const [wLen, wCross] = unrotateExtents(worldW, worldH, obs.rotation, obs.area);
          renderHeight = Math.min(rawHeight, 1.0);
          sx = wCross;
          sy = renderHeight / natY;
          sz = wLen;
          rotY = obs.rotation;
          break;
        }
        case 5: {
          // Debris - icosahedron; height <= 0.8m
          const debrisSize = Math.max(worldW, worldH);
          renderHeight = Math.min(rawHeight, 0.8);
          sx = debrisSize;
          sy = renderHeight / natY;
          sz = debrisSize;
          break;
        }
        default: {
          // Unknown (0) - wireframe box only
          const [uLen, uCross] = unrotateExtents(worldW, worldH, obs.rotation, obs.area);
          renderHeight = rawHeight;
          sx = uCross;
          sy = renderHeight / natY;
          sz = uLen;
          rotY = obs.rotation;
          break;
        }
      }

      // Position: Three.js X = -world_Y, Z = -world_X, Y = up
      const posX = -cy;
      const posY = baseZ + renderHeight / 2;
      const posZ = -cx;

      entry.solidMesh.position.set(posX, posY, posZ);
      entry.solidMesh.scale.set(sx, sy, sz);
      entry.solidMesh.rotation.set(0, rotY, 0);
      entry.solidMesh.visible = classId !== 0 && opacity > 0.01;

      entry.wireframe.position.set(posX, posY, posZ);
      entry.wireframe.scale.set(sx, sy, sz);
      entry.wireframe.rotation.set(0, rotY, 0);
      entry.wireframe.visible = opacity > 0.01;

      entry.label.position.set(posX, baseZ + renderHeight + 0.15, posZ);
      entry.label.visible = opacity > 0.01;

      entry.lastTrackId = obs.trackId;
    }

    // Hide excess pool entries
    for (let i = 0; i < pool.length; i++) {
      if (!usedPoolIndices.has(i)) {
        pool[i].solidMesh.visible = false;
        pool[i].wireframe.visible = false;
        pool[i].label.visible = false;
      }
    }
  });

  return <group ref={groupRef} />;
}
