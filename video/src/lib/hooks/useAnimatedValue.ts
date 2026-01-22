import { useCurrentFrame } from "remotion";
import { smoothValue, easeOutQuart } from "../easing";

export function useAnimatedValue(
  startFrame: number,
  endFrame: number,
  startValue: number,
  endValue: number,
  easingFn: (t: number) => number = easeOutQuart
): number {
  const frame = useCurrentFrame();
  return smoothValue(frame, startFrame, endFrame, startValue, endValue, easingFn);
}
