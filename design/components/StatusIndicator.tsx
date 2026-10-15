type Status = "complete" | "active" | "pending" | "error" | "offline";

interface StatusIndicatorProps {
  status: Status;
  label?: string;
  size?: "sm" | "md";
}

const statusStyles: Record<Status, string> = {
  complete: "bg-green-500",
  active: "bg-orange-500 animate-pulse",
  pending: "bg-fg-muted",
  error: "bg-red-600 animate-pulse",
  offline: "border border-fg-muted bg-transparent",
};

export function StatusIndicator({
  status,
  label,
  size = "md",
}: StatusIndicatorProps) {
  const dotSize = size === "sm" ? "w-1.5 h-1.5" : "w-2 h-2";

  return (
    <div className="inline-flex items-center gap-2">
      <span className={`${dotSize} ${statusStyles[status]}`} />
      {label && (
        <span className="text-sm">{label}</span>
      )}
    </div>
  );
}
