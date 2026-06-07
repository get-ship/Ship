// deck-monitor — real, unfakeable measurement of a running Ship app.
//
// Drives a headless Chromium through N navigations across the app's routes and
// samples, over time: daemon RSS (/proc/<pid>/VmRSS), open sockets to the
// daemon (the HTTP/1.1 connection-ceiling signal), page JS heap, per-route load
// time, and every console error + non-2xx response. Emits a JSON + a verdict.
//
// This is the behavioral bar from .ship/rules/engineering.md: a UI/perf/memory
// fix is "done" only when these numbers hold — never a 200 or a screenshot.
//
// Usage:
//   bun run scripts/perf/deck-monitor.ts --url http://127.0.0.1:<port> \
//     [--pid <shipd-pid>] [--routes /,/activity,/studio] [--iterations 20] \
//     [--out .artifacts/perf]
//
// Exits non-zero if any threshold is breached (CI-gate friendly).

import { chromium, type ConsoleMessage, type Response } from 'playwright'
import { execSync } from 'node:child_process'
import { mkdirSync, writeFileSync, readFileSync } from 'node:fs'

interface Args { url: string; pid?: number; routes: string[]; iterations: number; out: string; settleMs: number }

function parseArgs(): Args {
  const a = process.argv.slice(2)
  const get = (k: string) => { const i = a.indexOf(k); return i >= 0 ? a[i + 1] : undefined }
  const url = get('--url')
  if (!url) { console.error('deck-monitor: --url <base> is required'); process.exit(2) }
  const routesRaw = get('--routes') ?? '/'
  const routes = routesRaw.startsWith('@')
    ? readFileSync(routesRaw.slice(1), 'utf8').split(/\r?\n/).map(s => s.trim()).filter(Boolean)
    : routesRaw.split(',').map(s => s.trim()).filter(Boolean)
  return {
    url: url.replace(/\/$/, ''),
    pid: get('--pid') ? Number(get('--pid')) : undefined,
    routes,
    iterations: Number(get('--iterations') ?? '20'),
    out: get('--out') ?? '.artifacts/perf',
    settleMs: Number(get('--settle') ?? '0'),
  }
}

/** Daemon RSS in KB from /proc, or null if pid unknown/gone. */
function rssKb(pid?: number): number | null {
  if (!pid) return null
  try {
    const status = readFileSync(`/proc/${pid}/status`, 'utf8')
    const m = status.match(/VmRSS:\s+(\d+)\s+kB/)
    return m ? Number(m[1]) : null
  } catch { return null }
}

/** Count ESTABLISHED TCP sockets whose peer is the daemon port (the HTTP/1.1
 *  ceiling signal). Falls back to 0 if `ss` is unavailable. */
function daemonSockets(port: number): number {
  try {
    const out = execSync(`ss -tn state established 2>/dev/null || true`, { encoding: 'utf8' })
    return out.split('\n').filter(l => l.includes(`:${port} `) || l.includes(`:${port}\t`)).length
  } catch { return 0 }
}

function portOf(url: string): number { return Number(new URL(url).port || '80') }

/** Resolve the daemon pid from the listening port if not supplied. */
function resolvePid(port: number, given?: number): number | undefined {
  if (given) return given
  try {
    const out = execSync(`ss -tnlp 2>/dev/null | grep ':${port} ' || true`, { encoding: 'utf8' })
    const m = out.match(/pid=(\d+)/)
    return m ? Number(m[1]) : undefined
  } catch { return undefined }
}

interface Sample { iter: number; route: string; loadMs: number; heapBytes: number | null; rssKb: number | null; sockets: number }

