/// <reference types="vitest" />
import { defineConfig } from "vite";
import path from "path";
import react from "@vitejs/plugin-react";

import comlink from "vite-plugin-comlink";
import worker, { pluginHelper } from "vite-plugin-worker";

export default defineConfig({
  plugins: [
    react(),
    {
      name: "configure-response-headers",
      configureServer: (server) => {
        server.middlewares.use((_req, res, next) => {
          res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
          res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
          next();
        });
      },
    },
    comlink({ typeFile: "comlink.d.ts" }),
    pluginHelper(),
    worker({}),
  ],
  build: {
    rollupOptions: {
      input: {
        lib: path.resolve(__dirname, "src/main.tsx"),
        run: path.resolve(__dirname, "deno/run.ts"),
        wasm: path.resolve(__dirname, "wasm_dist/wasm.js"),
      },
    },
  },
  // @ts-ignore
  test: {
    dir: "./test",
  },
});
