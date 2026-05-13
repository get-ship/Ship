// Shell2 behavioral probe. Captures screenshots + console logs + nav state
// per interaction so we can spec the actual bugs.
//
// Direct-goto checks (section 5) wait for `load` rather than `networkidle`:
// views that own a live SSE source (skills/mesh/events/settings/git)
// hold an EventSource open for the whole mount lifecycle, so
// `networkidle` never settles. `load` fires after the document is
// parsed + the initial sub-resources finish — by that point the
// runtime has imported the view module and called `mount()`. We
// then wait for the SUI root to gain children (DOM count, not visible
// text — loading skeletons have no text but are valid rendered
// content) before we measure.
import { chromium } from 'playwright'
import { mkdirSync, writeFileSync, readFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'

const OUT = '/tmp/shell2-probe-out'
mkdirSync(OUT, { recursive: true })

function resolveBaseUrl() {
  if (process.env.SHELL2_URL) return process.env.SHELL2_URL.replace(/\/$/, '')
  // Default: derive vite port from this worktree's daemon. Daemon dir
  // lives at $HOME/.ship/machine/daemons/<hash>/, daemon-port + offset
  // maps to vite-port via `port - 9315 + 3200`.
  const daemonsDir = join(process.env.HOME ?? '', '.ship', 'machine', 'daemons')
  const repoRoot = process.cwd().replace(/\/apps\/shell2$/, '')
  // Best-effort: cargo run target/debug/ship to read daemon dir; fall
  // back to 3270 (legacy default).
  try {
    const { execFileSync } = require('node:child_process')
    const bin = `${repoRoot}/target/debug/ship`
    if (existsSync(bin)) {
      const dir = execFileSync(bin, ['daemon', 'dir'], { encoding: 'utf8' }).trim()
      const port = readFileSync(join(dir, 'port'), 'utf8').trim()
      const vite = 3200 + (Number(port) - 9315)
      return `http://127.0.0.1:${vite}`
    }
  } catch { /* ignore */ }
  void daemonsDir
  return 'http://127.0.0.1:3270'
}
const BASE = resolveBaseUrl()

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

console.log(`=== Probe target: ${BASE} ===`)
console.log('=== 1. Open root ===')
await page.goto(`${BASE}/`, { waitUntil: 'load', timeout: 10000 }).catch(e => console.log('goto err:', e.message))
await page.waitForTimeout(500)
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
    const url = `${BASE}/sui/${v}`
    // `waitUntil: 'load'` (not 'networkidle') — direct /sui/<view>
    // mounts open a long-lived SSE EventSource that keeps the network
    // busy for the lifetime of the view, so 'networkidle' would never
    // resolve. 'load' fires after the document + initial sub-resources
    // parse, by which point mount() has run.
    const resp = await page.goto(url, { waitUntil: 'load', timeout: 5000 })
    const status = resp?.status() ?? 0
    // Wait for the SUI root to gain a child (loading skeleton, error
    // banner, or real content — anything but the empty placeholder).
    // 1500ms cap matches the budget acceptance criteria allow.
    await page.waitForFunction(
      () => {
        const root = document.querySelector('#sui-root, #ship-content, [data-ship-content], main')
        return !!root && root.children.length > 0
      },
      { timeout: 1500 },
    ).catch(() => {})
    const text = await page.locator('body').innerText().catch(() => '')
    const len = text.length
    const head = text.slice(0, 80).replace(/\s+/g, ' ')
    // Element count — a more honest "rendered" check than innerText,
    // since loading skeletons / icon-only chrome have no visible text.
    const elCount = await page.locator('#sui-root *, #ship-content *').count().catch(() => 0)
    await page.screenshot({ path: `${OUT}/view-${v}.png` })
    console.log(`  /sui/${v}: ${status} len=${len} els=${elCount} head="${head}"`)
    trace.push({ view: v, status, contentLen: len, contentEls: elCount, contentHead: head })
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
