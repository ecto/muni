/**
 * Muni Design System — Typography Tokens
 *
 * Two-font hierarchy:
 *   Display — Space Grotesk (geometric sans, for headlines + brand moments)
 *   Mono   — Berkeley Mono (terminal aesthetic, body + UI + code + data)
 *
 * The mono-first approach inverts the typical sans-body/mono-code pattern.
 * Berkeley Mono IS the voice of muni. Space Grotesk is the accent.
 */

export const fontFamily = {
  mono: [
    "Berkeley Mono",
    "SF Mono",
    "Cascadia Code",
    "Fira Code",
    "JetBrains Mono",
    "Courier New",
    "monospace",
  ],
  display: [
    "Space Grotesk",
    "SF Pro Display",
    "Inter",
    "system-ui",
    "sans-serif",
  ],
} as const;

// Modular scale — 1.25 ratio (major third) from 14px base
// Round to nearest 0.5px for subpixel clarity
export const fontSize = {
  "2xs": "10px",
  xs: "11px",
  sm: "12px",
  base: "14px",
  lg: "16px",
  xl: "18px",
  "2xl": "22px",
  "3xl": "28px",
  "4xl": "36px",
  "5xl": "48px",
  "6xl": "60px",
  "7xl": "80px",
} as const;

export const fontWeight = {
  regular: "400",
  medium: "500",
  semibold: "600",
  bold: "700",
} as const;

export const lineHeight = {
  none: "1",
  tight: "1.15",
  snug: "1.3",
  normal: "1.5",
  relaxed: "1.625",
  loose: "2",
} as const;

export const letterSpacing = {
  tighter: "-0.04em",
  tight: "-0.02em",
  normal: "0em",
  wide: "0.02em",
  wider: "0.04em",
  widest: "0.08em",
} as const;

// Predefined text styles — composites of size + weight + leading + tracking
export const textStyle = {
  // Display — Space Grotesk, tight leading, bold headlines
  "display-xl": {
    fontFamily: "display",
    fontSize: "7xl",
    fontWeight: "bold",
    lineHeight: "none",
    letterSpacing: "tighter",
  },
  "display-lg": {
    fontFamily: "display",
    fontSize: "5xl",
    fontWeight: "bold",
    lineHeight: "tight",
    letterSpacing: "tighter",
  },
  "display-md": {
    fontFamily: "display",
    fontSize: "4xl",
    fontWeight: "semibold",
    lineHeight: "tight",
    letterSpacing: "tight",
  },
  "display-sm": {
    fontFamily: "display",
    fontSize: "2xl",
    fontWeight: "semibold",
    lineHeight: "snug",
    letterSpacing: "tight",
  },

  // Heading — Berkeley Mono, structural hierarchy
  "heading-lg": {
    fontFamily: "mono",
    fontSize: "xl",
    fontWeight: "bold",
    lineHeight: "snug",
    letterSpacing: "normal",
  },
  "heading-md": {
    fontFamily: "mono",
    fontSize: "lg",
    fontWeight: "bold",
    lineHeight: "snug",
    letterSpacing: "normal",
  },
  "heading-sm": {
    fontFamily: "mono",
    fontSize: "base",
    fontWeight: "bold",
    lineHeight: "snug",
    letterSpacing: "normal",
  },

  // Body — Berkeley Mono, comfortable reading
  "body-lg": {
    fontFamily: "mono",
    fontSize: "lg",
    fontWeight: "regular",
    lineHeight: "relaxed",
    letterSpacing: "normal",
  },
  "body-md": {
    fontFamily: "mono",
    fontSize: "base",
    fontWeight: "regular",
    lineHeight: "normal",
    letterSpacing: "normal",
  },
  "body-sm": {
    fontFamily: "mono",
    fontSize: "sm",
    fontWeight: "regular",
    lineHeight: "normal",
    letterSpacing: "normal",
  },

  // Label — Berkeley Mono, compact UI elements
  "label-lg": {
    fontFamily: "mono",
    fontSize: "sm",
    fontWeight: "medium",
    lineHeight: "tight",
    letterSpacing: "wide",
  },
  "label-md": {
    fontFamily: "mono",
    fontSize: "xs",
    fontWeight: "medium",
    lineHeight: "tight",
    letterSpacing: "wide",
  },
  "label-sm": {
    fontFamily: "mono",
    fontSize: "2xs",
    fontWeight: "medium",
    lineHeight: "tight",
    letterSpacing: "wider",
  },

  // Code — Berkeley Mono, data-dense
  code: {
    fontFamily: "mono",
    fontSize: "sm",
    fontWeight: "regular",
    lineHeight: "normal",
    letterSpacing: "normal",
  },

  // Tabular — Berkeley Mono, aligned columns
  tabular: {
    fontFamily: "mono",
    fontSize: "xs",
    fontWeight: "regular",
    lineHeight: "snug",
    letterSpacing: "normal",
  },
} as const;

// Print-specific overrides (Typst)
export const print = {
  baseFontSize: "9pt",
  headingFontSize: "14pt",
  lineHeight: "0.9em",
  maxWidth: "80ch",
  proseFamily: "Helvetica Neue", // print body stays serif-adjacent for legibility
  monoFamily: "Berkeley Mono",
} as const;

export const typography = {
  fontFamily,
  fontSize,
  fontWeight,
  lineHeight,
  letterSpacing,
  textStyle,
  print,
} as const;
