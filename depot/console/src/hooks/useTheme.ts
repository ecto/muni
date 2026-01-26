import { useEffect, useState, useCallback, useMemo } from "react";
import { useConsoleStore, type Theme } from "@/store";

/**
 * Hook to manage theme preference with system preference support.
 * Resolves 'system' to actual 'light' or 'dark' based on OS preference.
 */
export function useTheme() {
  const theme = useConsoleStore((s) => s.theme);
  const setTheme = useConsoleStore((s) => s.setTheme);
  const [systemPrefersDark, setSystemPrefersDark] = useState(() =>
    window.matchMedia("(prefers-color-scheme: dark)").matches
  );

  // Listen for system preference changes
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handler = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches);
    };

    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }, []);

  // Resolve 'system' to actual theme
  const resolvedTheme = useMemo((): "light" | "dark" => {
    if (theme === "system") {
      return systemPrefersDark ? "dark" : "light";
    }
    return theme;
  }, [theme, systemPrefersDark]);

  // Apply dark class to document element so portals inherit it
  useEffect(() => {
    if (resolvedTheme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [resolvedTheme]);

  // Cycle through themes: light → dark → system
  const cycleTheme = useCallback(() => {
    const order: Theme[] = ["light", "dark", "system"];
    const currentIndex = order.indexOf(theme);
    const nextIndex = (currentIndex + 1) % order.length;
    setTheme(order[nextIndex]);
  }, [theme, setTheme]);

  return {
    theme,
    resolvedTheme,
    setTheme,
    cycleTheme,
  };
}
