/**
 * Muni Design System — Spacing Tokens
 *
 * 4px base unit. Named scale for consistency across all layouts.
 * Use semantic aliases where intent matters more than exact value.
 */

// Raw scale — multiples of 4px base unit
export const space = {
  px: "1px",
  0: "0px",
  0.5: "2px",
  1: "4px",
  1.5: "6px",
  2: "8px",
  2.5: "10px",
  3: "12px",
  4: "16px",
  5: "20px",
  6: "24px",
  7: "28px",
  8: "32px",
  9: "36px",
  10: "40px",
  12: "48px",
  14: "56px",
  16: "64px",
  20: "80px",
  24: "96px",
  28: "112px",
  32: "128px",
  40: "160px",
  48: "192px",
  56: "224px",
  64: "256px",
} as const;

// Semantic spacing — use these for layout intent
export const layout = {
  "page-x": space[6], // horizontal page padding (24px)
  "page-x-lg": space[10], // wide viewport page padding (40px)
  "page-y": space[8], // vertical page padding (32px)
  "section-gap": space[24], // between major sections (96px)
  "content-gap": space[12], // between content blocks (48px)
  "element-gap": space[4], // between related elements (16px)
  "inline-gap": space[2], // between inline items (8px)
  "gutter": space[4], // grid gutter (16px)
  "gutter-lg": space[6], // wide grid gutter (24px)
} as const;

// Component-level spacing
export const component = {
  "card-padding": space[5], // 20px
  "card-gap": space[4], // 16px
  "input-x": space[3], // horizontal input padding (12px)
  "input-y": space[2], // vertical input padding (8px)
  "button-x": space[4], // 16px
  "button-y": space[2], // 8px
  "badge-x": space[2], // 8px
  "badge-y": space[1], // 4px
} as const;

// Content width constraints
export const maxWidth = {
  prose: "65ch", // reading width
  "prose-wide": "80ch", // wider reading (code, tables)
  content: "1200px", // main content area
  page: "1440px", // outer page constraint
} as const;

export const spacing = {
  space,
  layout,
  component,
  maxWidth,
} as const;
