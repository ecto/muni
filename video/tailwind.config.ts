import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        muni: {
          orange: "#ff6600",
          dark: "#0a0a0a",
        },
      },
      fontFamily: {
        mono: ['"Berkeley Mono"', "ui-monospace", "monospace"],
      },
    },
  },
  plugins: [],
} satisfies Config;
