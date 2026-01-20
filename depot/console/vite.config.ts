import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  base: "./",
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "next-themes": path.resolve(__dirname, "./src/lib/next-themes-shim.ts"),
    },
  },
  server: {
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
      "/grafana": {
        target: "http://depot:3000",
        changeOrigin: true,
      },
    },
  },
});
