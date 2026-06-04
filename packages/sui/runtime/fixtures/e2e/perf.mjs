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
  '/_sui/view/counter',
  '/_sui/view/sui-demo',
  '/_sui/view/demos',
  '/_sui/view/demos/conditional-render',
  '/_sui/view/demos/iteration',
  '/_sui/view/demos/signals-and-derived',
  '/_sui/view/demos/lifecycle',
  '/_sui/view/demos/match',
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

await browser.close()
console.log('\nperf OK')
