/**
 * Muni Design System — Motion Tokens
 *
 * Motion philosophy: purposeful, not decorative.
 * Every animation should communicate state or guide attention.
 *
 * Inspired by the physical robot's motion language:
 *   - Predictable, steady movements
 *   - Deliberate pauses (yield choreography)
 *   - No sudden or erratic motion
 */

// Duration scale — shorter for micro-interactions, longer for layout shifts
export const duration = {
  instant: "0ms",
  fast: "100ms",
  normal: "200ms",
  slow: "300ms",
  slower: "500ms",
  deliberate: "700ms",
  // Page-level transitions
  enter: "400ms",
  exit: "200ms",
} as const;

// Easing curves
export const easing = {
  // Standard — most interactions
  default: "cubic-bezier(0.25, 0.1, 0.25, 1)",
  // Entering elements — decelerating into place
  out: "cubic-bezier(0, 0, 0.2, 1)",
  // Exiting elements — accelerating away
  in: "cubic-bezier(0.4, 0, 1, 1)",
  // Emphasis — slight overshoot for tactile feel
  spring: "cubic-bezier(0.34, 1.56, 0.64, 1)",
  // Linear — progress bars, continuous motion
  linear: "linear",
} as const;

// Spring physics — for JS animation libraries (Framer Motion, React Spring)
export const spring = {
  // Snappy response — buttons, toggles
  snappy: { stiffness: 400, damping: 30, mass: 1 },
  // Default — most element transitions
  default: { stiffness: 200, damping: 24, mass: 1 },
  // Gentle — page transitions, large elements
  gentle: { stiffness: 120, damping: 20, mass: 1 },
  // Bouncy — playful moments (use sparingly)
  bouncy: { stiffness: 300, damping: 15, mass: 1 },
} as const;

// Predefined transitions — composites of property + duration + easing
export const transition = {
  // Micro — hover states, focus rings
  color: `color ${duration.fast} ${easing.default}`,
  opacity: `opacity ${duration.normal} ${easing.default}`,
  transform: `transform ${duration.normal} ${easing.out}`,
  // Compound — multiple properties
  interactive: `color ${duration.fast} ${easing.default}, background-color ${duration.fast} ${easing.default}, border-color ${duration.fast} ${easing.default}, opacity ${duration.normal} ${easing.default}`,
  // Layout — expanding/collapsing sections
  expand: `height ${duration.slow} ${easing.out}, opacity ${duration.slow} ${easing.out}`,
  collapse: `height ${duration.normal} ${easing.in}, opacity ${duration.fast} ${easing.in}`,
  // Page — route transitions
  "page-enter": `opacity ${duration.enter} ${easing.out}, transform ${duration.enter} ${easing.out}`,
  "page-exit": `opacity ${duration.exit} ${easing.in}`,
} as const;

// Keyframe definitions — reusable animation names
export const keyframes = {
  "fade-in": {
    from: { opacity: "0" },
    to: { opacity: "1" },
  },
  "fade-up": {
    from: { opacity: "0", transform: "translateY(8px)" },
    to: { opacity: "1", transform: "translateY(0)" },
  },
  "slide-in-right": {
    from: { opacity: "0", transform: "translateX(-8px)" },
    to: { opacity: "1", transform: "translateX(0)" },
  },
  pulse: {
    "0%, 100%": { opacity: "1" },
    "50%": { opacity: "0.5" },
  },
  "progress-slide": {
    from: { transform: "translateX(-100%)" },
    to: { transform: "translateX(100%)" },
  },
  // Dot-matrix grid shimmer — for background texture
  shimmer: {
    "0%": { opacity: "0.03" },
    "50%": { opacity: "0.06" },
    "100%": { opacity: "0.03" },
  },
} as const;

export const motion = {
  duration,
  easing,
  spring,
  transition,
  keyframes,
} as const;
