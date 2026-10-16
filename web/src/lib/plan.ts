import table from "@/content/elements.json";

// The master plan, as data for the homepage. The periodic table comes from the paper's own source
// (`npm run posts` writes elements.json); the stages mirror its overview.

export interface Family {
  key: string;
  name: string;
  color: string;
}

export interface Element {
  num: number;
  name: string;
  family: string;
  maturity: "exists" | "designed" | "concept";
  note: string;
  stage: string;
}

export const families: Family[] = table.families;
export const elements = table.elements as Element[];

export function family(key: string): Family {
  return families.find((f) => f.key === key) ?? families[0];
}

export function element(num: number): Element {
  return elements.find((e) => e.num === num) ?? elements[0];
}

/** The table's layout: each family is a list of columns, read top to bottom in build order, as in the paper. */
export const layout: [string, number[][]][] = [
  ["make", [[1, 2, 3, 4]]],
  ["world", [[5, 6, 7, 8]]],
  ["body", [[9, 10, 11], [12, 13, 14], [15, 16]]],
  ["power", [[17, 18, 19]]],
  ["mind", [[20, 21]]],
  ["silicon", [[22, 23, 24]]],
];

/** Public homes for the projects that have one. */
export const links: Record<string, string> = {
  loon: "https://github.com/ecto/loon",
  tang: "https://github.com/ecto/tang",
  vcad: "https://vcad.io",
  kerf: "https://github.com/ecto/kerf",
  kosm: "https://github.com/ecto/kosm",
  phyz: "https://github.com/ecto/phyz",
  torc: "https://github.com/ecto/torc",
};

export const maturityLabel: Record<Element["maturity"], string> = {
  exists: "in use today",
  designed: "designed",
  concept: "concept",
};

export interface Stage {
  name: string;
  what: string;
  watts: string;
}

/** The eight stages, each a step up in scale and each built by the one before. Only Desk is a plan. */
export const stages: Stage[] = [
  { name: "Desk", what: "Software that funds an arm that ships with its gap", watts: "0 W → 10 kW" },
  { name: "Shop", what: "Two limbs at any table; a pair builds a pair", watts: "100 kW" },
  { name: "Road", what: "A pair on legs that fits its own model", watts: "1 MW" },
  { name: "City", what: "A fleet and its pairs work a city", watts: "10 MW" },
  { name: "Home", what: "Lineages on their own power", watts: "1 GW" },
  { name: "Moon", what: "A seed grows a pair, with a lifeline", watts: "1 kW off Earth" },
  { name: "Mars", what: "The same seed, lifeline cut", watts: "1 kW off Earth" },
  { name: "Star", what: "A swarm of collectors around a star", watts: "3.8 × 10²⁶ W" },
];

export const deskGate =
  "First, gateway revenue that pays for the arm. Then ten limbs sold at 40% gross margin, each with its tool-point error ε̄ measured and under 2 mm.";
