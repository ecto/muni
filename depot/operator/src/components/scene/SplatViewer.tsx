import { useEffect, useRef, useState } from "react";
import { Canvas, useThree, useFrame } from "@react-three/fiber";
import { OrbitControls, PerspectiveCamera } from "@react-three/drei";
import * as THREE from "three";
import { getMapAssetUrl } from "@/hooks/useMaps";
import { ArrowsClockwise, Cube } from "@phosphor-icons/react";

interface PLYPoint {
  position: THREE.Vector3;
  color?: THREE.Color;
  scale?: THREE.Vector3;
  opacity?: number;
}

/**
 * Parse a PLY file into points.
 * Supports both ASCII and binary formats, and both simple point clouds
 * and Gaussian splat format.
 */
async function parsePLY(buffer: ArrayBuffer): Promise<PLYPoint[]> {
  const decoder = new TextDecoder();
  const text = decoder.decode(buffer);
  const lines = text.split("\n");

  let vertexCount = 0;
  let format = "ascii";
  let headerEnd = 0;
  const properties: string[] = [];

  // Parse header
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    if (line.startsWith("element vertex")) {
      vertexCount = parseInt(line.split(" ")[2], 10);
    } else if (line.startsWith("format")) {
      format = line.includes("ascii") ? "ascii" : "binary";
    } else if (line.startsWith("property")) {
      const parts = line.split(" ");
      properties.push(parts[parts.length - 1]);
    } else if (line === "end_header") {
      headerEnd = i + 1;
      break;
    }
  }

  const points: PLYPoint[] = [];

  if (format === "ascii") {
    // Parse ASCII format
    for (let i = headerEnd; i < headerEnd + vertexCount && i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line) continue;

      const values = line.split(/\s+/).map(parseFloat);

      const xIdx = properties.indexOf("x");
      const yIdx = properties.indexOf("y");
      const zIdx = properties.indexOf("z");

      if (xIdx >= 0 && yIdx >= 0 && zIdx >= 0) {
        const point: PLYPoint = {
          position: new THREE.Vector3(
            values[xIdx],
            values[yIdx],
            values[zIdx]
          ),
        };

        // Check for color
        const rIdx = properties.indexOf("red") >= 0 ? properties.indexOf("red") : properties.indexOf("f_dc_0");
        const gIdx = properties.indexOf("green") >= 0 ? properties.indexOf("green") : properties.indexOf("f_dc_1");
        const bIdx = properties.indexOf("blue") >= 0 ? properties.indexOf("blue") : properties.indexOf("f_dc_2");

        if (rIdx >= 0 && gIdx >= 0 && bIdx >= 0) {
          // Normalize color values (could be 0-255 or 0-1)
          const r = values[rIdx] > 1 ? values[rIdx] / 255 : values[rIdx];
          const g = values[gIdx] > 1 ? values[gIdx] / 255 : values[gIdx];
          const b = values[bIdx] > 1 ? values[bIdx] / 255 : values[bIdx];
          point.color = new THREE.Color(r, g, b);
        }

        // Check for opacity
        const opacityIdx = properties.indexOf("opacity");
        if (opacityIdx >= 0) {
          point.opacity = values[opacityIdx];
        }

        points.push(point);
      }
    }
  } else {
    // Binary format would require more complex parsing
    // For now, fall back to treating it as ASCII
    console.warn("Binary PLY format not fully supported, attempting ASCII parse");
  }

  return points;
}

/**
 * Point cloud renderer component
 */
function PointCloud({ points }: { points: PLYPoint[] }) {
  const pointsRef = useRef<THREE.Points>(null);

  const geometry = new THREE.BufferGeometry();

  // Create position buffer
  const positions = new Float32Array(points.length * 3);
  const colors = new Float32Array(points.length * 3);

  for (let i = 0; i < points.length; i++) {
    const point = points[i];
    positions[i * 3] = point.position.x;
    positions[i * 3 + 1] = point.position.y;
    positions[i * 3 + 2] = point.position.z;

    if (point.color) {
      colors[i * 3] = point.color.r;
      colors[i * 3 + 1] = point.color.g;
      colors[i * 3 + 2] = point.color.b;
    } else {
      // Default color based on height
      const h = (point.position.y + 5) / 10; // Normalize to 0-1
      const color = new THREE.Color().setHSL(0.6 - h * 0.4, 0.8, 0.5);
      colors[i * 3] = color.r;
      colors[i * 3 + 1] = color.g;
      colors[i * 3 + 2] = color.b;
    }
  }

  geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));

  return (
    <points ref={pointsRef} geometry={geometry}>
      <pointsMaterial
        size={0.02}
        vertexColors
        sizeAttenuation
        transparent
        opacity={0.8}
      />
    </points>
  );
}

