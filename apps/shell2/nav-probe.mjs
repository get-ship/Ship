import { chromium } from 'playwright'
const b = await chromium.launch({ headless: true })
const p = await (await b.newContext({ viewport: { width: 1440, height: 900 } })).newPage()
await p.goto('http://127.0.0.1:3270/', { waitUntil: 'load' })
await p.waitForTimeout(500)
await p.screenshot({ path: '/tmp/nav-now.png' })
const nav = await p.locator('sui-nav-top').first().evaluate(el => {
  return { outer: el.outerHTML, shadow: el.shadowRoot?.innerHTML }
}).catch(e => ({ err: e.message }))
console.log('=== NAV OUTER ===')
console.log(nav.outer?.slice(0, 1500))
console.log('=== NAV SHADOW (head) ===')
console.log(nav.shadow?.slice(0, 2500))
console.log('=== NAV BUTTONS / ANCHORS ===')
const items = await p.$$eval('sui-nav-top button, sui-nav-top a', els => els.map(e => ({
  tag: e.tagName.toLowerCase(),
  cls: e.className.toString().slice(0,60),
  title: e.getAttribute('title') ?? e.getAttribute('aria-label'),
  text: e.textContent?.trim().slice(0,30),
  visible: e.offsetWidth > 0 && e.offsetHeight > 0,
})))
items.forEach((it,i) => console.log(`  [${i}] vis=${it.visible} ${it.tag} cls=${it.cls} title=${it.title} text="${it.text}"`))
await b.close()
