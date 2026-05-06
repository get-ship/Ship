// Repo-root Vite config for the Ship asset pipeline.
//
// Replaces scripts/build-bundles.mjs (esbuild). Each call to `vite build`
// produces one bundle, selected by SHIP_BUNDLE_TARGET. The runner in
// scripts/build-bundles-vite.mjs walks the target list and invokes Vite
// once per target — Vite's lib mode is single-entry, and per-target
// externals + output paths differ enough that one config-per-target
// keeps the diff readable.
//
// Output is byte-stable with esbuild's layout (the daemon `include_str!`s
// these paths at compile time):
//   apps/shipd/assets/                  — Lit bundles
//     ship-elements.js
//     ship-code-editor.js
//     ship-markdown-editor.js
//     ship-terminal.js
//   crates/ship-ui-framework/assets/    — every SUI bundle
//     sui-runtime.js
//     sui-dom.js
//     sui-sdk-client.js
//     sui-components.js
//     sui-islands/<bundle>.js
//     views/<name>/view.js

import { defineConfig } from 'vite'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { resolve } from 'node:path'

import suiPlugin from './packages/vite-plugin-sui/src/index.js'

const repoRoot = __dirname
const litAssetsDir = resolve(repoRoot, 'apps/shipd/assets')
const suiAssetsDir = resolve(repoRoot, 'crates/ship-ui-framework/assets')

export interface BundleTarget {
  /** Logical name used as SHIP_BUNDLE_TARGET. Doubles as the output stem
   *  unless `outFile` is supplied (per-view bundles need a nested path). */
  name: string
  /** Real entry file. Per-island targets supply a virtual entry id;
   *  resolved by the inline plugin below. */
  entry: string
  /** Symbols vite/rollup should leave as `import …` lines instead of
   *  bundling. Mirrors esbuild's `external` for parity. */
  external?: string[]
  /** Override the output file path relative to the assets dir. Defaults
   *  to `<name>.js` under the appropriate dir for the target's name. */
  outFile?: string
  /** When set, the build output goes to this dir. Defaults to
   *  assetsDirFor(name). */
  outDirOverride?: string
  /** Synthetic entry module body. When supplied, the build pulls the
   *  entry from this string instead of disk — used by per-island bundles
   *  that aggregate every .sui in a directory under one synthetic
   *  `import './<stem>.compiled.js'` chain. */
  virtualEntry?: string
}

function assetsDirFor(name: string): string {
  if (name.startsWith('sui-') || name.startsWith('views/')) return suiAssetsDir
  return litAssetsDir
}

function discoverIslandBundles(): BundleTarget[] {
  const islandsDir = resolve(repoRoot, 'packages/ui-components/src/islands')
  if (!existsSync(islandsDir)) return []
  const out: BundleTarget[] = []
  for (const e of readdirSync(islandsDir, { withFileTypes: true })) {
    if (!e.isDirectory()) continue
    const dir = resolve(islandsDir, e.name)
    const stems = readdirSync(dir, { withFileTypes: true })
      .filter((f) => f.isFile() && f.name.endsWith('.sui'))
      .map((f) => f.name.replace(/\.sui$/, ''))
      .sort()
    if (stems.length === 0) continue
    out.push({
      name: `sui-islands/${e.name}`,
      // Virtual entry: one `import './<stem>.compiled.js'` line per stem
      // in the bundle directory. These imports trigger
      // `customElements.define()` for each component, registration by
      // side-effect.
      entry: resolve(dir, '.virtual-entry.ts'),
      virtualEntry:
        stems
          .map((s) => `import './dist/${s}.compiled.js'`)
          .join('\n') + '\n',
      external: ['@ship/ui-runtime', '@ship/ui-dom'],
      outFile: `sui-islands/${e.name}.js`,
    })
  }
  return out
}

