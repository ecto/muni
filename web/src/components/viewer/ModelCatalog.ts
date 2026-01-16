export interface ModelInfo {
  id: string;
  name: string;
  path: string;
  icon: string;
  desc: string;
  hasLabels: boolean;
}

export interface ModelSection {
  section: string;
  items: ModelInfo[];
}

export interface ComponentInfo {
  id: number;
  section: string;
  name: string;
  desc: string;
  specs: string;
  position: { x: number; y: number; z: number };
}

export interface ScaleReference {
  id: string;
  name: string;
  icon: string;
  desc: string;
  path: string;
  scale: number;
  rotateY?: number;
  attribution?: {
    title: string;
    author: string;
    url: string;
    license: string;
  };
}

export const models: ModelSection[] = [
  {
    section: "Assemblies",
    items: [
      {
        id: "bvr1",
        name: "BVR1 Production",
        path: "/models/bvr1_assembly_realistic.glb",
        icon: "rocket-launch",
        desc: "Current production rover",
        hasLabels: true,
      },
      {
        id: "bvr0",
        name: "BVR0 Prototype",
        path: "/models/bvr0_assembly.glb",
        icon: "rocket-launch",
        desc: "First prototype rover",
        hasLabels: false,
      },
    ],
  },
  {
    section: "Frame",
    items: [
      {
        id: "frame",
        name: "BVR1 Frame",
        path: "/models/bvr1_frame.glb",
        icon: "blueprint",
        desc: "Aluminum extrusion frame",
        hasLabels: false,
      },
    ],
  },
  {
    section: "Drivetrain",
    items: [
      {
        id: "svb6hs",
        name: "SVB6HS 6.5\"",
        path: "/models/uumotor_svb6hs.glb",
        icon: "tire",
        desc: "500W hub motor, 6.5\" wheel",
        hasLabels: false,
      },
      {
        id: "kn6104",
        name: 'KN6104 10"',
        path: "/models/uumotor_kn6104.glb",
        icon: "tire",
        desc: '800W hub motor, 10" wheel',
        hasLabels: false,
      },
      {
        id: "mount",
        name: "Motor Mount",
        path: "/models/uumotor_mount.glb",
        icon: "wrench",
        desc: "Adjustable wheel mount",
        hasLabels: false,
      },
    ],
  },
  {
    section: "Electronics",
    items: [
      {
        id: "tray",
        name: "Base Tray",
        path: "/models/base_tray.glb",
        icon: "package",
        desc: "Electronics mounting plate",
        hasLabels: false,
      },
      {
        id: "panel",
        name: "Access Panel",
        path: "/models/access_panel.glb",
        icon: "door-open",
        desc: "Removable service panel",
        hasLabels: false,
      },
    ],
  },
];

