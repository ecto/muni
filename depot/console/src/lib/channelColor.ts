/** Determine color based on channel health. */
export function getChannelColor(
  messagesPerSec: number,
  lastMessageTime: number,
  isOpen: boolean
): string {
  if (!isOpen) return "#6b7280"; // Gray - channel not open

  const now = performance.now();
  const staleness = (now - lastMessageTime) / 1000; // seconds

  if (lastMessageTime === 0) return "#6b7280"; // Gray - no messages yet
  if (staleness > 2) return "#ef4444"; // Red - stale
  if (staleness > 1) return "#f59e0b"; // Yellow - warning
  if (messagesPerSec > 0) return "#22c55e"; // Green - healthy
  return "#f59e0b"; // Yellow - no recent messages
}
