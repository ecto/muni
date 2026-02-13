import { useMemo } from "react";
import * as THREE from "three";
import { gridColors, MUNI_ORANGE } from "../../lib/brand";

const vertexShader = `
  varying vec3 vWorldPos;
  void main() {
    vec4 worldPos = modelMatrix * vec4(position, 1.0);
    vWorldPos = worldPos.xyz;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

const fragmentShader = `
  varying vec3 vWorldPos;
  uniform float uCellSize;
  uniform float uSectionSize;
  uniform vec3 uColor1;
  uniform vec3 uColor2;
  uniform float uFadeDistance;
  uniform float uOpacity;

  void main() {
    float dist = length(vWorldPos.xz);
    float fade = 1.0 - smoothstep(uFadeDistance * 0.3, uFadeDistance, dist);
    vec2 cellFrac = fract(vWorldPos.xz / uCellSize);
    vec2 sectionFrac = fract(vWorldPos.xz / uSectionSize);
    float cellLine = step(cellFrac.x, 0.04) + step(1.0 - cellFrac.x, 0.04) + step(cellFrac.y, 0.04) + step(1.0 - cellFrac.y, 0.04);
    float sectionLine = step(sectionFrac.x, 0.015) + step(1.0 - sectionFrac.x, 0.015) + step(sectionFrac.y, 0.015) + step(1.0 - sectionFrac.y, 0.015);
    vec3 color = uColor1;
    float alpha = min(cellLine, 1.0) * 0.25 * fade * uOpacity;
    if (sectionLine > 0.5) { color = uColor2; alpha = 0.5 * fade * uOpacity; }
    if (alpha < 0.02) discard;
    gl_FragColor = vec4(color, alpha);
  }
`;

interface GridProps {
  opacity?: number;
  fadeDistance?: number;
  cellColor?: string;
  sectionColor?: string;
}

export function Grid({ opacity = 1, fadeDistance = 2000, cellColor, sectionColor }: GridProps) {
  const uniforms = useMemo(
    () => ({
      uCellSize: { value: 10.0 },
      uSectionSize: { value: 100.0 },
      uColor1: { value: new THREE.Color(cellColor ?? gridColors.cell) },
      uColor2: { value: new THREE.Color(sectionColor ?? MUNI_ORANGE) },
      uFadeDistance: { value: fadeDistance },
      uOpacity: { value: opacity },
    }),
    [fadeDistance, opacity, cellColor, sectionColor]
  );

  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} frustumCulled={false}>
      <planeGeometry args={[20000, 20000]} />
      <shaderMaterial
        uniforms={uniforms}
        vertexShader={vertexShader}
        fragmentShader={fragmentShader}
        side={THREE.DoubleSide}
        transparent
        depthWrite={false}
      />
    </mesh>
  );
}