export const TARGETS: BundleTarget[] = [
  // Lit / non-SUI bundles
  {
    name: 'ship-elements',
    entry: 'packages/ship-set/src/web-components/index.ts',
  },
  {
    name: 'ship-code-editor',
    entry: 'packages/ship-set/src/web-components/islands/ship-code-editor.ts',
  },
  {
    name: 'ship-markdown-editor',
    entry: 'packages/ship-set/src/web-components/islands/ship-markdown-editor.ts',
  },
  {
    name: 'ship-terminal',
    entry: 'packages/ship-set/src/web-components/islands/ship-terminal.ts',
  },

  // SUI runtime — every page loads sui-runtime.js once; downstream bundles
  // externalise it.
  { name: 'sui-runtime', entry: 'packages/ui-runtime/src/index.ts' },
  { name: 'sui-dom', entry: 'packages/ui-dom/src/index.ts', external: ['@ship/ui-runtime'] },
  { name: 'sui-sdk-client', entry: 'packages/ui-sdk-client/src/index.ts', external: ['@ship/ui-runtime'] },

  // Components: pre-compiled .sui → .compiled.js → bundled aggregate.
  // Pre-compile lives in scripts/build-bundles-vite.mjs.
  { name: 'sui-components', entry: 'packages/ui-components/src/index.ts', external: ['@ship/ui-runtime'] },

  // Per-island bundles (synthesized entries — see discoverIslandBundles).
  ...discoverIslandBundles(),

  // Per-view bundles. Each view's page.sui is pre-compiled to
  // <view>/dist/page.compiled.js; that's the entry vite bundles. The
  // compiled output `import setup from "../controller.ts"` is inlined.
  {
    name: 'views/agents/view',
    entry: 'views/agents/dist/page.compiled.js',
    external: ['@ship/ui-runtime', '@ship/ui-dom', '@ship/ui-sdk-client'],
    outFile: 'views/agents/view.js',
  },
  {
    name: 'views/counter/view',
    entry: 'views/counter/dist/page.compiled.js',
    external: ['@ship/ui-runtime', '@ship/ui-dom', '@ship/ui-sdk-client'],
    outFile: 'views/counter/view.js',
  },
  {
    name: 'views/git/view',
    entry: 'views/git/dist/page.compiled.js',
    external: ['@ship/ui-runtime', '@ship/ui-dom', '@ship/ui-sdk-client'],
    outFile: 'views/git/view.js',
  },
]

const targetName = process.env.SHIP_BUNDLE_TARGET ?? 'sui-runtime'
const target = TARGETS.find((t) => t.name === targetName)
if (!target) {
  throw new Error(
    `vite.config.ts: unknown SHIP_BUNDLE_TARGET=${targetName}. Known: ${TARGETS.map((t) => t.name).join(', ')}`,
  )
}

const outDir = target.outDirOverride ?? assetsDirFor(target.name)
const outFile = target.outFile ?? `${target.name}.js`

// Inline plugin that synthesises the entry body for per-island bundles.
const virtualEntryPlugin = target.virtualEntry
  ? {
      name: 'ship-virtual-entry',
      resolveId(id: string) {
        if (id === target.entry) return id
        return null
      },
      load(id: string) {
        if (id === target.entry) return target.virtualEntry
        return null
      },
    }
  : null

export default defineConfig({
  root: repoRoot,
  plugins: [suiPlugin(), ...(virtualEntryPlugin ? [virtualEntryPlugin] : [])],
  resolve: {
    // Order-sensitive: longer / more specific regexes must come before
    // the bare `@ship/ui-runtime` match. Vite walks the array top-down
    // and stops at the first hit.
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
      entry: target.virtualEntry ? target.entry : resolve(repoRoot, target.entry),
      formats: ['es'],
      fileName: () => outFile,
      name: target.name.replace(/[^a-zA-Z0-9]/g, '_'),
    },
    rollupOptions: {
      external: target.external,
      output: { inlineDynamicImports: true },
    },
  },
})
