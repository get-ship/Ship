#!/usr/bin/env node
// CPU-profile capture harness for js-framework-benchmark ops.
//
// Drives a framework's bench page (sui, solid, ...) via Chrome DevTools
// Protocol through Playwright, runs a chosen op N times under the
// V8 sampling profiler, and writes a `.cpuprofile` file plus a
// self-time summary to stdout.
//
// Out of scope for this script: median timing — the bench framework
// (`just bench`) is the source of truth for that. This script's job is
// "where is the time spent", not "how much time".
//
// USAGE
//   node scripts/dev/perf-profile.mjs                                       # sui / 01_run1k / 10 iter
//   node scripts/dev/perf-profile.mjs --framework solid --op 02_replace1k
//   node scripts/dev/perf-profile.mjs --all --iter 10                       # sui+solid × all ops
//   node scripts/dev/perf-profile.mjs --headed                              # show the browser
//
// FLAGS
//   --framework <sui|solid>    Default: sui
//   --op <01_run1k|02_replace1k|05_swap1k>   Default: 01_run1k
//   --iter <N>                 Default: 10
//   --all                      Iterate {sui,solid} × all ops
//   --headed                   Run with visible browser (default: headless)
//   --out <dir>                Profile output directory
//
// Requires:
//   - js-framework-benchmark checkout at ~/dev/js-framework-benchmark
//     (override with BENCH_HOME). Server auto-started on :8080 if down.
//   - Playwright Chromium installed (devDep `playwright`, already in repo).

import { chromium } from 'playwright'
import { spawn } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { homedir } from 'node:os'

const HERE = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(HERE, '..', '..')
const BENCH_HOME = process.env.BENCH_HOME || resolve(homedir(), 'dev/js-framework-benchmark')
const DEFAULT_OUT = resolve(REPO_ROOT, 'tests/perf/profiles')

// Map an op to (setup clicks before each measured iteration, measured click(s)).
// `wait` is a predicate run inside the page after the measured click(s).
const OPS = {
  '01_run1k': {
    setup: [],
    measured: ['#run'],
    wait: () => document.querySelectorAll('.test-data tr').length === 1000,
    // Between iterations, clear the table so the next click is a true
    // empty→1000 transition (not a 1000→1000 replace).
    between: 'clear',
  },
  '02_replace1k': {
    // Replace-1k IS "click run while 1000 rows are already mounted".
    // Pre-populate with one click that's not measured, then each measured
    // iteration replaces. The first measured click is technically the
    // 1k → 1k replace too.
    setup: [],
    measured: ['#run'],
    wait: () => document.querySelectorAll('.test-data tr').length === 1000,
    between: 'noop',
  },
  '05_swap1k': {
    setup: [],
    measured: ['#swaprows'],
    // Swap moves rows 2 and 999. Hard to assert programmatically without
    // before-state — settle on a microtask instead.
    wait: () => true,
    between: 'noop',
  },
}

const FRAMEWORK_PATH = {
  sui: 'frameworks/keyed/sui/index.html',
  solid: 'frameworks/keyed/solid/index.html',
}

function parseArgs(argv) {
  const out = {
    framework: 'sui',
    op: '01_run1k',
    iter: 10,
    all: false,
    headed: false,
    outDir: DEFAULT_OUT,
  }
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--framework') out.framework = argv[++i]
    else if (a === '--op') out.op = argv[++i]
    else if (a === '--iter' || a === '--iterations') out.iter = Number(argv[++i])
    else if (a === '--all') out.all = true
    else if (a === '--headed') out.headed = true
    else if (a === '--out') out.outDir = resolve(argv[++i])
    else if (a === '--help' || a === '-h') { printUsage(); process.exit(0) }
    else { console.error(`unknown arg: ${a}`); printUsage(); process.exit(2) }
  }
  return out
}

function printUsage() {
  console.log(`perf-profile.mjs — capture CPU profiles for SUI vs comparison frameworks

  --framework <sui|solid>      Default: sui
  --op <01_run1k|02_replace1k|05_swap1k>   Default: 01_run1k
  --iter <N>                   Iterations under the profiler. Default: 10
  --all                        Sweep {sui,solid} × all configured ops
  --headed                     Visible browser (headless by default)
  --out <dir>                  Override output directory`)
}

async function isUp(url) {
  try {
    const r = await fetch(url, { method: 'HEAD' })
    return r.ok
  } catch { return false }
}

async function ensureServer() {
  const url = 'http://localhost:8080/index.html'
  if (await isUp(url)) return null
  console.error(`bench server not up — starting at ${BENCH_HOME}`)
  const child = spawn('npm', ['start'], { cwd: BENCH_HOME, stdio: 'ignore', detached: true })
  child.unref()
  for (let i = 0; i < 60; i++) {
    await new Promise(r => setTimeout(r, 500))
    if (await isUp(url)) return child
  }
  throw new Error(`bench server failed to come up on :8080 (BENCH_HOME=${BENCH_HOME})`)
}

async function settle(page) {
  // Two RAFs: layout flushed, paint scheduled.
  await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))))
}

