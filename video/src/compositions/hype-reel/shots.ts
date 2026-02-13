// Shot registry — every shot in the video with metadata
// Swap null src for real footage = one line change

export type ShotType = "image" | "video" | "3d" | "splat" | "text" | "black";

export interface KenBurnsConfig {
  startScale: number;
  endScale: number;
  startX: number; // % offset
  startY: number;
  endX: number;
  endY: number;
}

export interface Shot {
  type: ShotType;
  src: string | null; // null = placeholder
  description: string; // shown in placeholder frame
  textOverlay?: string;
  tag?: string; // small label (e.g. "PROBLEM", "SOLUTION")
  kenBurns?: KenBurnsConfig;
  bgColor?: string;
}

const KB_SLOW_ZOOM: KenBurnsConfig = {
  startScale: 1.0,
  endScale: 1.15,
  startX: 0,
  startY: 0,
  endX: 0,
  endY: 0,
};

const KB_PAN_RIGHT: KenBurnsConfig = {
  startScale: 1.1,
  endScale: 1.1,
  startX: -3,
  startY: 0,
  endX: 3,
  endY: 0,
};

const KB_PAN_LEFT: KenBurnsConfig = {
  startScale: 1.1,
  endScale: 1.1,
  startX: 3,
  startY: 0,
  endX: -3,
  endY: 0,
};

