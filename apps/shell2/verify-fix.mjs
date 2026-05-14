// Quick verify: home loads, no defineRoutes error, icons present, leak <= 2
import { chromium } from 'playwright'
const b = await chromium.launch({ headless: true })
const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } })
const p = await ctx.newPage()
let sseOpen = 0
const errors = []
p.on('console', m => { if (m.type() === 'error') errors.push(m.text().slice(0, 200)) })
p.on('pageerror', e => errors.push('[pe] ' + e.message.slice(0, 200)))
p.on('request', r => { if (r.url().includes('/_sui/data')) sseOpen++ })
p.on('requestfinished', r => { if (r.url().includes('/_sui/data')) sseOpen-- })

await p.goto('http://127.0.0.1:3270/', { waitUntil: 'load', timeout: 8000 })
await p.waitForTimeout(800)

const routes = ['/home', '/agents', '/git', '/skills', '/workspaces', '/events']
for (const r of routes) {
  await p.click(`sui-rail-item[href="${r}"]`).catch(() => {})
  await p.waitForTimeout(700)
}
await p.waitForTimeout(2000)

const railIcons = await p.locator('sui-rail-item').evaluateAll(items => items.map(i => {
  const icon = i.shadowRoot?.querySelector('sui-icon')
  const svg = icon?.querySelector('svg')
  const paths = svg?.querySelectorAll('path,rect,line,circle').length ?? 0
  return { icon: i.getAttribute('icon'), label: i.getAttribute('label'), pathCount: paths }
}))
console.log('=== Rail icons (pathCount > 0 = renders, 0 = empty) ===')
railIcons.forEach(r => console.log(`  ${r.icon} (${r.label}): paths=${r.pathCount}`))

console.log(`\n=== SSE state ===`)
console.log(`SSE open at end: ${sseOpen}`)

console.log(`\n=== Console errors (${errors.length}) ===`)
errors.slice(0, 8).forEach(e => console.log(`  ${e}`))

await p.screenshot({ path: '/tmp/verify-now.png' })
await b.close()
