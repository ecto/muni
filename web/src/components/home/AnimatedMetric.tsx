"use client";

import { AnimatedCounter } from "./AnimatedCounter";

interface AnimatedMetricProps {
  end: number;
  suffix?: string;
  prefix?: string;
}

export function AnimatedMetric({ end, suffix, prefix }: AnimatedMetricProps) {
  return <AnimatedCounter end={end} suffix={suffix} prefix={prefix} />;
}
