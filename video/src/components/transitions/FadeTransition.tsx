import { useCurrentFrame } from "remotion";
import { fade } from "../../lib/easing";

interface FadeTransitionProps {
  children: React.ReactNode;
  fadeInStart: number;
  fadeInEnd: number;
  fadeOutStart: number;
  fadeOutEnd: number;
}

export function FadeTransition({
  children,
  fadeInStart,
  fadeInEnd,
  fadeOutStart,
  fadeOutEnd,
}: FadeTransitionProps) {
  const frame = useCurrentFrame();
  const opacity = fade(frame, fadeInStart, fadeInEnd, fadeOutStart, fadeOutEnd);

  return <div style={{ opacity }}>{children}</div>;
}
