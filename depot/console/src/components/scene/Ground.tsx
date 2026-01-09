import { Grid } from "@react-three/drei";

/**
 * Ground plane with infinite grid overlay.
 *
 * Grid styling is consistent with the web CAD viewer (web/viewer.html).
 * Colors use the stone palette: #1c1917 ground, #292524 cells, #44403c sections.
 */
export function Ground() {
  return (
    <Grid
      position={[0, 0, 0]}
      args={[100, 100]}
      cellSize={1}
      cellThickness={0.5}
      cellColor="#292524"
      sectionSize={5}
      sectionThickness={1}
      sectionColor="#44403c"
      fadeDistance={50}
      fadeStrength={1.5}
      followCamera
      infiniteGrid
    />
  );
}
