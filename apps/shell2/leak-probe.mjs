// Measure navigation perf + leak across 10 rail clicks
import { chromium } from 'playwright'
const b = await chromium.launch({ headless: true })
const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } })
const p = await ctx.newPage()

let reqOpen = 0
const sseOpen = new Set()
p.on('request', r => { if (r.url().includes('/_sui/data')) { reqOpen++; sseOpen.add(r.url()) } })
p.on('requestfinished', r => { if (r.url().includes('/_sui/data')) reqOpen-- })

await p.goto('http://127.0.0.1:3270/', { waitUntil: 'load' })
await p.waitForTimeout(500)

const routes = ['/home', '/agents', '/overview', '/sessions', '/workspaces', '/skills', '/events', '/git', '/mesh', '/settings']
console.log('round\troute\ttime_ms\tsse_open\tdom_nodes\theap_mb')
for (let round = 0; round < 2; round++) {
  for (const r of routes) {
    const t0 = Date.now()
    await p.click(`sui-rail-item[href="${r}"]`).catch(() => {})
    await p.waitForFunction(() => location.pathname !== '/', { timeout: 3000 }).catch(() => {})
    await p.waitForTimeout(400)
    const dur = Date.now() - t0
    const nodes = await p.evaluate(() => document.querySelectorAll('*').length)
    const heap = await p.evaluate(() => performance.memory?.usedJSHeapSize / 1024 / 1024 || 0)
    console.log(`${round}\t${r}\t${dur}\t${reqOpen}\t${nodes}\t${heap.toFixed(1)}`)
  }
}
console.log(`\nUnique SSE urls opened across run: ${sseOpen.size}`)
await b.close()
