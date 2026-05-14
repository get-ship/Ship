#!/usr/bin/env node
// Compare current SUI perf vs the latest committed baseline.
//
// Re-runs the benchmark against keyed/sui only, reads the medians, and
// computes (current - baseline) / baseline for every op. Exits 1 if any op
// regresses by more than BENCH_REGRESSION_THRESHOLD (default 0.15 = 15%).
//
// Skips comparison-framework runs for speed — only the gate matters here.

import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import {
  ensureBenchServer,
  latestBaseline,
  rebuildSuiBundle,
  runBench,
  tableForFramework,
} from './bench-lib.mjs'

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..')
const THRESHOLD = Number(process.env.BENCH_REGRESSION_THRESHOLD || '0.15')

const baselinePath = latestBaseline(REPO_ROOT)
if (!baselinePath) {
  console.error('no baseline found under tests/perf/baselines/. Run `just bench` first.')
  process.exit(1)
}
const baseline = JSON.parse(readFileSync(baselinePath, 'utf8'))
console.log(`▸ baseline: ${baselinePath.replace(REPO_ROOT + '/', '')} (captured ${baseline.captured} @ ${baseline.git_sha})`)

if (process.env.BENCH_SKIP_RUN === '1') {
  console.log('▸ BENCH_SKIP_RUN=1 — reading existing results without rebuild/rerun')
} else {
  console.log('▸ rebuilding SUI bundle')
  rebuildSuiBundle()

  console.log('▸ ensuring bench http-server')
  await ensureBenchServer()

  console.log('▸ running benchmark (keyed/sui only)')
  runBench(['keyed/sui'])
}

const current = tableForFramework('sui')

const sections = [
  { label: 'cpu (ms)',     kind: 'cpu',    baseline: baseline.operations },
  { label: 'memory (mb)',  kind: 'memory', baseline: baseline.memory_mb },
  { label: 'size (kb)',    kind: 'size',   baseline: baseline.size_kb },
  { label: 'paint (ms)',   kind: 'paint',  baseline: baseline.first_paint_ms || {} },
]

const RED = '\x1b[31m'
const GREEN = '\x1b[32m'
const DIM = '\x1b[2m'
const RESET = '\x1b[0m'

let regressed = 0
let checked = 0
for (const section of sections) {
  console.log(`\n${section.label}`)
  console.log(`  ${'op'.padEnd(28)} ${'baseline'.padStart(10)} ${'current'.padStart(10)} ${'delta'.padStart(10)}`)
  for (const [op, row] of Object.entries(section.baseline)) {
    const base = row.sui
    if (typeof base !== 'number') continue
    const cur = current[section.kind][op]
    if (typeof cur !== 'number') {
      console.log(`  ${op.padEnd(28)} ${String(base).padStart(10)} ${'(missing)'.padStart(10)}`)
      continue
    }
    checked++
    const ratio = (cur - base) / base
    const pct = (ratio * 100).toFixed(1) + '%'
    const color = ratio > THRESHOLD ? RED : ratio < -0.05 ? GREEN : DIM
    if (ratio > THRESHOLD) regressed++
    console.log(`  ${op.padEnd(28)} ${String(base).padStart(10)} ${String(cur).padStart(10)} ${color}${pct.padStart(10)}${RESET}`)
  }
}

console.log('')
if (checked === 0) {
  console.error('no comparable ops between baseline and current run')
  process.exit(1)
}
if (regressed > 0) {
  console.error(`✗ ${regressed} op(s) regressed by more than ${(THRESHOLD * 100).toFixed(0)}%`)
  process.exit(1)
}
console.log(`✓ all ${checked} ops within ±${(THRESHOLD * 100).toFixed(0)}% of baseline`)
