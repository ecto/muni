export interface ModeConfig {
  id: "snow" | "sweep" | "wash";
  label: string;
  fogColor: string;
  lightIntensity: number;
  tint?: string;
  stats: Array<{ value: string; label: string }>;
}

export const PLATFORM_MODES: ModeConfig[] = [
  {
    id: "snow",
    label: "Snow",
    fogColor: "#1a1a2e",
    lightIntensity: 0.8,
    stats: [
      { value: '40"', label: "clearance width" },
      { value: "4-8hr", label: "runtime" },
      { value: "$38", label: "per day" },
    ],
  },
  {
    id: "sweep",
    label: "Sweep",
    fogColor: "#1a1e14",
    lightIntensity: 1.0,
    tint: "#88aa66",
    stats: [
      { value: "Silent", label: "operation" },
      { value: "Daily", label: "schedule" },
      { value: "ADA", label: "compliant" },
    ],
  },
  {
    id: "wash",
    label: "Wash",
    fogColor: "#141a22",
    lightIntensity: 0.6,
    tint: "#6688aa",
    stats: [
      { value: "2000 PSI", label: "pressure wash" },
      { value: "Overnight", label: "cycle" },
      { value: "Zero", label: "chemicals" },
    ],
  },
];
