"use client";

import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useTheme } from "./hooks/useTheme";

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

  void main() {
    float dist = length(vWorldPos.xz);
    float fade = 1.0 - smoothstep(uFadeDistance * 0.3, uFadeDistance, dist);
    vec2 cellFrac = fract(vWorldPos.xz / uCellSize);
    vec2 sectionFrac = fract(vWorldPos.xz / uSectionSize);
    float cellLine = step(cellFrac.x, 0.04) + step(1.0 - cellFrac.x, 0.04) + step(cellFrac.y, 0.04) + step(1.0 - cellFrac.y, 0.04);
    float sectionLine = step(sectionFrac.x, 0.015) + step(1.0 - sectionFrac.x, 0.015) + step(sectionFrac.y, 0.015) + step(1.0 - sectionFrac.y, 0.015);
    vec3 color = uColor1;
    float alpha = min(cellLine, 1.0) * 0.25 * fade;
    if (sectionLine > 0.5) { color = uColor2; alpha = 0.5 * fade; }
    if (alpha < 0.02) discard;
    gl_FragColor = vec4(color, alpha);
  }
`;

export function InfiniteGrid() {
  const meshRef = useRef<THREE.Mesh>(null);
  const theme = useTheme();

  const uniforms = useMemo(
    () => ({
      uCellSize: { value: 10.0 },
      uSectionSize: { value: 100.0 },
      uColor1: { value: new THREE.Color(theme.gridCell) },
      uColor2: { value: new THREE.Color(theme.gridSection) },
      uFadeDistance: { value: 2000.0 },
    }),
    []
  );

  useFrame(() => {
    if (meshRef.current) {
      const material = meshRef.current.material as THREE.ShaderMaterial;
      material.uniforms.uColor1.value.set(theme.gridCell);
      material.uniforms.uColor2.value.set(theme.gridSection);
    }
  });

  return (
    <mesh ref={meshRef} rotation={[-Math.PI / 2, 0, 0]} frustumCulled={false}>
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