export const bvr1Components: ComponentInfo[] = [
  {
    id: 1,
    section: "Sensors",
    name: "LiDAR",
    desc: "Livox Mid-360, 360° scanning",
    specs: "40m range, 200k pts/sec",
    position: { x: 0, y: 520, z: 200 },
  },
  {
    id: 2,
    section: "Sensors",
    name: "360° Camera",
    desc: "Insta360 X4, panoramic video",
    specs: "8K 360° capture",
    position: { x: 0, y: 420, z: 200 },
  },
  {
    id: 3,
    section: "Sensors",
    name: "RTK GPS",
    desc: "Multi-band GNSS antenna",
    specs: "cm-level positioning",
    position: { x: 80, y: 470, z: 200 },
  },
  {
    id: 4,
    section: "Controls",
    name: "E-Stop",
    desc: "Emergency stop button",
    specs: "NC contacts, 22mm",
    position: { x: -150, y: 340, z: -200 },
  },
  {
    id: 5,
    section: "Electronics",
    name: "Jetson Orin NX",
    desc: "Main compute module",
    specs: "100 TOPS, 16GB RAM",
    position: { x: 130, y: 170, z: 180 },
  },
  {
    id: 6,
    section: "Electronics",
    name: "DC-DC Converter",
    desc: "48V to 12V supply",
    specs: "300W continuous",
    position: { x: 130, y: 155, z: -180 },
  },
  {
    id: 7,
    section: "Electronics",
    name: "VESC (FL)",
    desc: "Front-left motor controller",
    specs: "VESC 6, 60A",
    position: { x: -160, y: 155, z: 120 },
  },
  {
    id: 8,
    section: "Electronics",
    name: "VESC (RL)",
    desc: "Rear-left motor controller",
    specs: "VESC 6, 60A",
    position: { x: -160, y: 155, z: -120 },
  },
  {
    id: 9,
    section: "Power",
    name: "Battery Pack",
    desc: "Custom 13S4P Li-ion",
    specs: "48V 14Ah, 672Wh",
    position: { x: 0, y: 185, z: 0 },
  },
  {
    id: 10,
    section: "Frame",
    name: "Frame",
    desc: "2020 aluminum extrusion",
    specs: "380×600mm footprint",
    position: { x: 0, y: 250, z: 0 },
  },
  {
    id: 11,
    section: "Frame",
    name: "Access Panel",
    desc: "Removable top cover",
    specs: "ABS, quick-release",
    position: { x: 100, y: 324, z: 100 },
  },
  {
    id: 12,
    section: "Frame",
    name: "Base Tray",
    desc: "Electronics mounting",
    specs: "ABS, 6mm thick",
    position: { x: 100, y: 140, z: -100 },
  },
  {
    id: 13,
    section: "Drivetrain",
    name: "Hub Motor (FL)",
    desc: "UUMotor SVB6HS",
    specs: '500W, 6.5" wheel',
    position: { x: -132, y: 38, z: 260 },
  },
  {
    id: 14,
    section: "Drivetrain",
    name: "Hub Motor (FR)",
    desc: "UUMotor SVB6HS",
    specs: '500W, 6.5" wheel',
    position: { x: 132, y: 38, z: 260 },
  },
  {
    id: 15,
    section: "Drivetrain",
    name: "Hub Motor (RL)",
    desc: "UUMotor SVB6HS",
    specs: '500W, 6.5" wheel',
    position: { x: -132, y: 38, z: -260 },
  },
  {
    id: 16,
    section: "Drivetrain",
    name: "Hub Motor (RR)",
    desc: "UUMotor SVB6HS",
    specs: '500W, 6.5" wheel',
    position: { x: 132, y: 38, z: -260 },
  },
  {
    id: 17,
    section: "Drivetrain",
    name: "Motor Mount",
    desc: "L-bracket mount",
    specs: "6061-T6 aluminum",
    position: { x: -190, y: 100, z: 260 },
  },
];

export const scaleReferences: ScaleReference[] = [
  {
    id: "banana",
    name: "Banana",
    icon: "plant",
    desc: "Universal scale reference",
    path: "/models/banana.glb",
    scale: 1.0,
    rotateY: Math.PI,
  },
  {
    id: "astronaut",
    name: "Astronaut",
    icon: "person-arms-spread",
    desc: "1.85m tall",
    path: "/models/astronaut.glb",
    scale: 1000.0,
    attribution: {
      title: "astronaut",
      author: "Antropik",
      url: "https://skfb.ly/oVwuz",
      license: "CC BY 4.0",
    },
  },
  {
    id: "grogu",
    name: "Grogu",
    icon: "star-four",
    desc: "34cm tall",
    path: "/models/grogu.glb",
    scale: 34.0,
    attribution: {
      title: "BABY YODA FREE 3D",
      author: "OSCAR CREATIVO",
      url: "https://skfb.ly/6Rovs",
      license: "CC BY 4.0",
    },
  },
];

export function findModelById(id: string): ModelInfo | undefined {
  for (const section of models) {
    for (const item of section.items) {
      if (item.id === id) return item;
    }
  }
  return undefined;
}
