import { useRef, useEffect } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useConsoleStore } from "@/store";
import { getObstacleData, getObstacleVersion } from "@/lib/obstacleStore";
import type { DecodedObstacle } from "@/lib/protocol";

/**
 * Renders detected obstacles as wireframe 3D bounding boxes with
 * class-specific colors and floating labels.
 *
 * Obstacle classes (from firmware heuristics):
 *   0 = Unknown  → gray
 *   1 = Pole     → cyan
 *   2 = Vehicle  → red
 *   3 = Pedestrian → yellow
 *   4 = Wall     → blue
 *   5 = Debris   → orange
 *
 * Coordinate mapping (same as CostmapOverlay / RoverModel):
 *   Three.js X = -world_Y
 *   Three.js Z = -world_X
 *   Three.js Y = height (vertical)
 */

/** Maximum number of obstacle boxes to render. */
const MAX_BOXES = 64;

/** Class-specific colors. Index = ObstacleClass enum value. */
const CLASS_COLORS: number[] = [
  0x9ca3af, // 0: Unknown  — gray
  0x22d3ee, // 1: Pole     — cyan
  0xef4444, // 2: Vehicle  — red
  0xfacc15, // 3: Pedestrian — yellow
  0x3b82f6, // 4: Wall     — blue
  0xfb923c, // 5: Debris   — orange
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

/** Create a canvas-based text sprite for a label. */
function createLabelSprite(text: string, color: number): THREE.Sprite {
  const canvas = document.createElement("canvas");
  canvas.width = 128;
  canvas.height = 32;
  const ctx = canvas.getContext("2d")!;

  // Background
  ctx.fillStyle = "rgba(0, 0, 0, 0.6)";
  ctx.roundRect(0, 0, 128, 32, 4);
  ctx.fill();

  // Text
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

interface BoxEntry {
  wireframe: THREE.LineSegments;
  label: THREE.Sprite;
  /** Last class assigned (to avoid unnecessary label redraws). */
  lastClass: number;
}

export function ObstacleOverlay() {
  const obstaclesEnabled = useConsoleStore((s) => s.obstaclesEnabled);

  const groupRef = useRef<THREE.Group>(null);
  const lastVersionRef = useRef<number>(0);
  const poolRef = useRef<BoxEntry[]>([]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      for (const entry of poolRef.current) {
        entry.wireframe.geometry.dispose();
        (entry.wireframe.material as THREE.Material).dispose();
        (entry.label.material as THREE.SpriteMaterial).map?.dispose();
        (entry.label.material as THREE.SpriteMaterial).dispose();
      }
      poolRef.current = [];
    };
  }, []);

  useFrame(() => {
    if (!obstaclesEnabled || !groupRef.current) {
      if (groupRef.current) {
        /* eslint-disable react-hooks/immutability -- imperative Three.js pool updates in frame loop */
        for (const entry of poolRef.current) {
          entry.wireframe.visible = false;
          entry.label.visible = false;
        }
        /* eslint-enable react-hooks/immutability */
      }
      return;
    }

    const version = getObstacleVersion();
    if (version === lastVersionRef.current) return;
    lastVersionRef.current = version;

    const obstacles: DecodedObstacle[] = getObstacleData();
    const group = groupRef.current;
    const pool = poolRef.current;

    // Grow pool if needed
    while (pool.length < Math.min(obstacles.length, MAX_BOXES)) {
      const boxGeo = new THREE.BoxGeometry(1, 1, 1);
      const edgesGeo = new THREE.EdgesGeometry(boxGeo);
      const material = new THREE.LineBasicMaterial({
        color: CLASS_COLORS[0],
        linewidth: 1,
        transparent: true,
        opacity: 0.8,
      });
      const wireframe = new THREE.LineSegments(edgesGeo, material);
      wireframe.visible = false;

      const label = createLabelSprite("?", CLASS_COLORS[0]);
      label.visible = false;

      group.add(wireframe);
      group.add(label);
      pool.push({ wireframe, label, lastClass: -1 });
    }

    // Update visible boxes
    const count = Math.min(obstacles.length, MAX_BOXES);
    for (let i = 0; i < count; i++) {
      const obs = obstacles[i];
      const entry = pool[i];

      const worldW = obs.bboxMaxX - obs.bboxMinX;
      const worldH = obs.bboxMaxY - obs.bboxMinY;
      const height = estimateHeight(obs.area);
      const cx = obs.centroidX;
      const cy = obs.centroidY;

      // Position wireframe (Three.js X = -world_Y, Z = -world_X, Y = up)
      entry.wireframe.position.set(-cy, height / 2, -cx);
      entry.wireframe.scale.set(worldH, height, worldW);
      entry.wireframe.visible = true;

      // Update color if class changed
      const classId = obs.obstacleClass;
      const color = getClassColor(classId);
      if (entry.lastClass !== classId) {
        (entry.wireframe.material as THREE.LineBasicMaterial).color.setHex(color);
        updateLabelSprite(entry.label, getClassName(classId), color);
        entry.lastClass = classId;
      }

      // Position label above the box
      entry.label.position.set(-cy, height + 0.15, -cx);
      entry.label.visible = true;
    }

    // Hide excess pool entries
    for (let i = count; i < pool.length; i++) {
      pool[i].wireframe.visible = false;
      pool[i].label.visible = false;
    }
  });

  return <group ref={groupRef} />;
}