/** Shot registry — keyed by CutPoint labels from beat.ts */
export const SHOTS: Record<string, Shot> = {
  // === INTRO BUILD ===
  "black-open": {
    type: "black",
    src: null,
    description: "Black frame, audio fade in",
  },
  "snow-problem-1": {
    type: "image",
    src: "reel/shots/snow-problem-1.jpg",
    description: "Snowy unshoveled sidewalk, someone struggling",
    tag: "THE PROBLEM",

  },
  "snow-problem-2": {
    type: "image",
    src: "reel/shots/snow-problem-2.jpg",
    description: "Wheelchair blocked by snow on sidewalk",

  },
  "stat-cities": {
    type: "text",
    src: null,
    description: "Stat card",
    textOverlay: "70% OF U.S. CITIES\nCAN'T CLEAR SIDEWALKS",
    bgColor: "#0a0a0a",
  },
  "snow-problem-3": {
    type: "image",
    src: "reel/shots/snow-problem-3.jpg",
    description: "Aerial view of snow-covered neighborhood sidewalks",
  },
  "dirty-sidewalk": {
    type: "image",
    src: "reel/shots/dirty-sidewalk.jpg",
    description: "Trash-covered SF sidewalk",
    tag: "THE PROBLEM",
  },
  "stat-cost": {
    type: "text",
    src: null,
    description: "Cost stat",
    textOverlay: "$2.4B ANNUAL\nSIDEWALK SNOW REMOVAL",
    bgColor: "#0a0a0a",
  },
  "stat-injuries": {
    type: "text",
    src: null,
    description: "Injury stat",
    textOverlay: "76,000+ INJURIES\nEVERY WINTER",
    bgColor: "#0a0a0a",
  },
  "fuck-snow": {
    type: "text",
    src: null,
    description: "Aggressive problem statement",
    textOverlay: "FUCK SNOW.\nFUCK ICE.\nFUCK TRASH.",
    bgColor: "#0a0a0a",
  },
  "what-if": {
    type: "text",
    src: null,
    description: "What if text card",
    textOverlay: "WHAT IF SIDEWALKS\nCLEARED THEMSELVES?",
    bgColor: "#0a0a0a",
  },
  "muni-logo-flash": {
    type: "image",
    src: null, // will use staticFile("images/muni-logo-light.svg")
    description: "Muni logo on black, quick flash",
    bgColor: "#0a0a0a",
  },
  "text-autonomous": {
    type: "text",
    src: null,
    description: "Autonomous text card",
    textOverlay: "AUTONOMOUS",
    bgColor: "#0a0a0a",
  },
  "text-electric": {
    type: "text",
    src: null,
    description: "Electric text card",
    textOverlay: "ELECTRIC",
    bgColor: "#0a0a0a",
  },
  "text-open-source": {
    type: "text",
    src: null,
    description: "Open source text card",
    textOverlay: "OPEN SOURCE",
    bgColor: "#0a0a0a",
  },
  "text-sidewalk-roombas": {
    type: "text",
    src: null,
    description: "Sidewalk roombas hook before drop",
    textOverlay: "SIDEWALK ROOMBAS",
    bgColor: "#0a0a0a",
  },
  "build-tension": {
    type: "black",
    src: null,
    description: "Black frame, tension build before drop",
  },

  // === THE DROP / REVEAL ===
  "drop-black": {
    type: "black",
    src: null,
    description: "Beat drop — flash to black",
  },
  "bvr1-hero-front": {
    type: "image",
    src: null,
    description: "BVR1 hero shot — front 3/4 angle",

  },
  "bvr1-hero-side": {
    type: "image",
    src: null,
    description: "BVR1 side profile, dramatic lighting",

  },
  "bvr1-hero-rear": {
    type: "image",
    src: null,
    description: "BVR1 rear 3/4, showing plow attachment",

  },
  "bvr1-3d-spin": {
    type: "3d",
    src: null,
    description: "3D model spin (uses ThreeCanvas BVR1Model)",
  },
  "no-gas-48v": {
    type: "text",
    src: null,
    description: "Electric stat: 48V",
    textOverlay: "48V SYSTEM",
    tag: "NO GAS",
    bgColor: "#050505",
  },
  "no-gas-vesc": {
    type: "text",
    src: null,
    description: "Electric stat: VESC",
    textOverlay: "4x VESC 75/300",
    tag: "NO GAS",
    bgColor: "#050505",
  },
  "no-gas-12kw": {
    type: "text",
    src: null,
    description: "Electric stat: 12kW",
    textOverlay: "12 kW PEAK",
    tag: "NO GAS",
    bgColor: "#050505",
  },
  "no-gas-zero": {
    type: "text",
    src: null,
    description: "Electric stat: zero emissions",
    textOverlay: "ZERO EMISSIONS",
    tag: "NO GAS",
    bgColor: "#050505",
  },
  "splat-flythrough": {
    type: "splat",
    src: null,
    description: "Gaussian splat fly-through, HELICOPTER music video style",
  },
  "drop-end": {
    type: "black",
    src: null,
    description: "Transition black",
  },

  // === THE FLEX ===
  "flex-snow-action-1": {
    type: "video",
    src: null,
    description: "BVR1 plowing snow on a sidewalk",

  },
  "flex-text-sidewalks": {
    type: "text",
    src: null,
    textOverlay: "SIDEWALKS FIRST",
    description: "Sidewalks first text",
    bgColor: "#0a0a0a",
  },
  "flex-snow-action-2": {
    type: "3d",
    src: null,
    description: "BVR1 in rain — all-weather operations",
  },
  "flex-text-24-7": {
    type: "text",
    src: null,
    textOverlay: "24/7 OPERATIONS",
    description: "24/7 text",
    bgColor: "#0a0a0a",
  },
  "flex-campus": {
    type: "image",
    src: "reel/shots/flex-campus.jpg",
    description: "BVR1 on university campus path",

  },
  "flex-text-no-driver": {
    type: "text",
    src: null,
    textOverlay: "NO DRIVER NEEDED",
    description: "No driver text",
    bgColor: "#0a0a0a",
  },
  "flex-night-ops": {
    type: "image",
    src: "reel/shots/flex-night-ops.jpg",
    description: "BVR1 operating at night with headlights",

  },
  "flex-text-silent": {
    type: "text",
    src: null,
    textOverlay: "SILENT RUNNING",
    description: "Silent text",
    bgColor: "#0a0a0a",
  },
  "flex-plow-closeup": {
    type: "image",
    src: "reel/shots/flex-plow-closeup.jpg",
    description: "Macro shot of plow mechanism",

  },
  "flex-text-40-inch": {
    type: "text",
    src: null,
    textOverlay: '40" CLEARANCE WIDTH',
    description: "Width text",
    bgColor: "#0a0a0a",
  },
  "flex-multi-rover": {
    type: "image",
    src: "reel/shots/flex-multi-rover.jpg",
    description: "Multiple BVR1 units in formation",

  },
  "flex-text-fleet": {
    type: "text",
    src: null,
    textOverlay: "FLEET SCALE",
    description: "Fleet text",
    bgColor: "#0a0a0a",
  },
  "flex-depot-shot": {
    type: "image",
    src: "reel/shots/flex-depot-shot.jpg",
    description: "Depot console UI showing fleet map",

  },
  "flex-text-one-operator": {
    type: "text",
    src: null,
    textOverlay: "1 OPERATOR\n10 ROVERS",
    description: "Operator ratio text",
    bgColor: "#0a0a0a",
  },

  // === THE TECH ===
  "tech-jetson": {
    type: "image",
    src: "reel/shots/tech-jetson.jpg",
    description: "Jetson Orin NX module close-up",

  },
  "tech-text-orin": {
    type: "text",
    src: null,
    textOverlay: "JETSON ORIN NX\n100 TOPS",
    description: "Orin spec text",
    bgColor: "#0a0a0a",
  },
  "tech-lidar": {
    type: "image",
    src: "reel/shots/tech-lidar.png",
    description: "LiDAR point cloud visualization",

  },
  "tech-text-rtk": {
    type: "text",
    src: null,
    textOverlay: "RTK GPS\n2cm ACCURACY",
    description: "RTK text",
    bgColor: "#0a0a0a",
  },
  "tech-console-ui": {
    type: "image",
    src: "reel/shots/tech-teleop-controller.jpg",
    description: "Xbox controller held next to rover — teleop in action",

  },
  "tech-text-teleop": {
    type: "text",
    src: null,
    textOverlay: "REMOTE TELEOP\nSUB-100ms LATENCY",
    description: "Teleop text",
    bgColor: "#0a0a0a",
  },
  "tech-splat-recon": {
    type: "splat",
    src: null,
    description: "Gaussian splat 3D reconstruction fly-through",
  },
  "tech-text-3d-maps": {
    type: "text",
    src: null,
    textOverlay: "3D GAUSSIAN\nSPLAT MAPS",
    description: "3D maps text",
    bgColor: "#0a0a0a",
  },
  "tech-can-bus": {
    type: "image",
    src: "reel/shots/tech-can-bus.jpg",
    description: "PCB / CAN bus wiring close-up",

  },
  "pink-bvr1": {
    type: "3d",
    src: null,
    description: "BVR1 3D model + grid tinted hot pink",
  },
  "tech-text-rust": {
    type: "text",
    src: null,
    textOverlay: "WRITTEN IN RUST\nZERO CRASHES",
    description: "Rust text",
    bgColor: "#0a0a0a",
  },
  "tech-training": {
    type: "image",
    src: "reel/shots/tech-training.jpg",
    description: "Training pipeline visualization / Rerun UI",

  },
  "tech-text-bc": {
    type: "text",
    src: null,
    textOverlay: "BEHAVIORAL CLONING\nLEARN BY WATCHING",
    description: "BC text",
    bgColor: "#0a0a0a",
  },

  // === THE BUSINESS ===
  "biz-tam-map": {
    type: "image",
    src: null,
    description: "US map highlighting snow belt cities",

  },
  "biz-text-32b": {
    type: "text",
    src: null,
    textOverlay: "$32B TAM\nSIDEWALK MAINTENANCE",
    description: "TAM text",
    bgColor: "#0a0a0a",
  },
  "biz-cities": {
    type: "image",
    src: "reel/shots/biz-cities.jpg",
    description: "City skyline / municipal building",

  },
  "biz-text-raas": {
    type: "text",
    src: null,
    textOverlay: "ROBOTS AS A SERVICE\nRECURRING REVENUE",
    description: "RaaS text",
    bgColor: "#0a0a0a",
  },
  "biz-fleet-diagram": {
    type: "image",
    src: null,
    description: "Fleet operations diagram",

  },
  "biz-text-margins": {
    type: "text",
    src: null,
    textOverlay: "80%+ GROSS MARGINS\nAFTER SCALE",
    description: "Margins text",
    bgColor: "#0a0a0a",
  },
  "biz-open-source": {
    type: "image",
    src: "reel/shots/biz-open-source.jpg",
    description: "GitHub repo / open source community",

  },
  "biz-text-community": {
    type: "text",
    src: null,
    textOverlay: "OPEN SOURCE\nCOMMUNITY MOAT",
    description: "Community text",
    bgColor: "#0a0a0a",
  },
  "biz-text-first-mover": {
    type: "text",
    src: null,
    textOverlay: "FIRST MOVER\nSIDEWALK AUTONOMY",
    description: "First mover text",
    bgColor: "#0a0a0a",
  },
  "biz-runway": {
    type: "image",
    src: null,
    description: "Product roadmap / timeline graphic",

  },

  // === THE CLOSE ===
  "close-checklist": {
    type: "text",
    src: null,
    description: "Boot sequence checklist (BootSequence component)",
  },
  "close-founder": {
    type: "image",
    src: null,
    description: "Founder with BVR1",

  },
  "close-bvr1-beauty": {
    type: "3d",
    src: null,
    description: "Final 3D beauty shot, slow orbit",
  },
  "close-cta": {
    type: "text",
    src: null,
    textOverlay: "LET'S CLEAR\nTHE WAY",
    description: "CTA text",
    bgColor: "#0a0a0a",
  },
  "close-url": {
    type: "text",
    src: null,
    textOverlay: "MUNI.WORKS",
    description: "Website URL",
    bgColor: "#0a0a0a",
  },
  "close-event": {
    type: "text",
    src: null,
    textOverlay: "SEE IT IN PERSON",
    description: "Event promo — staggered lines on beat",
    bgColor: "#0a0a0a",
  },
};

export function getShot(label: string): Shot {
  return SHOTS[label] ?? {
    type: "black",
    src: null,
    description: `Unknown shot: ${label}`,
  };
}
