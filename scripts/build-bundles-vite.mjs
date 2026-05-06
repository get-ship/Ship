#!/usr/bin/env bun
// Drive Vite once per asset target. Mirrors the per-target invocation
// pattern of scripts/build-bundles.mjs (esbuild) — but every target
// goes through Vite + @ship/vite-plugin-sui.
//
// Targets:
//   Lit:       ship-elements, ship-code-editor, ship-markdown-editor, ship-terminal
//   SUI core:  sui-runtime, sui-dom, sui-sdk-client, sui-components
//   Islands:   sui-islands/<bundle> (auto-discovered from packages/ui-components/src/islands/)
//   Views:     views/<name>/view (currently agents, counter, git)
//
// Pre-compile step: targets that aggregate .compiled.js files
// (sui-components, sui-islands/*, views/*/view) need their .sui sources
// run through sui-compiler before bundling. This script does that walk
// up-front; Step 5c will refactor the entries to import .sui directly
// and let vite-plugin-sui drive everything via its transform hook.
//
// Usage:
//   bun scripts/build-bundles-vite.mjs                  # build every target
//   bun scripts/build-bundles-vite.mjs --target sui-dom # one target
//   bun scripts/build-bundles-vite.mjs --only-precompile

import { spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')

// We mirror the target list from vite.config.ts statically here — a
// dynamic import at runtime would force us to ship the config through a
// TS loader. The list shape is small and the pair stays in sync via
// CI bundle-shape checks (Step 7).
const STATIC_TARGETS = [
  'ship-elements',
  'ship-code-editor',
  'ship-markdown-editor',
  'ship-terminal',
  'sui-runtime',
  'sui-dom',
  'sui-sdk-client',
  'sui-components',
  'views/agents/view',
  'views/counter/view',
  'views/git/view',
]

function discoverIslandTargets() {
  const dir = join(repoRoot, 'packages/ui-components/src/islands')
  if (!existsSync(dir)) return []
  const out = []
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    if (e.isDirectory()) out.push(`sui-islands/${e.name}`)
  }
  return out
}

const TARGETS = [...STATIC_TARGETS, ...discoverIslandTargets()]

// Pre-compile rules: target name → list of .sui sources to compile + the
// component-mode flag (when true, emits a registered customElement).
const PRECOMPILE = {
  'sui-components': () =>
    collectSuiSources(join(repoRoot, 'packages/ui-components/src'), {
      excludeDir: 'islands',
      mode: 'component',
    }),
  'views/agents/view': () => [{ path: join(repoRoot, 'views/agents/page.sui'), mode: 'view' }],
  'views/counter/view': () => [{ path: join(repoRoot, 'views/counter/page.sui'), mode: 'view' }],
  'views/git/view': () => [{ path: join(repoRoot, 'views/git/page.sui'), mode: 'view' }],
}
// Per-island targets share a discovery rule: every .sui in
// packages/ui-components/src/islands/<bundle>/ as a component.
for (const t of discoverIslandTargets()) {
  const bundle = t.slice('sui-islands/'.length)
  PRECOMPILE[t] = () =>
    collectSuiSources(join(repoRoot, `packages/ui-components/src/islands/${bundle}`), {
      mode: 'component',
    })
}

function collectSuiSources(rootDir, { excludeDir, mode }) {
  const out = []
  if (!existsSync(rootDir)) return out
  const stack = [rootDir]
  while (stack.length > 0) {
    const dir = stack.pop()
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      if (e.isDirectory()) {
        if (e.name === excludeDir) continue
        if (e.name === 'dist') continue
        stack.push(join(dir, e.name))
      } else if (e.isFile() && e.name.endsWith('.sui')) {
        out.push({ path: join(dir, e.name), mode })
      }
    }
  }
  return out
}

function precompileSources(sources) {
  if (sources.length === 0) return
  const compiler = resolve(repoRoot, 'target/release/sui-compiler')
  if (!existsSync(compiler)) {
    console.error(`[bundles-vite] missing ${compiler} — run \`cargo build -p sui-compiler --release\``)
    process.exit(1)
  }
  for (const { path: sui, mode } of sources) {
    const stem = sui.split('/').pop().replace(/\.sui$/, '')
    const args = ['build', sui, '--no-dts']
    if (mode === 'component') args.push('--component-name', stem)
    const r = spawnSync(compiler, args, { stdio: 'inherit' })
    if (r.status !== 0) {
      console.error(`[bundles-vite] sui-compiler failed for ${sui}`)
      process.exit(r.status ?? 1)
    }
  }
}

function aggregateCem() {
  // Mirrors the same pass in build-bundles.mjs: collects per-component
  // CEMs into one sui-custom-elements.json the daemon serves.
  const srcDir = join(repoRoot, 'packages/ui-components/src')
  const out = { schemaVersion: '1.0.0', readme: '', modules: [] }
  for (const e of readdirSync(srcDir, { withFileTypes: true })) {
    if (!e.isDirectory() || e.name === 'islands') continue
    const cemPath = join(srcDir, e.name, 'dist', `${e.name}.custom-elements.json`)
    if (!existsSync(cemPath)) continue
    try {
      const cem = JSON.parse(readFileSync(cemPath, 'utf8'))
      if (Array.isArray(cem.modules)) out.modules.push(...cem.modules)
    } catch {
      // Skip malformed CEMs; surfaced by the next compile.
    }
  }
  const target = join(repoRoot, 'crates/ship-ui-framework/assets/sui-custom-elements.json')
  mkdirSync(dirname(target), { recursive: true })
  const next = JSON.stringify(out)
  let prev = ''
  try {
    prev = readFileSync(target, 'utf8')
  } catch {
    // First run: prev stays empty.
  }
  if (prev !== next) {
    writeFileSync(target, next)
    console.log(`[bundles-vite] write  sui-custom-elements.json  (${out.modules.length} modules)`)
  }
}

const args = process.argv.slice(2)
const targetIndex = args.indexOf('--target')
const wanted = targetIndex >= 0 && args[targetIndex + 1] ? [args[targetIndex + 1]] : TARGETS
const onlyPrecompile = args.includes('--only-precompile')

const viteEntry = resolve(repoRoot, 'node_modules/vite/bin/vite.js')

for (const t of wanted) {
  if (!TARGETS.includes(t)) {
    console.error(`unknown target: ${t}. known: ${TARGETS.join(', ')}`)
    process.exit(1)
  }
  if (PRECOMPILE[t]) {
    const sources = PRECOMPILE[t]()
    console.log(`[bundles-vite] precompile ${sources.length} .sui for ${t}`)
    precompileSources(sources)
  }
  if (onlyPrecompile) continue
  console.log(`[bundles-vite] build ${t}`)
  const r = spawnSync(process.execPath, [viteEntry, 'build', '--logLevel', 'warn'], {
    cwd: repoRoot,
    stdio: 'inherit',
    env: { ...process.env, SHIP_BUNDLE_TARGET: t },
  })
  if (r.status !== 0) {
    console.error(`[bundles-vite] ${t} failed (exit ${r.status})`)
    process.exit(r.status ?? 1)
  }
}

if (!onlyPrecompile) aggregateCem()

console.log(`[bundles-vite] done (${wanted.length} target${wanted.length === 1 ? '' : 's'})`)
