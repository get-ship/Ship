// Shared helpers for bench-run.mjs / bench-check.mjs.
// No external deps. Node 20+.

import { execFileSync, spawn } from 'node:child_process'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { homedir } from 'node:os'
import { resolve } from 'node:path'

export const BENCH_HOME = process.env.BENCH_HOME || resolve(homedir(), 'dev/js-framework-benchmark')
export const CHROME_BINARY = process.env.CHROME_BINARY
  || resolve(homedir(), '.cache/ms-playwright/chromium-1217/chrome-linux64/chrome')

export const SUI_FRAMEWORK_DIR = resolve(BENCH_HOME, 'frameworks/keyed/sui')
export const RUNNER = resolve(BENCH_HOME, 'webdriver-ts/dist/benchmarkRunner.js')
export const RESULTS_DIR = resolve(BENCH_HOME, 'webdriver-ts/results')

// Canonical ordered op list for the baseline + check tables.
export const CPU_OPS = [
  '01_run1k',
  '02_replace1k',
  '03_update10th1k_x16',
  '04_select1k',
  '05_swap1k',
  '06_remove-one-1k',
  '08_create1k-after1k_x2',
  '09_clear1k_x8',
]
export const MEMORY_OPS = ['21_ready-memory', '22_run-memory', '25_run-clear-memory']
export const SIZE_OPS = ['41_size-uncompressed', '42_size-compressed']
export const PAINT_OPS = ['43_first-paint']

export const FRAMEWORK_PREFIX = {
  sui: 'sui-v',
  solid: 'solid-v',
  svelte: 'svelte-v',
  vue: 'vue-v',
  vanillajs: 'vanillajs-keyed',
  react: 'react-hooks-v',
}

// Find the result file for a framework + op. Returns absolute path or null.
export function findResultFile(framework, op) {
  const prefix = FRAMEWORK_PREFIX[framework]
  if (!prefix) throw new Error(`unknown framework: ${framework}`)
  if (!existsSync(RESULTS_DIR)) return null
  for (const f of readdirSync(RESULTS_DIR)) {
    if (f.startsWith(prefix) && f.endsWith(`_${op}.json`)) return resolve(RESULTS_DIR, f)
  }
  return null
}

// Read median value out of a single result file. Returns null if missing.
export function readMedian(framework, op) {
  const path = findResultFile(framework, op)
  if (!path) return null
  const data = JSON.parse(readFileSync(path, 'utf8'))
  // CPU ops -> values.total.median; memory/size/paint -> values.DEFAULT.median.
  const bucket = data.values?.total || data.values?.DEFAULT
  if (!bucket || typeof bucket.median !== 'number') return null
  return round(bucket.median)
}

export function round(n) {
  return Math.round(n * 10) / 10
}

// Build the operations table for one framework.
export function tableForFramework(framework) {
  const out = { cpu: {}, memory: {}, size: {}, paint: {} }
  for (const op of CPU_OPS) out.cpu[op] = readMedian(framework, op)
  for (const op of MEMORY_OPS) out.memory[op] = readMedian(framework, op)
  for (const op of SIZE_OPS) out.size[op] = readMedian(framework, op)
  for (const op of PAINT_OPS) out.paint[op] = readMedian(framework, op)
  return out
}

// Rebuild the SUI bundle in the benchmark framework dir.
// Throws on failure (build errors bubble up).
export function rebuildSuiBundle() {
  execFileSync('npm', ['run', 'build-prod'], {
    cwd: SUI_FRAMEWORK_DIR,
    stdio: 'inherit',
  })
}

// Ensure the bench http-server is up. If not, start it as a detached child.
// Returns { started: bool, child }.
export async function ensureBenchServer() {
  if (await isUp('http://localhost:8080/index.html')) return { started: false, child: null }
  const child = spawn('npm', ['start'], {
    cwd: BENCH_HOME,
    stdio: 'ignore',
    detached: true,
  })
  child.unref()
  for (let i = 0; i < 30; i++) {
    await sleep(500)
    if (await isUp('http://localhost:8080/index.html')) return { started: true, child }
  }
  throw new Error('bench http-server failed to come up on :8080')
}

async function isUp(url) {
  try {
    const r = await fetch(url, { method: 'HEAD' })
    return r.ok
  } catch {
    return false
  }
}

function sleep(ms) { return new Promise(r => setTimeout(r, ms)) }

// Run the benchmark runner against a list of framework paths (e.g. "keyed/sui").
// Tolerates non-zero exit: js-framework-benchmark exits 1 if ANY sub-benchmark
// fails plausibility (e.g. the known 07_create10k mousedown harness issue),
// even when the ops we aggregate succeeded and wrote result JSONs. The
// aggregator + checker speak from the data — missing ops surface there.
export function runBench(frameworks) {
  try {
    execFileSync(
      'node',
      [
        RUNNER,
        '--framework', ...frameworks,
        '--headless',
        '--nothrottling',
        '--chromeBinary', CHROME_BINARY,
      ],
      {
        cwd: resolve(BENCH_HOME, 'webdriver-ts'),
        stdio: 'inherit',
      },
    )
  } catch (err) {
    if (err?.status == null) throw err
    console.warn(`(bench runner exited ${err.status} — some sub-benchmarks failed plausibility; aggregating what completed)`)
  }
}

// Today as YYYY-MM-DD (local).
export function today() {
  const d = new Date()
  const pad = n => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

// Latest committed baseline file under tests/perf/baselines/. Returns abs path or null.
export function latestBaseline(repoRoot) {
  const dir = resolve(repoRoot, 'tests/perf/baselines')
  if (!existsSync(dir)) return null
  const files = readdirSync(dir).filter(f => /^\d{4}-\d{2}-\d{2}-baseline\.json$/.test(f)).sort()
  if (files.length === 0) return null
  return resolve(dir, files[files.length - 1])
}
