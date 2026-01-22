export interface StateInfo {
  slug: string;
  name: string;
  abbreviation: string;
  snowBelt: boolean;
  avgAnnualSnowfall: number; // inches
  liability: {
    standard: "reasonable-care" | "natural-accumulation" | "varies";
    municipalResponsibility: boolean;
    typicalClearanceWindow: string;
    summary: string;
    source: string;
  };
  majorCities: string[];
}

export const states: StateInfo[] = [
  {
    slug: "ohio",
    name: "Ohio",
    abbreviation: "OH",
    snowBelt: true,
    avgAnnualSnowfall: 28,
    liability: {
      standard: "reasonable-care",
      municipalResponsibility: true,
      typicalClearanceWindow: "24 hours",
      summary:
        "Ohio municipalities can require property owners to clear sidewalks, typically within 24 hours after snowfall ends. Property owners who fail to comply may face fines and liability for injuries. Municipal immunity is limited—cities can be liable for negligent maintenance of public sidewalks.",
      source: "https://codes.ohio.gov/ohio-revised-code/section-723.011",
    },
    majorCities: ["Cleveland", "Columbus", "Cincinnati", "Toledo", "Akron"],
  },
  {
    slug: "michigan",
    name: "Michigan",
    abbreviation: "MI",
    snowBelt: true,
    avgAnnualSnowfall: 60,
    liability: {
      standard: "reasonable-care",
      municipalResponsibility: true,
      typicalClearanceWindow: "24-48 hours",
      summary:
        "Michigan's sidewalk liability varies by municipality. Many cities require abutting property owners to maintain sidewalks, but the municipality retains ultimate responsibility for public right-of-way. The governmental immunity act limits but does not eliminate municipal liability for dangerous conditions.",
      source: "https://www.legislature.mi.gov/",
    },
    majorCities: ["Detroit", "Grand Rapids", "Ann Arbor", "Lansing", "Flint"],
  },
  {
    slug: "pennsylvania",
    name: "Pennsylvania",
    abbreviation: "PA",
    snowBelt: true,
    avgAnnualSnowfall: 41,
    liability: {
      standard: "reasonable-care",
      municipalResponsibility: true,
      typicalClearanceWindow: "24-48 hours",
      summary:
        "Pennsylvania follows the 'hills and ridges' doctrine—property owners aren't liable for generally slippery conditions but are liable for failing to remove accumulated ridges of ice and snow. Municipalities must keep public sidewalks reasonably safe and can pass ordinances requiring property owner compliance.",
      source:
        "https://www.legis.state.pa.us/cfdocs/legis/LI/consCheck.cfm?txtType=HTM&ttl=53",
    },
    majorCities: ["Philadelphia", "Pittsburgh", "Allentown", "Erie", "Scranton"],
  },
  {
    slug: "new-york",
    name: "New York",
    abbreviation: "NY",
    snowBelt: true,
    avgAnnualSnowfall: 55,
    liability: {
      standard: "reasonable-care",
      municipalResponsibility: true,
      typicalClearanceWindow: "4-24 hours",
      summary:
        "New York has strong sidewalk liability laws. NYC specifically holds property owners responsible for sidewalk maintenance and injury liability. Upstate municipalities vary but generally require prompt snow removal. The 'storm in progress' rule provides temporary immunity during active precipitation.",
      source: "https://www.nyc.gov/site/buildings/codes/administrative-code.page",
    },
    majorCities: ["New York City", "Buffalo", "Rochester", "Syracuse", "Albany"],
  },
  {
    slug: "massachusetts",
    name: "Massachusetts",
    abbreviation: "MA",
    snowBelt: true,
    avgAnnualSnowfall: 48,
    liability: {
      standard: "reasonable-care",
      municipalResponsibility: true,
      typicalClearanceWindow: "Varies by municipality",
      summary:
        "Massachusetts municipalities set their own snow removal requirements. Boston requires clearance within 3 hours after snowfall ends for commercial properties, longer for residential. Property owners can be fined and held liable for injuries on poorly maintained sidewalks.",
      source: "https://www.mass.gov/",
    },
    majorCities: ["Boston", "Worcester", "Springfield", "Cambridge", "Lowell"],
  },
];

export function findStateBySlug(slug: string): StateInfo | undefined {
  return states.find((s) => s.slug === slug);
}

export function getAllStateSlugs(): string[] {
  return states.map((s) => s.slug);
}
