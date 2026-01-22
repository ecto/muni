import { interpolate, Easing } from "remotion";

// Standard easing curves
export const easeOutQuart = (t: number): number => 1 - Math.pow(1 - t, 4);
export const easeInOutQuart = (t: number): number =>
  t < 0.5 ? 8 * t * t * t * t : 1 - Math.pow(-2 * t + 2, 4) / 2;
export const easeOutExpo = (t: number): number =>
  t === 1 ? 1 : 1 - Math.pow(2, -10 * t);
export const easeInOutExpo = (t: number): number =>
  t === 0
    ? 0
    : t === 1
      ? 1
      : t < 0.5
        ? Math.pow(2, 20 * t - 10) / 2
        : (2 - Math.pow(2, -20 * t + 10)) / 2;

// Spring-like bounce
export const easeOutBack = (t: number): number => {
  const c1 = 1.70158;
  const c3 = c1 + 1;
  return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
};

// Smooth interpolation helper for Remotion
export function smoothValue(
  frame: number,
  start: number,
  end: number,
  outputStart: number,
  outputEnd: number,
  easingFn: (t: number) => number = easeOutQuart
): number {
  return interpolate(frame, [start, end], [outputStart, outputEnd], {
    easing: easingFn,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
}

// Fade helper
export function fade(
  frame: number,
  fadeInStart: number,
  fadeInEnd: number,
  fadeOutStart: number,
  fadeOutEnd: number
): number {
  const fadeIn = interpolate(frame, [fadeInStart, fadeInEnd], [0, 1], {
    easing: Easing.ease,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const fadeOut = interpolate(frame, [fadeOutStart, fadeOutEnd], [1, 0], {
    easing: Easing.ease,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  return fadeIn * fadeOut;
}

// Camera orbit helper (returns angle in radians)
export function orbitAngle(
  frame: number,
  startFrame: number,
  endFrame: number,
  startAngle: number = 0,
  endAngle: number = Math.PI * 2,
  easingFn: (t: number) => number = easeInOutExpo
): number {
  return smoothValue(frame, startFrame, endFrame, startAngle, endAngle, easingFn);
}
