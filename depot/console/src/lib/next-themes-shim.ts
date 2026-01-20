// Shim for next-themes to use our own theme hook
import { useTheme as useAppTheme } from "@/hooks/useTheme";

export function useTheme() {
  const { theme, resolvedTheme, setTheme } = useAppTheme();
  return {
    theme,
    resolvedTheme,
    setTheme,
    themes: ["light", "dark", "system"],
    forcedTheme: undefined,
    systemTheme: resolvedTheme,
  };
}

// Re-export for compatibility
export const ThemeProvider = ({ children }: { children: React.ReactNode }) => children;
