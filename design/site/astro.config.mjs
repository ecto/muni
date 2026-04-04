import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://design.municipal.bot",
  outDir: "dist",
  vite: {
    server: {
      allowedHosts: ["mew"],
    },
    resolve: {
      alias: {
        "@tokens": new URL("../tokens", import.meta.url).pathname,
      },
    },
  },
});
