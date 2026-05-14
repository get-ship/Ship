import { chromium } from 'playwright'
const b = await chromium.launch({ headless: true })
const p = await (await b.newContext({ viewport: { width: 1440, height: 900 } })).newPage()
await p.goto('http://127.0.0.1:3270/', { waitUntil: 'load' })
await p.waitForTimeout(500)
// Get LIGHT DOM children of sui-nav-top
const navChildren = await p.locator('sui-nav-top').first().evaluate(el => {
  // Walk shadow root for the actual rendered buttons
  const out = []
  const root = el.shadowRoot
  if (!root) return out
  root.querySelectorAll('sui-button, sui-theme-picker, a.brand, .sep, nav.crumbs, span.cmd-slot, span.view-actions').forEach(c => {
    out.push({
      tag: c.tagName.toLowerCase(),
      part: c.getAttribute('part'),
      title: c.getAttribute('title'),
      cls: c.className.toString().slice(0,40),
    })
  })
  return out
})
console.log('=== nav-top shadow children (rendered order) ===')
navChildren.forEach((c,i) => console.log(`  [${i}] ${c.tag} part="${c.part || ''}" title="${c.title || ''}" cls="${c.cls}"`))
await p.screenshot({ path: '/tmp/nav-after.png' })
await b.close()
