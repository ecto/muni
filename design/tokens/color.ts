/**
 * Muni Design System — Color Tokens
 *
 * All colors in OKLCH for perceptual uniformity.
 * oklch(lightness chroma hue)
 *   lightness: 0–1
 *   chroma:    0–0.4 (saturation)
 *   hue:       0–360 degrees
 */

// Named hue angles for programmatic palette generation
export const hue = {
  orange: 47.604,
  red: 18.5,
  blue: 264,
  green: 149,
  stone: 106.424,
  cyan: 200,
  purple: 300,
  amber: 70,
} as const;

// Full palette — every raw color available to the system
export const palette = {
  black: "oklch(0 0 0)",
  white: "oklch(1 0 0)",

  orange: {
    50: "oklch(0.97 0.02 47.604)",
    100: "oklch(0.93 0.05 47.604)",
    200: "oklch(0.87 0.09 47.604)",
    300: "oklch(0.80 0.13 47.604)",
    400: "oklch(0.75 0.17 47.604)",
    500: "oklch(0.705 0.213 47.604)", // #ff6600 — safety orange, brand primary
    600: "oklch(0.62 0.19 47.604)",
    700: "oklch(0.54 0.16 47.604)",
    800: "oklch(0.45 0.13 47.604)",
    900: "oklch(0.37 0.10 47.604)",
    950: "oklch(0.27 0.07 47.604)",
  },

  stone: {
    50: "oklch(0.985 0.001 106.424)",
    100: "oklch(0.97 0.001 106.424)",
    200: "oklch(0.923 0.003 106.424)",
    300: "oklch(0.869 0.005 106.424)",
    400: "oklch(0.709 0.010 106.424)",
    500: "oklch(0.553 0.013 106.424)",
    600: "oklch(0.444 0.011 106.424)",
    700: "oklch(0.374 0.010 106.424)",
    800: "oklch(0.268 0.007 106.424)",
    900: "oklch(0.216 0.006 106.424)",
    950: "oklch(0.147 0.004 49.25)",
  },

  red: {
    50: "oklch(0.97 0.02 18.5)",
    100: "oklch(0.93 0.05 18.5)",
    200: "oklch(0.85 0.10 18.5)",
    300: "oklch(0.75 0.14 18.5)",
    400: "oklch(0.64 0.17 18.5)",
    500: "oklch(0.53 0.19 18.5)",
    600: "oklch(0.467 0.186 18.5)", // #C41E3A — danger
    700: "oklch(0.39 0.15 18.5)",
    800: "oklch(0.32 0.12 18.5)",
    900: "oklch(0.26 0.09 18.5)",
    950: "oklch(0.18 0.06 18.5)",
  },

  blue: {
    50: "oklch(0.97 0.02 264)",
    100: "oklch(0.93 0.04 264)",
    200: "oklch(0.87 0.08 264)",
    300: "oklch(0.78 0.12 264)",
    400: "oklch(0.68 0.17 264)",
    500: "oklch(0.59 0.20 264)",
    600: "oklch(0.546 0.215 264)", // #2563EB — info
    700: "oklch(0.45 0.18 264)",
    800: "oklch(0.37 0.15 264)",
    900: "oklch(0.30 0.11 264)",
    950: "oklch(0.21 0.07 264)",
  },

  green: {
    50: "oklch(0.97 0.02 149)",
    100: "oklch(0.93 0.05 149)",
    200: "oklch(0.87 0.09 149)",
    300: "oklch(0.80 0.13 149)",
    400: "oklch(0.76 0.16 149)",
    500: "oklch(0.723 0.191 149)", // #22c55e — success
    600: "oklch(0.62 0.17 149)",
    700: "oklch(0.53 0.14 149)",
    800: "oklch(0.44 0.11 149)",
    900: "oklch(0.36 0.08 149)",
    950: "oklch(0.25 0.06 149)",
  },
} as const;

// Semantic aliases — use these in components, not raw palette values
export const semantic = {
  danger: palette.red[600],
  warning: palette.orange[500],
  info: palette.blue[600],
  success: palette.green[500],
} as const;

// Theme tokens — light and dark surfaces
export const theme = {
  light: {
    bg: palette.white,
    fg: palette.stone[950],
    "fg-muted": palette.stone[500],
    surface: palette.stone[50],
    "surface-raised": palette.white,
    border: palette.stone[300],
    "border-muted": palette.stone[200],
    primary: palette.orange[500],
    "primary-fg": palette.black,
    ring: palette.orange[200],
  },
  dark: {
    bg: palette.black,
    fg: "oklch(0.985 0.002 247.839)",
    "fg-muted": palette.stone[500],
    surface: palette.stone[950],
    "surface-raised": palette.stone[900],
    border: palette.stone[800],
    "border-muted": palette.stone[900],
    primary: palette.orange[500],
    "primary-fg": palette.black,
    ring: palette.orange[800],
  },
} as const;

// Chart palette — 5 visually distinct colors for data visualization
export const chart = {
  1: palette.orange[500],      // brand orange — oklch(0.705 0.213 47.604)
  2: "oklch(0.75 0.20 195)",   // electric cyan
  3: "oklch(0.82 0.19 85)",    // hot amber
  4: "oklch(0.68 0.22 300)",   // vivid purple
  5: "oklch(0.72 0.21 25)",    // bright coral
} as const;

// All tokens grouped for build consumption
export const color = {
  hue,
  palette,
  semantic,
  theme,
  chart,
} as const;
