import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// 48V lithium battery: 13S config, 39V empty (3.0V/cell), 54.6V full (4.2V/cell)
const BATTERY_MIN_VOLTAGE = 39;
const BATTERY_MAX_VOLTAGE = 54.6;

export function getBatteryPercent(voltage: number): number {
  return Math.max(
    0,
    Math.min(100, ((voltage - BATTERY_MIN_VOLTAGE) / (BATTERY_MAX_VOLTAGE - BATTERY_MIN_VOLTAGE)) * 100)
  );
}
