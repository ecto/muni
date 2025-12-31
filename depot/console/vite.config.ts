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
    },
  },
  server: {
    proxy: {
      // Proxy API requests to internal services during development
      "/api/discovery": {
        target: "http://localhost:4860",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/discovery/, ""),
      },
      "/api/maps": {
        target: "http://localhost:4870",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/maps/, ""),
      },
    },
  },
});