async function capture({ framework, op, iter, outDir, headed }) {
  const path = FRAMEWORK_PATH[framework]
  if (!path) throw new Error(`unknown framework: ${framework}`)
  const spec = OPS[op]
  if (!spec) throw new Error(`unsupported op: ${op}`)

  const browser = await chromium.launch({ headless: !headed })
  try {
    const ctx = await browser.newContext()
    const page = await ctx.newPage()
    const cdp = await ctx.newCDPSession(page)
    await page.goto(`http://localhost:8080/${path}`, { waitUntil: 'load' })
    await page.waitForSelector('#run', { timeout: 15_000 })

    // Warmup: one un-profiled run to JIT the hot paths.
    await page.click('#run')
    await page.waitForFunction(spec.wait, null, { timeout: 15_000 })
    if (await page.$('#clear')) {
      await page.click('#clear')
      await page.waitForFunction(() => document.querySelectorAll('.test-data tr').length === 0)
    }
    await settle(page)

    // 02_replace1k needs a pre-populated table; pre-seed once before the
    // profiled window opens so the first measured click is a true replace.
    if (op === '02_replace1k') {
      await page.click('#run')
      await page.waitForFunction(spec.wait)
      await settle(page)
    }

    await cdp.send('Profiler.enable')
    // 100us interval = 10kHz sampling. Default is 1kHz; we want detail.
    await cdp.send('Profiler.setSamplingInterval', { interval: 100 })
    await cdp.send('Profiler.start')

    for (let i = 0; i < iter; i++) {
      for (const sel of spec.setup) {
        await page.click(sel)
        await page.waitForFunction(spec.wait)
      }
      for (const sel of spec.measured) {
        await page.click(sel)
      }
      await page.waitForFunction(spec.wait)
      await settle(page)

      if (spec.between === 'clear' && await page.$('#clear')) {
        await page.click('#clear')
        await page.waitForFunction(() => document.querySelectorAll('.test-data tr').length === 0)
        await settle(page)
      }
    }

    const { profile } = await cdp.send('Profiler.stop')
    await cdp.send('Profiler.disable')

    mkdirSync(outDir, { recursive: true })
    const date = new Date().toISOString().slice(0, 10)
    const file = resolve(outDir, `${date}-${framework}-${op}.cpuprofile`)
    writeFileSync(file, JSON.stringify(profile))

    const summary = summarize(profile)
    const totalMs = summary.totalMs
    console.log(`\n${framework}/${op} — ${iter} iter, ${totalMs.toFixed(1)}ms sampled — ${file}`)
    console.log('  rank  self ms   %    function')
    for (let i = 0; i < Math.min(10, summary.rows.length); i++) {
      const r = summary.rows[i]
      const pct = (r.selfMs / totalMs * 100).toFixed(1)
      console.log(`  ${String(i + 1).padStart(4)}  ${r.selfMs.toFixed(1).padStart(7)}  ${pct.padStart(4)}  ${r.fn}`)
    }
    return { file, summary, totalMs }
  } finally {
    await browser.close()
  }
}

function summarize(profile) {
  const { nodes, samples, timeDeltas } = profile
  const byId = new Map()
  for (const n of nodes) byId.set(n.id, n)
  const selfUs = new Map()
  let totalUs = 0
  for (let i = 0; i < samples.length; i++) {
    const id = samples[i]
    const dt = timeDeltas[i]
    selfUs.set(id, (selfUs.get(id) ?? 0) + dt)
    totalUs += dt
  }
  const rows = []
  for (const [id, us] of selfUs) {
    const n = byId.get(id)
    if (!n) continue
    const cf = n.callFrame
    const name = cf.functionName || '(anonymous)'
    // Skip (program) and (idle) — those are V8 housekeeping, not our code.
    if (name === '(program)' || name === '(idle)' || name === '(garbage collector)') {
      // Track GC separately though.
      if (name === '(garbage collector)') rows.push({ fn: 'GC', selfMs: us / 1000 })
      continue
    }
    const url = cf.url || ''
    const short = url ? shortUrl(url) : '(native)'
    rows.push({ fn: `${name} @ ${short}:${cf.lineNumber + 1}`, selfMs: us / 1000 })
  }
  rows.sort((a, b) => b.selfMs - a.selfMs)
  return { rows, totalMs: totalUs / 1000 }
}

function shortUrl(url) {
  // Strip query/hash; keep last two path segments.
  const clean = url.split('?')[0].split('#')[0]
  const parts = clean.split('/')
  return parts.slice(-2).join('/') || clean
}

async function main() {
  const args = parseArgs(process.argv.slice(2))
  const server = await ensureServer()
  try {
    if (args.all) {
      for (const framework of ['sui', 'solid']) {
        for (const op of Object.keys(OPS)) {
          await capture({ ...args, framework, op })
        }
      }
    } else {
      await capture(args)
    }
  } finally {
    if (server) {
      try { process.kill(-server.pid) } catch {}
    }
  }
}

main().catch((e) => { console.error(e); process.exit(1) })
