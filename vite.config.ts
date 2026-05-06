// Repo-root Vite config for the Ship asset pipeline.
//
// Status (Step 5 of vite-plugin-bun): replaces the SUI subset of
// scripts/build-bundles.mjs — the four shared-runtime bundles and the
// per-component aggregate. The Lit ship-elements bundles + per-island +
// per-view bundles still flow through esbuild via the legacy script;
// they migrate in Step 5b once we've validated parity here.
//
// Output shape:
//   crates/ship-ui-framework/assets/
//     sui-runtime.js          # @ship/ui-runtime
//     sui-dom.js              # @ship/ui-dom (externalises @ship/ui-runtime)
//     sui-sdk-client.js       # @ship/ui-sdk-client (externalises @ship/ui-runtime)
//     sui-components.js       # all .sui components, compiled + bundled
//
// The ship-ui-framework crate `include_str!`s these at compile time, so
// keeping the paths byte-stable is non-negotiable. Bundle byte sizes must
// stay within 10 % of the esbuild baseline (job-spec acceptance #8).

import { defineConfig } from 'vite'
import { resolve } from 'node:path'

import suiPlugin from './packages/vite-plugin-sui/src/index.js'

const repoRoot = __dirname
const outDir = resolve(repoRoot, 'crates/ship-ui-framework/assets')

// Each target maps to a Vite "lib" entry. `external` mirrors the esbuild
// config in scripts/build-bundles.mjs — keeping the runtime split intact
// (sui-runtime.js loads once on every page; downstream bundles reach into
// it via the runtime registry rather than re-bundling).
//
// `name` is required by Vite's lib mode but is meaningless for ESM-only
// output (no UMD global is emitted). We use the file stem as the name to
// keep grep-ability.

export interface BundleTarget {
  name: string
  entry: string
  external?: string[]
}

export const SUI_RUNTIME_TARGETS: BundleTarget[] = [
  {
    name: 'sui-runtime',
    entry: 'packages/ui-runtime/src/index.ts',
  },
  {
    name: 'sui-dom',
    entry: 'packages/ui-dom/src/index.ts',
    external: ['@ship/ui-runtime'],
  },
  {
    name: 'sui-sdk-client',
    entry: 'packages/ui-sdk-client/src/index.ts',
    external: ['@ship/ui-runtime'],
  },
]

// Default config — single-target build driven by SHIP_BUNDLE_TARGET env var.
// `scripts/build-bundles-vite.mjs` invokes `vite build` once per target,
// passing the target's name. This mirrors how esbuild ran one build per
// target, but in Vite's idiom (Vite's lib mode is single-entry per build).
const targetName = process.env.SHIP_BUNDLE_TARGET ?? 'sui-runtime'
const target = SUI_RUNTIME_TARGETS.find((t) => t.name === targetName)
if (!target) {
  throw new Error(
    `vite.config.ts: unknown SHIP_BUNDLE_TARGET=${targetName}. ` +
      `Known: ${SUI_RUNTIME_TARGETS.map((t) => t.name).join(', ')}`,
  )
}

export default defineConfig({
  root: repoRoot,
  plugins: [suiPlugin()],
  resolve: {
    // Order-sensitive: longer / more specific regexes must come before the
    // bare `@ship/ui-runtime` match. Vite walks the array top-down and stops
    // at the first hit. A bare string alias for `@ship/ui-runtime` would do
    // a prefix-match and swallow the `/patch`, `/route`, etc. subpaths,
    // routing them through index.ts (ENOTDIR).
    alias: [
      { find: /^@ship\/ui-runtime\/template-hooks$/, replacement: resolve(repoRoot, 'packages/ui-runtime/src/template-hooks/index.ts') },
      { find: /^@ship\/ui-runtime\/dev$/, replacement: resolve(repoRoot, 'packages/ui-runtime/src/dev.ts') },
      { find: /^@ship\/ui-runtime\/route$/, replacement: resolve(repoRoot, 'packages/ui-runtime/src/route/index.ts') },
      { find: /^@ship\/ui-runtime\/patch$/, replacement: resolve(repoRoot, 'packages/ui-runtime/src/patch.ts') },
      { find: /^@ship\/ui-runtime$/, replacement: resolve(repoRoot, 'packages/ui-runtime/src/index.ts') },
      { find: /^@ship\/ui-dom$/, replacement: resolve(repoRoot, 'packages/ui-dom/src/index.ts') },
      { find: /^@ship\/ui-sdk-client$/, replacement: resolve(repoRoot, 'packages/ui-sdk-client/src/index.ts') },
    ],
  },
  build: {
    outDir,
    emptyOutDir: false,
    sourcemap: false,
    minify: false,
    target: 'es2020',
    lib: {
      entry: resolve(repoRoot, target.entry),
      formats: ['es'],
      fileName: () => `${target.name}.js`,
      // Lib mode requires a `name` even for ESM-only — Vite reads it for
      // potential UMD globals it never emits.
      name: target.name.replace(/[^a-zA-Z0-9]/g, '_'),
    },
    rollupOptions: {
      external: target.external,
      output: {
        // Single chunk per build — no auto code-splitting. Matches esbuild
        // `bundle: true` semantics and keeps the daemon's `include_str!()`
        // pointing at one file per target.
        inlineDynamicImports: true,
      },
    },
  },
})
