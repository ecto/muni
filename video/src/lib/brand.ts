// Muni brand constants

export const MUNI_ORANGE = "#ff6600";
export const DARK_BG = "#0a0a0a";
export const WHITE = "#ffffff";
export const BLACK = "#000000";

export const FONT_MONO = '"Berkeley Mono", ui-monospace, monospace';

export const colors = {
  orange: MUNI_ORANGE,
  dark: DARK_BG,
  white: WHITE,
  black: BLACK,
  // Variations
  orangeDim: "#cc5200",
  orangeBright: "#ff8833",
  grayDark: "#1a1a1a",
  grayMid: "#333333",
  grayLight: "#666666",
} as const;

export const gridColors = {
  cell: "#333333",
  section: MUNI_ORANGE,
} as const;