async function main() {
  const args = parseArgs()
  const port = portOf(args.url)
  const pid = resolvePid(port, args.pid)
  mkdirSync(args.out, { recursive: true })

  const consoleErrors: { route: string; text: string }[] = []
  const netFailures: { route: string; url: string; status: number }[] = []
  const samples: Sample[] = []

  const browser = await chromium.launch({ headless: true })
  const ctx = await browser.newContext()
  const page = await ctx.newPage()
  let currentRoute = '(startup)'
  page.on('console', (m: ConsoleMessage) => { if (m.type() === 'error') consoleErrors.push({ route: currentRoute, text: m.text().slice(0, 300) }) })
  page.on('pageerror', (e) => consoleErrors.push({ route: currentRoute, text: `pageerror: ${String(e).slice(0, 300)}` }))
  page.on('response', (r: Response) => { const s = r.status(); if (s >= 400) netFailures.push({ route: currentRoute, url: r.url(), status: s }) })

  const baseRss = rssKb(pid)
  const baseSockets = daemonSockets(port)

  for (let iter = 0; iter < args.iterations; iter++) {
    for (const route of args.routes) {
      currentRoute = route
      const t0 = performance.now()
      let loadMs = -1
      try {
        await page.goto(args.url + route, { waitUntil: 'load', timeout: 20000 })
        loadMs = performance.now() - t0
      } catch (e) {
        consoleErrors.push({ route, text: `navigation failed: ${String(e).slice(0, 200)}` })
      }
      // Settle window: async work (dock icons, live-data subscriptions, cmdk)
      // happens AFTER `load` fires. Sample sockets repeatedly to catch the peak
      // (the connection-ceiling pile-up) and let console errors accumulate.
      let sockets = daemonSockets(port)
      if (args.settleMs > 0) {
        const end = performance.now() + args.settleMs
        while (performance.now() < end) {
          sockets = Math.max(sockets, daemonSockets(port))
          await page.waitForTimeout(500)
        }
      }
      const heapBytes = await page.evaluate(() => (performance as any).memory?.usedJSHeapSize ?? null).catch(() => null)
      samples.push({ iter, route, loadMs, heapBytes, rssKb: rssKb(pid), sockets })
    }
  }

  await browser.close()

  const last = samples.slice(-args.routes.length)
  const heapStart = samples[0]?.heapBytes ?? null
  const heapEnd = last.reduce((mx, s) => Math.max(mx, s.heapBytes ?? 0), 0)
  const rssEnd = last.reduce((mx, s) => Math.max(mx, s.rssKb ?? 0), 0)
  const peakSockets = samples.reduce((mx, s) => Math.max(mx, s.sockets), baseSockets)
  const slowest = samples.reduce((w, s) => (s.loadMs > (w?.loadMs ?? -1) ? s : w), samples[0])

  // Thresholds (the bar). Tune as baselines land.
  const TH = { rssGrowthKb: 200_000, heapGrowthBytes: 50_000_000, maxSockets: 8, maxConsoleErrors: 0, maxLoadMs: 3000 }
  const rssGrowth = baseRss != null ? rssEnd - baseRss : 0
  const heapGrowth = heapStart != null ? heapEnd - heapStart : 0

  const checks = [
    { name: 'daemon RSS growth', value: rssGrowth, limit: TH.rssGrowthKb, unit: 'kB', pass: baseRss == null || rssGrowth <= TH.rssGrowthKb },
    { name: 'page heap growth', value: heapGrowth, limit: TH.heapGrowthBytes, unit: 'bytes', pass: heapStart == null || heapGrowth <= TH.heapGrowthBytes },
    { name: 'peak daemon sockets', value: peakSockets, limit: TH.maxSockets, unit: 'conns', pass: peakSockets <= TH.maxSockets },
    { name: 'console errors', value: consoleErrors.length, limit: TH.maxConsoleErrors, unit: 'errors', pass: consoleErrors.length <= TH.maxConsoleErrors },
    { name: 'slowest route load', value: Math.round(slowest?.loadMs ?? 0), limit: TH.maxLoadMs, unit: 'ms', pass: (slowest?.loadMs ?? 0) <= TH.maxLoadMs },
    { name: 'network failures', value: netFailures.length, limit: 0, unit: 'non-2xx', pass: netFailures.length === 0 },
  ]
  const verdict = checks.every(c => c.pass) ? 'PASS' : 'FAIL'

  const report = {
    verdict, url: args.url, pid: pid ?? null, iterations: args.iterations, routes: args.routes,
    baseRssKb: baseRss, rssEndKb: rssEnd, rssGrowthKb: rssGrowth,
    heapStartBytes: heapStart, heapEndBytes: heapEnd, heapGrowthBytes: heapGrowth,
    baseSockets, peakSockets, slowestRoute: slowest?.route, slowestMs: Math.round(slowest?.loadMs ?? 0),
    consoleErrors: consoleErrors.slice(0, 50), netFailures: netFailures.slice(0, 50), checks,
  }
  writeFileSync(`${args.out}/deck-monitor.json`, JSON.stringify(report, null, 2))

  const fmt = (n: number) => n.toLocaleString()
  console.log(`\ndeck-monitor: ${verdict}  (${args.url}, ${args.iterations}× ${args.routes.length} routes, pid=${pid ?? '?'})`)
  for (const c of checks) console.log(`  ${c.pass ? '✅' : '❌'} ${c.name}: ${fmt(c.value)} ${c.unit} (limit ${fmt(c.limit)})`)
  if (consoleErrors.length) console.log(`  first console error: [${consoleErrors[0].route}] ${consoleErrors[0].text}`)
  console.log(`  report → ${args.out}/deck-monitor.json`)
  process.exit(verdict === 'PASS' ? 0 : 1)
}

main().catch((e) => { console.error('deck-monitor crashed:', e); process.exit(2) })
