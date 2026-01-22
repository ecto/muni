import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import path from "path";

export default defineConfig({
  plugins: [react(), tailwindcss(), wasm(), topLevelAwait()],
  base: "./",
  optimizeDeps: {
    exclude: ["@rerun-io/web-viewer"],
    // Don't scan viewer.html in public/rerun - it's a standalone page
    entries: ["index.html", "src/**/*.{ts,tsx}"],
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "next-themes": path.resolve(__dirname, "./src/lib/next-themes-shim.ts"),
    },
  },
  server: {
    fs: {
      // Allow serving files from node_modules/@rerun-io
      allow: [".", "../node_modules/@rerun-io"],
    },
    proxy: {
      // Proxy API requests to depot server during development
      "/api/discovery": {
        target: "http://depot:4860",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/discovery/, ""),
        ws: true,
      },
      "/api/gps": {
        target: "http://depot:4880",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/gps/, ""),
        ws: true,
      },
      "/api/maps": {
        target: "http://depot:4870",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/maps/, ""),
      },
      "/api/mapper": {
        target: "http://depot:4895",
        changeOrigin: true,
        // Strip /api/mapper prefix and also strip .rrd suffix for Rerun compatibility
        rewrite: (path) => path.replace(/^\/api\/mapper/, "").replace(/\.rrd$/, ""),
      },
      "/api/dispatch": {
        target: "http://depot:4890",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/dispatch/, ""),
        ws: true,
      },
      "/grafana": {
        target: "http://depot:3000",
        changeOrigin: true,
      },
    },
  },
});
