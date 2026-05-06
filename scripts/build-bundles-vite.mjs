#!/usr/bin/env bun
// Step 5 (a): drive Vite once per SUI runtime target.
//
// Replaces the shared-runtime subset of scripts/build-bundles.mjs:
//   - sui-runtime.js
//   - sui-dom.js
//   - sui-sdk-client.js
//
// Lit / per-component / per-island / per-view bundles still flow through
// the legacy esbuild script. Step 5b extends Vite to those targets once
// parity is verified here.
//
// Usage:
//   bun scripts/build-bundles-vite.mjs                # build all SUI runtime targets
//   bun scripts/build-bundles-vite.mjs --target sui-dom

import { spawnSync } from 'node:child_process'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')

const TARGETS = ['sui-runtime', 'sui-dom', 'sui-sdk-client']

const args = process.argv.slice(2)
const targetIndex = args.indexOf('--target')
const wanted =
  targetIndex >= 0 && args[targetIndex + 1]
    ? [args[targetIndex + 1]]
    : TARGETS

for (const t of wanted) {
  if (!TARGETS.includes(t)) {
    console.error(`unknown target: ${t}. known: ${TARGETS.join(', ')}`)
    process.exit(1)
  }
  console.log(`[bundles-vite] build ${t}`)
  // Resolve vite's JS entry directly — `node_modules/.bin/vite` is a
  // POSIX shell script that bun's parser refuses to run, and `bun x` /
  // `process.execPath` against the shim hits the same wall. The JS file
  // is the canonical entry the shim wraps anyway.
  const viteEntry = resolve(repoRoot, 'node_modules/vite/bin/vite.js')
  const r = spawnSync(
    process.execPath,
    [viteEntry, 'build', '--logLevel', 'warn'],
    {
      cwd: repoRoot,
      stdio: 'inherit',
      env: { ...process.env, SHIP_BUNDLE_TARGET: t },
    },
  )
  if (r.status !== 0) {
    console.error(`[bundles-vite] ${t} failed (exit ${r.status})`)
    process.exit(r.status ?? 1)
  }
}

console.log(`[bundles-vite] done (${wanted.length} target${wanted.length === 1 ? '' : 's'})`)
