/**
 * Muni Design System — Breakpoint Tokens
 *
 * Mobile-first. Breakpoints mark where layouts adapt.
 * Named for viewport intent, not device type.
 */

export const breakpoint = {
  sm: "640px",
  md: "768px",
  lg: "1024px",
  xl: "1280px",
  "2xl": "1536px",
} as const;

// Container queries — component-level responsiveness
export const container = {
  compact: "320px",
  medium: "480px",
  wide: "640px",
} as const;

export const breakpoints = {
  breakpoint,
  container,
} as const;
