#!/usr/bin/env node
// Capture a fresh perf baseline.
//
// Rebuilds the SUI benchmark bundle, runs the js-framework-benchmark suite
// against SUI + a fixed set of comparison frameworks, then aggregates the
// median results into tests/perf/baselines/<date>-baseline.json.
//
// See tests/perf/baselines/README.md.

import { execFileSync } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import {
  ensureBenchServer,
  rebuildSuiBundle,
  runBench,
  tableForFramework,
  today,
} from './bench-lib.mjs'

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..')

const FRAMEWORKS = ['keyed/sui', 'keyed/solid', 'keyed/svelte', 'keyed/vue', 'keyed/vanillajs']
const FRAMEWORK_KEYS = ['sui', 'solid', 'svelte', 'vue', 'vanillajs']

const sha = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: REPO_ROOT })
  .toString().trim()

console.log('▸ rebuilding SUI bundle')
rebuildSuiBundle()

console.log('▸ ensuring bench http-server')
await ensureBenchServer()

console.log(`▸ running benchmark for ${FRAMEWORKS.join(', ')}`)
runBench(FRAMEWORKS)

console.log('▸ aggregating results')
const tables = Object.fromEntries(FRAMEWORK_KEYS.map(k => [k, tableForFramework(k)]))

const baseline = {
  captured: today(),
  git_sha: sha,
  framework_version: '0.1.0',
  compared_against: FRAMEWORK_KEYS.filter(k => k !== 'sui'),
  operations: flattenByOp(tables, 'cpu'),
  memory_mb: flattenByOp(tables, 'memory'),
  size_kb: flattenByOp(tables, 'size'),
  first_paint_ms: flattenByOp(tables, 'paint'),
}

const outDir = resolve(REPO_ROOT, 'tests/perf/baselines')
mkdirSync(outDir, { recursive: true })
const outFile = resolve(outDir, `${baseline.captured}-baseline.json`)
writeFileSync(outFile, JSON.stringify(baseline, null, 2) + '\n')
console.log(`✓ wrote ${outFile.replace(REPO_ROOT + '/', '')}`)

const suiOps = Object.entries(baseline.operations)
  .map(([op, row]) => `  ${op.padEnd(28)} sui=${row.sui ?? '—'}  solid=${row.solid ?? '—'}`)
  .join('\n')
console.log('\nSUI vs Solid (median ms):\n' + suiOps)

function flattenByOp(tables, kind) {
  const ops = new Set()
  for (const t of Object.values(tables)) for (const op of Object.keys(t[kind])) ops.add(op)
  const out = {}
  for (const op of ops) {
    out[op] = {}
    for (const [fw, t] of Object.entries(tables)) {
      const v = t[kind][op]
      if (v !== null) out[op][fw] = v
    }
  }
  return out
}
