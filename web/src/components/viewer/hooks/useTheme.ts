"use client";

import { useState, useEffect } from "react";

export interface ThemeColors {
  background: number;
  gridCell: number;
  gridSection: number;
}

const darkTheme: ThemeColors = {
  background: 0x0a0a0a,
  gridCell: 0x3a3634,
  gridSection: 0x57534e,
};

const lightTheme: ThemeColors = {
  background: 0xf5f5f4,
  gridCell: 0xd6d3d1,
  gridSection: 0xa8a29e,
};

export function useTheme() {
  const [theme, setTheme] = useState<ThemeColors>(darkTheme);

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const updateTheme = () => {
      setTheme(mediaQuery.matches ? darkTheme : lightTheme);
    };

    updateTheme();
    mediaQuery.addEventListener("change", updateTheme);

    return () => mediaQuery.removeEventListener("change", updateTheme);
  }, []);

  return theme;
}
