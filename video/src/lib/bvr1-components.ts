// Component data from web/src/components/viewer/ModelCatalog.ts

export interface ComponentInfo {
  id: number;
  section: string;
  name: string;
  desc: string;
  specs: string;
  position: { x: number; y: number; z: number };
}

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

// Key components for video callouts (subset of full list)
export const videoHighlightComponents = [
  bvr1Components[0], // LiDAR
  bvr1Components[1], // 360° Camera
  bvr1Components[12], // Hub Motor (FL)
  bvr1Components[8], // Battery Pack
] as const;

export function getComponentById(id: number): ComponentInfo | undefined {
  return bvr1Components.find((c) => c.id === id);
}
