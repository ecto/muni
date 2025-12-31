import { Grid } from "@react-three/drei";

export function Ground() {
  return (
    <>
      {/* Ground plane */}
      <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, 0, 0]} receiveShadow>
        <planeGeometry args={[100, 100]} />
        <meshStandardMaterial color="#1c1917" />
      </mesh>

      {/* Grid overlay */}
      <Grid
        position={[0, 0.001, 0]}
        args={[100, 100]}
        cellSize={1}
        cellThickness={0.5}
        cellColor="#292524"
        sectionSize={5}
        sectionThickness={1}
        sectionColor="#44403c"
        fadeDistance={50}
        fadeStrength={1}
        followCamera={false}
        infiniteGrid={false}
      />
    </>
  );
}