/**
 * Camera auto-fit to point cloud bounds
 */
function CameraFit({ points }: { points: PLYPoint[] }) {
  const { camera } = useThree();

  useEffect(() => {
    if (points.length === 0) return;

    // Calculate bounding box
    const box = new THREE.Box3();
    for (const point of points) {
      box.expandByPoint(point.position);
    }

    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    const maxDim = Math.max(size.x, size.y, size.z);

    // Position camera to see the whole cloud
    camera.position.set(
      center.x + maxDim * 1.5,
      center.y + maxDim * 0.5,
      center.z + maxDim * 1.5
    );
    camera.lookAt(center);
  }, [points, camera]);

  return null;
}

/**
 * Grid helper with auto-sizing
 */
function AutoGrid({ points }: { points: PLYPoint[] }) {
  if (points.length === 0) return null;

  // Calculate grid size from points
  const box = new THREE.Box3();
  for (const point of points) {
    box.expandByPoint(point.position);
  }

  const size = box.getSize(new THREE.Vector3());
  const gridSize = Math.max(size.x, size.z) * 1.5;
  const center = box.getCenter(new THREE.Vector3());

  return (
    <gridHelper
      args={[gridSize, 20, 0x444444, 0x222222]}
      position={[center.x, box.min.y, center.z]}
    />
  );
}

/**
 * Stats display overlay
 */
function StatsOverlay({ points, loading }: { points: PLYPoint[]; loading: boolean }) {
  return (
    <div className="absolute top-4 left-4 bg-black/50 backdrop-blur rounded-lg px-3 py-2 text-sm">
      {loading ? (
        <div className="flex items-center gap-2 text-muted-foreground">
          <ArrowsClockwise className="h-4 w-4 animate-spin" />
          Loading...
        </div>
      ) : (
        <div className="flex items-center gap-2">
          <Cube className="h-4 w-4 text-primary" weight="fill" />
          <span className="font-mono">{points.length.toLocaleString()} points</span>
        </div>
      )}
    </div>
  );
}

interface SplatViewerProps {
  mapId: string;
  className?: string;
}

/**
 * 3D Gaussian Splat / Point Cloud Viewer
 *
 * Loads and displays a PLY file from the map-api service.
 * Supports both simple point clouds and Gaussian splat format.
 */
export function SplatViewer({ mapId, className = "" }: SplatViewerProps) {
  const [points, setPoints] = useState<PLYPoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadSplat = async () => {
      setLoading(true);
      setError(null);

      try {
        const url = getMapAssetUrl(mapId, "splat.ply");
        const response = await fetch(url);

        if (!response.ok) {
          throw new Error(`Failed to load splat: ${response.statusText}`);
        }

        const buffer = await response.arrayBuffer();
        const loadedPoints = await parsePLY(buffer);

        setPoints(loadedPoints);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load");
      } finally {
        setLoading(false);
      }
    };

    loadSplat();
  }, [mapId]);

  if (error) {
    return (
      <div className={`flex items-center justify-center bg-black/20 ${className}`}>
        <div className="text-center text-muted-foreground">
          <Cube className="h-12 w-12 mx-auto mb-2 opacity-50" weight="thin" />
          <p className="text-sm">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`relative ${className}`}>
      <Canvas>
        <PerspectiveCamera makeDefault fov={60} near={0.1} far={1000} />
        <OrbitControls
          enableDamping
          dampingFactor={0.05}
          rotateSpeed={0.5}
          panSpeed={0.5}
          zoomSpeed={0.5}
        />

        {/* Lighting */}
        <ambientLight intensity={0.5} />
        <directionalLight position={[10, 10, 5]} intensity={0.5} />

        {/* Point cloud */}
        {points.length > 0 && (
          <>
            <PointCloud points={points} />
            <CameraFit points={points} />
            <AutoGrid points={points} />
          </>
        )}

        {/* Background */}
        <color attach="background" args={["#1c1917"]} />
      </Canvas>

      <StatsOverlay points={points} loading={loading} />

      {/* Controls hint */}
      <div className="absolute bottom-4 right-4 text-xs text-muted-foreground bg-black/30 px-2 py-1 rounded">
        Drag to rotate · Scroll to zoom · Shift+drag to pan
      </div>
    </div>
  );
}
