import { chromium } from 'playwright'
const b = await chromium.launch({ headless: true })
const p = await (await b.newContext({ viewport: { width: 1440, height: 900 } })).newPage()
await p.goto('http://127.0.0.1:3270/', { waitUntil: 'load' })
await p.waitForTimeout(800)
await p.screenshot({ path: '/tmp/rail-now.png' })
// dump rail HTML
const railHTML = await p.locator('sui-rail').first().evaluate(el => {
  const sr = el.shadowRoot
  return { outer: el.outerHTML.slice(0, 500), shadow: sr ? sr.innerHTML.slice(0, 4000) : null }
}).catch(e => ({ err: e.message }))
console.log(JSON.stringify(railHTML, null, 2))
await b.close()
