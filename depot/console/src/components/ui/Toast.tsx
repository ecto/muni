import { useConsoleStore } from "@/store";

/**
 * Simple toast notification that appears briefly in the center of the screen.
 */
export function Toast() {
  const toast = useConsoleStore((s) => s.toast);

  if (!toast) return null;

  return (
    <div className="fixed inset-0 pointer-events-none flex items-center justify-center z-50">
      <div className="bg-card/95 backdrop-blur border border-border px-6 py-3 rounded-lg shadow-lg animate-in fade-in zoom-in-95 duration-200">
        <span className="text-foreground font-medium">{toast}</span>
      </div>
    </div>
  );
}
