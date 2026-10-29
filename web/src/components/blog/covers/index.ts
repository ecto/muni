import type { ComponentType } from "react";
import { PinnedRows } from "./PinnedRows";
import { SolarSystem } from "./SolarSystem";

// Cover art per post, chosen by `cover` in posts.json. The rail's two end labels
// travel with the art so the scrollbar reads in the cover's own terms.
export interface Cover {
  Art: ComponentType;
  rail: { top: string; bottom: string; variant: string };
}

export const COVERS: Record<string, Cover> = {
  "solar-system": { Art: SolarSystem, rail: { top: "Earth", bottom: "Sun", variant: "orbit" } },
  "pinned-rows": { Art: PinnedRows, rail: { top: "g = 0.9", bottom: "g = 1", variant: "gate" } },
};

export function coverFor(id?: string): Cover {
  return (id && COVERS[id]) || COVERS["solar-system"];
}
