#!/usr/bin/env node
// Browser-side perf for SUI rendering.
//
// Two measurements:
//
//   1. Cold-load time-to-interactive per fixture view. Median of 3.
//   2. 1000-cycle mount/unmount on the counter view, with a heap-size
//      delta sampled before/after. Catches retain-on-mount leaks.
//
// Numbers are baselines. Report and compare across changes.

import { chromium } from 'playwright'

const BASE = process.env.SUI_E2E_BASE_URL || 'http://127.0.0.1:9385'

const VIEWS = [
  '/view-host/counter',
  '/view-host/sui-demo',
  '/view-host/demos',
  '/view-host/demos/conditional-render',
  '/view-host/demos/iteration',
  '/view-host/demos/signals-and-derived',
  '/view-host/demos/lifecycle',
  '/view-host/demos/match',
]

function median(xs) {
  const s = [...xs].sort((a, b) => a - b)
  const mid = Math.floor(s.length / 2)
  return s.length % 2 === 1 ? s[mid] : (s[mid - 1] + s[mid]) / 2
}

const browser = await chromium.launch()

console.log('=== Cold-load TTI (median of 3) ===')
const ttiRows = []
for (const path of VIEWS) {
  const samples = []
  for (let i = 0; i < 3; i++) {
    const ctx = await browser.newContext()
    const page = await ctx.newPage()
    const t0 = Date.now()
    await page.goto(BASE + path, { waitUntil: 'networkidle' })
    samples.push(Date.now() - t0)
    await ctx.close()
  }
  const m = median(samples)
  ttiRows.push({ path, median: m, samples })
  console.log(`  ${m.toString().padStart(5)} ms  ${path}   (samples: ${samples.join(', ')})`)
}

console.log('\n=== 1000-cycle mount/unmount soak on counter ===')
const soakCtx = await browser.newContext()
const soakPage = await soakCtx.newPage()
await soakPage.goto(BASE + '/view-host/counter', { waitUntil: 'networkidle' })

const soakResult = await soakPage.evaluate(async ({ base, cycles }) => {
  const payload = await fetch(base + '/_sui/views/counter').then((r) => r.json())
  // mount() lives in @ship/sui-runtime, not in the view module.
  // The view-host page set up the importmap; resolve from there.
  const { mount } = await import('@ship/sui-runtime')
  const mod = await import(payload.moduleUrl)
  // Pre-resolve the module so mount() doesn't refetch it every cycle.
  const mountOpts = { cssScope: 'none', module: mod }
  if (globalThis.gc) globalThis.gc()
  const before = performance.memory?.usedJSHeapSize ?? 0
  const t0 = performance.now()
  for (let i = 0; i < cycles; i++) {
    const host = document.createElement('div')
    document.body.appendChild(host)
    const handle = await mount(host, payload, mountOpts)
    handle.dispose()
    host.remove()
  }
  const dt = performance.now() - t0
  if (globalThis.gc) globalThis.gc()
  const after = performance.memory?.usedJSHeapSize ?? 0
  return { dt, before, after, delta: after - before }
}, { base: BASE, cycles: 1000 })

const ms = soakResult.dt.toFixed(0)
const perOp = (soakResult.dt / 1000).toFixed(2)
const deltaKb = (soakResult.delta / 1024).toFixed(1)
console.log(`  total: ${ms} ms  (${perOp} ms/cycle)`)
console.log(`  heap delta: ${deltaKb} KB (chromium-only metric)`)

await soakCtx.close()
await browser.close()

const LEAK_THRESHOLD_KB = 25 * 1024
if (soakResult.delta > LEAK_THRESHOLD_KB * 1024) {
  console.error(`\nleak suspect: heap delta ${deltaKb} KB > ${LEAK_THRESHOLD_KB} KB`)
  process.exit(1)
}
console.log('\nperf OK')
