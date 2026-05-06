import { defineConfig } from "vite";
import legacy from "@vitejs/plugin-legacy";
import { resolve } from "node:path";

import suiPlugin from "./vite-plugin-sui.ts";

const repoRoot = resolve(__dirname, "../..");
const compilerPath = resolve(repoRoot, "target/release/sui-compiler");

export default defineConfig({
  root: __dirname,
  plugins: [
    suiPlugin({ compilerPath }),
    // Tests plugin composition under Bun: a non-trivial third-party plugin
    // that adds a transform stage and additional rollup outputs.
    legacy({ targets: ["defaults", "not IE 11"] }),
  ],
  resolve: {
    alias: {
      "@ship/ui-runtime": resolve(repoRoot, "packages/ui-runtime/src/index.ts"),
      "@ship/ui-dom": resolve(repoRoot, "packages/ui-dom/src/index.ts"),
    },
  },
  server: {
    port: 5180,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
