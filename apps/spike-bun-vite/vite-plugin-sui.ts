// Spike-local sui plugin. Step 2 will extract this to packages/vite-plugin-sui/.
// For now it lives in-app so the spike validates the Bun + Vite + plugin
// composition without prematurely committing to a published-package surface.

import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";

import type { Plugin } from "vite";

interface Options {
  /** Absolute path to the sui-compiler binary. */
  compilerPath: string;
}

export default function suiPlugin(opts: Options): Plugin {
  const compile = (file: string): { code: string } => {
    // sui-compiler reads from disk — we feed it the file path directly and
    // collect emitted JS from a per-call temp dir. Vite caches transform
    // results, so this only fires on first import + on edits.
    const out = mkdtempSync(join(tmpdir(), "sui-spike-"));
    try {
      const r = spawnSync(
        opts.compilerPath,
        ["build", file, "--out", out, "--no-dts"],
        { stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" },
      );
      if (r.status !== 0) {
        const stderr = (r.stderr || "").toString();
        throw new Error(
          `sui-compiler failed (exit ${r.status}) for ${file}:\n${stderr}`,
        );
      }
      const stem = basename(file).replace(/\.sui$/, "");
      const jsPath = join(out, `${stem}.js`);
      const code = readFileSync(jsPath, "utf8");
      return { code };
    } finally {
      rmSync(out, { recursive: true, force: true });
    }
  };

  return {
    name: "vite-plugin-sui-spike",

    config() {
      return {
        resolve: { extensions: [".ts", ".js", ".sui"] },
      };
    },

    async transform(_src, id) {
      if (!id.endsWith(".sui")) return null;
      const { code } = compile(id);
      // Source maps will land in step 2; the spike validates plugin
      // composition, not source-map fidelity.
      return { code, map: null };
    },

    async handleHotUpdate(ctx) {
      if (!ctx.file.endsWith(".sui")) return;
      // Default Vite behavior — invalidate the changed module — already
      // re-fires `transform`. Returning ctx.modules is explicit but
      // semantically a no-op here.
      return ctx.modules;
    },
  };
}
