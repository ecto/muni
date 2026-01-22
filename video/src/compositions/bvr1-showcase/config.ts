// BVR1 Showcase video configuration

export const SHOWCASE_CONFIG = {
  // Video specs
  fps: 30,
  width: 1920,
  height: 1080,
  totalFrames: 1500, // 50 seconds

  // Scene timings (in frames)
  scenes: {
    heroReveal: { start: 0, end: 240 }, // 8s
    orbitalRotation: { start: 240, end: 540 }, // 10s
    scaleComparison: { start: 540, end: 780 }, // 8s
    componentCallouts: { start: 780, end: 1260 }, // 16s
    specsClose: { start: 1260, end: 1500 }, // 8s
  },

  // Camera settings
  camera: {
    defaultDistance: 800,
    defaultElevation: Math.PI / 8,
    defaultTarget: { x: 0, y: 250, z: 0 },
    fov: 45,
  },

  // Model settings
  model: {
    centerY: 250, // Model center height
  },
} as const;

// Component callout timings (within the componentCallouts scene)
// Each component gets 4 seconds (120 frames)
export const CALLOUT_TIMINGS = {
  lidar: { start: 0, end: 120 }, // LiDAR
  camera: { start: 120, end: 240 }, // 360° Camera
  motors: { start: 240, end: 360 }, // Hub Motors
  battery: { start: 360, end: 480 }, // Battery Pack
} as const;

// BVR1 specs for final card
export const BVR1_SPECS = [
  { label: "Dimensions", value: "380 × 600 × 550mm" },
  { label: "Weight", value: "~25 kg" },
  { label: "Motors", value: "4× 500W Hub" },
  { label: "Battery", value: "48V 14Ah (672Wh)" },
  { label: "Compute", value: "Jetson Orin NX" },
  { label: "Sensors", value: "LiDAR + 360° Cam + RTK" },
] as const;
