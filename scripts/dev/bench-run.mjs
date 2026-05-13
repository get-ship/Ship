#!/usr/bin/env node
// Capture a fresh perf baseline.
//
// Rebuilds the SUI benchmark bundle, runs the js-framework-benchmark suite
// against SUI + a fixed set of comparison frameworks, then aggregates the
// median results into tests/perf/baselines/<date>-<label>.json.
//
// Usage:
//   node scripts/dev/bench-run.mjs --label <name>
//
//   --label is REQUIRED. The output filename is always
//   `<YYYY-MM-DD>-<label>.json` — there is no implicit "baseline" suffix.
//   This prevents post-cycle captures from silently overwriting the
//   historical anchor baseline that lives under the same date.
//
// Examples:
//   node scripts/dev/bench-run.mjs --label baseline       # anchor capture
//   node scripts/dev/bench-run.mjs --label post-d5        # post-change capture
//   node scripts/dev/bench-run.mjs --label post-mem-fix   # follow-up capture
//
// See tests/perf/baselines/README.md.

import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
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

const label = parseLabel(process.argv.slice(2))

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
  label,
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
const outFile = resolve(outDir, `${baseline.captured}-${label}.json`)
if (existsSync(outFile)) {
  console.error(`✗ ${outFile.replace(REPO_ROOT + '/', '')} already exists. Pick a different --label or delete the file first.`)
  process.exit(1)
}
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

function parseLabel(argv) {
  let value = null
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--label') {
      value = argv[i + 1] ?? null
      i++
    } else if (a.startsWith('--label=')) {
      value = a.slice('--label='.length)
    } else {
      console.error(`✗ unknown argument: ${a}`)
      printUsage()
      process.exit(2)
    }
  }
  if (!value) {
    console.error('✗ --label <name> is required (output is tests/perf/baselines/<date>-<label>.json)')
    printUsage()
    process.exit(2)
  }
  if (!/^[a-z0-9][a-z0-9-]*$/i.test(value)) {
    console.error(`✗ --label must match [a-z0-9][a-z0-9-]* (got: ${JSON.stringify(value)})`)
    process.exit(2)
  }
  return value
}

function printUsage() {
  console.error('  usage: node scripts/dev/bench-run.mjs --label <name>')
  console.error('  examples:')
  console.error('    --label baseline')
  console.error('    --label post-d5')
  console.error('    --label post-mem-fix')
}
