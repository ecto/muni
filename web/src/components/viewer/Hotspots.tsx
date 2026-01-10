"use client";

import { useMemo, useRef, useEffect } from "react";
import { useThree, useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { bvr1Components, type ComponentInfo } from "./ModelCatalog";

interface HotspotProps {
  component: ComponentInfo;
  isActive: boolean;
  modelBounds: { size: THREE.Vector3; center: THREE.Vector3 } | null;
  onClick: () => void;
}

function createHotspotTexture(number: number, isActive: boolean): THREE.CanvasTexture {
  const size = 128;
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext("2d")!;

  const center = size / 2;
  const radius = size / 2 - 8;

  // Draw circle background
  ctx.beginPath();
  ctx.arc(center, center, radius, 0, Math.PI * 2);

  if (isActive) {
    ctx.fillStyle = "#ff6600";
    ctx.fill();
    ctx.strokeStyle = "#ffffff";
    ctx.lineWidth = 4;
    ctx.stroke();
  } else {
    ctx.fillStyle = "#000000";
    ctx.fill();
    ctx.strokeStyle = "#ff6600";
    ctx.lineWidth = 4;
    ctx.stroke();
  }

  // Draw number
  ctx.font = '700 52px "Berkeley Mono", ui-monospace, monospace';
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = isActive ? "#000000" : "#ffffff";
  ctx.fillText(String(number), center, center + 2);

  const texture = new THREE.CanvasTexture(canvas);
  texture.minFilter = THREE.LinearFilter;
  texture.magFilter = THREE.LinearFilter;
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.generateMipmaps = false;

  return texture;
}

function Hotspot({ component, isActive, modelBounds, onClick }: HotspotProps) {
  const spriteRef = useRef<THREE.Sprite>(null);
  const dotRef = useRef<THREE.Mesh>(null);
  const ringRef = useRef<THREE.Mesh>(null);
  const { camera } = useThree();

  const texture = useMemo(
    () => createHotspotTexture(component.id, isActive),
    [component.id, isActive]
  );

  const anchor = useMemo(
    () => new THREE.Vector3(component.position.x, component.position.y, component.position.z),
    [component.position]
  );

  const labelPosition = useMemo(() => {
    if (!modelBounds) return anchor.clone();

    const labelOffset = Math.max(modelBounds.size.x, modelBounds.size.z) * 0.12 + 50;
    const dir = new THREE.Vector3(component.position.x, 0, component.position.z).normalize();
    if (dir.length() < 0.1) dir.set(1, 0, 0);

    const pos = anchor.clone().add(dir.multiplyScalar(labelOffset));
    pos.y = anchor.y + 30;
    return pos;
  }, [anchor, component.position, modelBounds]);

  const lineObject = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    geo.setFromPoints([labelPosition, anchor]);
    const mat = new THREE.LineBasicMaterial({ color: "#ff6600", transparent: true, opacity: 1.0 });
    return new THREE.Line(geo, mat);
  }, [labelPosition, anchor]);

  // Update dot/ring to face camera
  useFrame(() => {
    if (dotRef.current) {
      dotRef.current.lookAt(camera.position);
    }
    if (ringRef.current) {
      ringRef.current.lookAt(camera.position);
    }
  });

  useEffect(() => {
    return () => {
      texture.dispose();
    };
  }, [texture]);

  return (
    <group>
      {/* Sprite label */}
      <sprite
        ref={spriteRef}
        position={labelPosition}
        scale={[28, 28, 1]}
        renderOrder={999}
        onClick={onClick}
      >
        <spriteMaterial
          map={texture}
          transparent
          depthTest={false}
          sizeAttenuation
          toneMapped={false}
        />
      </sprite>

      {/* Line from label to anchor */}
      <primitive object={lineObject} />

      {/* Anchor ring */}
      <mesh ref={ringRef} position={anchor}>
        <ringGeometry args={[3, 5, 16]} />
        <meshBasicMaterial color="#000000" side={THREE.DoubleSide} />
      </mesh>

      {/* Anchor dot */}
      <mesh ref={dotRef} position={anchor}>
        <circleGeometry args={[3, 16]} />
        <meshBasicMaterial color="#ff6600" side={THREE.DoubleSide} />
      </mesh>
    </group>
  );
}

interface HotspotsProps {
  visible: boolean;
  selectedComponent: ComponentInfo | null;
  modelBounds: { size: THREE.Vector3; center: THREE.Vector3 } | null;
  onSelectComponent: (component: ComponentInfo) => void;
}

export function Hotspots({
  visible,
  selectedComponent,
  modelBounds,
  onSelectComponent,
}: HotspotsProps) {
  if (!visible) return null;

  return (
    <group>
      {bvr1Components.map((component) => (
        <Hotspot
          key={component.id}
          component={component}
          isActive={selectedComponent?.id === component.id}
          modelBounds={modelBounds}
          onClick={() => onSelectComponent(component)}
        />
      ))}
    </group>
  );
}
