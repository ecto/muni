import { Canvas } from "@react-three/fiber";
import { Ground } from "./Ground";
import { RoverModel } from "./RoverModel";
import { CameraController } from "./CameraController";
import { EquirectangularSky } from "./EquirectangularSky";
import { useOperatorStore } from "@/store";

export function Scene() {
  const { videoConnected } = useOperatorStore();

  return (
    <Canvas
      shadows
      camera={{ position: [3, 2.5, 3], fov: 60 }}
      gl={{ antialias: true }}
      className="absolute inset-0"
    >
      {/* 360° video skybox (when connected) */}
      <EquirectangularSky />

      {/* Lighting - reduce when video is active for better visibility */}
      <ambientLight intensity={videoConnected ? 0.5 : 0.3} />
      <directionalLight
        position={[10, 20, 10]}
        intensity={videoConnected ? 0.8 : 1.5}
        castShadow
        shadow-mapSize={[2048, 2048]}
        shadow-camera-far={50}
        shadow-camera-left={-20}
        shadow-camera-right={20}
        shadow-camera-top={20}
        shadow-camera-bottom={-20}
      />

      {/* Scene elements */}
      <Ground />
      <RoverModel />
      <CameraController />

      {/* Fog for depth - disabled when video is active */}
      {!videoConnected && <fog attach="fog" args={["#0c0a09", 20, 80]} />}
    </Canvas>
  );
}
