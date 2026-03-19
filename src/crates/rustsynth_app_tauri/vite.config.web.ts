import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import path from "path";

export default defineConfig({
  plugins: [react(), wasm()],
  resolve: {
    alias: {
      "rustsynth-wasm": path.resolve(__dirname, "../rustsynth_wasm/pkg"),
    },
  },
  server: {
    port: 3000,
    fs: {
      allow: [
        ".",
        "../rustsynth_wasm/pkg",
      ],
    },
  },
  build: {
    outDir: "dist-web",
  },
});
