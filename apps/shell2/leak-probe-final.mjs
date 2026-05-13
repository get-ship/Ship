// Same as leak-probe but with idle settling at end + EventSource-specific tracking.
import { chromium } from 'playwright'
const b = await chromium.launch({ headless: true })
const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } })
const p = await ctx.newPage()

let sseOpen = 0
p.on('request', r => { if (r.url().includes('/_sui/data')) sseOpen++ })
p.on('requestfinished', r => { if (r.url().includes('/_sui/data')) sseOpen-- })
p.on('requestfailed', r => { if (r.url().includes('/_sui/data')) sseOpen-- })

await p.goto('http://127.0.0.1:3270/', { waitUntil: 'load' })
await p.waitForTimeout(500)

const routes = ['/home', '/agents', '/workspaces', '/skills', '/events', '/git', '/settings']
for (let round = 0; round < 2; round++) {
  for (const r of routes) {
    await p.click(`sui-rail-item[href="${r}"]`).catch(() => {})
    await p.waitForTimeout(300)
  }
}
console.log('After 14 navs, sse_open =', sseOpen)
// Now navigate to a non-live view (home likely OK, or empty path)
await p.click(`sui-rail-item[href="/"]`).catch(() => {})
await p.waitForTimeout(5000) // give time to settle
console.log('After 5s idle on landing, sse_open =', sseOpen)
// Also count active EventSource instances
const esCount = await p.evaluate(() => {
  // Track via patching — alas, no global registry. Check window for any leaks
  return performance.getEntriesByType?.('resource').filter(e => e.name.includes('/_sui/data')).length ?? 0
})
console.log('Performance resource entries for /_sui/data:', esCount)
await b.close()
