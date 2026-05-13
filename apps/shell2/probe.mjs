// Shell2 behavioral probe. Captures screenshots + console logs + nav state
// per interaction so we can spec the actual bugs.
import { chromium } from 'playwright'
import { mkdirSync, writeFileSync } from 'node:fs'

const OUT = '/tmp/shell2-probe-out'
mkdirSync(OUT, { recursive: true })

const browser = await chromium.launch({ headless: true })
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } })
const page = await ctx.newPage()

const logs = []
page.on('console', m => logs.push(`[${m.type()}] ${m.text()}`))
page.on('pageerror', e => logs.push(`[pageerror] ${e.message}`))
page.on('requestfailed', r => logs.push(`[reqfail] ${r.url()} - ${r.failure()?.errorText}`))

const trace = []
async function snap(label) {
  await page.waitForTimeout(400)
  await page.screenshot({ path: `${OUT}/${label}.png`, fullPage: false })
  const url = page.url()
  // Probe rail + nav state from the page
  const railVisible = await page.locator('sui-rail').first().isVisible().catch(() => null)
  const navTopVisible = await page.locator('sui-nav-top').first().isVisible().catch(() => null)
  const mainContent = await page.locator('#ship-content, [data-ship-content], main').first().innerText().catch(() => '').then(t => t?.slice(0, 200))
  trace.push({ label, url, railVisible, navTopVisible, mainContentHead: mainContent })
  console.log(`  → ${label}: url=${url} rail=${railVisible} nav=${navTopVisible} content="${mainContent?.slice(0, 60)}…"`)
}

console.log('=== 1. Open root ===')
await page.goto('http://127.0.0.1:3270/', { waitUntil: 'networkidle', timeout: 10000 }).catch(e => console.log('goto err:', e.message))
await snap('01-root')

console.log('=== 2. List rail items ===')
const railItems = await page.$$eval('sui-rail a, sui-rail button', els =>
  els.map(e => ({
    tag: e.tagName.toLowerCase(),
    text: e.textContent?.trim().slice(0, 30),
    href: e.getAttribute('href'),
    title: e.getAttribute('title') ?? e.getAttribute('aria-label'),
    cls: e.className.toString().slice(0, 60),
  }))
).catch(() => [])
console.log('rail items:', railItems.length)
railItems.forEach((r, i) => console.log(`  [${i}] ${r.tag} href=${r.href} title=${r.title} text="${r.text}"`))

console.log('=== 3. List nav-top buttons ===')
const navItems = await page.$$eval('sui-nav-top button, sui-nav-top a', els =>
  els.map(e => ({
    tag: e.tagName.toLowerCase(),
    text: e.textContent?.trim().slice(0, 30),
    title: e.getAttribute('title') ?? e.getAttribute('aria-label'),
    cls: e.className.toString().slice(0, 60),
  }))
).catch(() => [])
console.log('nav-top items:', navItems.length)
navItems.forEach((r, i) => console.log(`  [${i}] ${r.tag} title=${r.title} text="${r.text}"`))

console.log('=== 4. Click each rail nav item (top 5) ===')
for (let i = 0; i < Math.min(railItems.length, 8); i++) {
  const r = railItems[i]
  if (!r.href || r.href === '#') continue
  try {
    const sel = `sui-rail a[href="${r.href}"]`
    await page.click(sel, { timeout: 2000 })
    await snap(`02-rail-click-${i}-${r.href.replace(/[/]/g, '_')}`)
  } catch (e) {
    console.log(`  click ${r.href} FAILED: ${e.message}`)
  }
}

console.log('=== 5. Direct goto each /sui/<view> ===')
const views = ['home', 'agents', 'overview', 'sessions', 'workspaces', 'skills', 'chat', 'mesh', 'events', 'settings', 'git', 'canvas', 'specs', 'collections', 'pipelines', 'pipelines-board', 'dashboard', 'theme-studio', 'setup']
for (const v of views) {
  try {
    const url = `http://127.0.0.1:3270/sui/${v}`
    const resp = await page.goto(url, { waitUntil: 'networkidle', timeout: 4000 })
    const status = resp?.status() ?? 0
    await page.waitForTimeout(200)
    const text = await page.locator('body').innerText().catch(() => '')
    const len = text.length
    const head = text.slice(0, 80).replace(/\s+/g, ' ')
    await page.screenshot({ path: `${OUT}/view-${v}.png` })
    console.log(`  /sui/${v}: ${status} len=${len} head="${head}"`)
    trace.push({ view: v, status, contentLen: len, contentHead: head })
  } catch (e) {
    console.log(`  /sui/${v}: ERROR ${e.message}`)
  }
}

console.log('=== 6. Toggle rail (find toggle button) ===')
const toggleBtn = await page.locator('sui-rail [class*="toggle"], sui-rail button:has-text("⏵"), sui-rail button:has-text("⏴"), sui-nav-top button:has-text("Menu"), sui-nav-top [aria-label*="menu" i], sui-nav-top [aria-label*="rail" i]').first()
try {
  if (await toggleBtn.isVisible()) {
    await toggleBtn.click()
    await snap('06-after-rail-toggle')
  } else {
    console.log('  toggle button not visible')
  }
} catch (e) {
  console.log('  toggle click err:', e.message)
}

writeFileSync(`${OUT}/trace.json`, JSON.stringify({ railItems, navItems, trace }, null, 2))
writeFileSync(`${OUT}/console.log`, logs.join('\n'))
console.log(`\nWrote ${trace.length} trace entries, ${logs.length} console events`)
console.log(`Screenshots in: ${OUT}`)
await browser.close()
