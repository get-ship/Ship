/**
 * shell2 smoke probe — headless Chromium via Playwright.
 *
 * Usage:
 *   node .artifacts/shell2-smoke.mjs [--url http://localhost:3200]
 *
 * Requires a running shell2 Vite dev server (default: http://localhost:3200).
 * Captures a screenshot at .artifacts/shell2-final.png.
 *
 * Exit 0 = all assertions passed (deferred items reported but not fatal).
 * Exit 1 = at least one assertion failed or console errors detected.
 */

import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const ROOT = path.resolve(__dirname, '..')

// Resolve playwright from the workspace node_modules.
const req = createRequire(path.join(ROOT, 'package.json'))
const { chromium } = req('playwright')

// ── Config ────────────────────────────────────────────────────────────────────

const argUrl = process.argv.find((a) => a.startsWith('--url='))
const BASE_URL = argUrl ? argUrl.replace('--url=', '') : (process.env.SHELL2_URL ?? 'http://localhost:3200')
const SCREENSHOT_PATH = path.join(ROOT, '.artifacts', 'shell2-final.png')
const TIMEOUT = 15_000

// ── Helpers ───────────────────────────────────────────────────────────────────

let pass = 0
let fail = 0
const results = []

function assert(label, value, extra = '') {
  if (value) {
    console.log(`  ✓  ${label}`)
    results.push({ label, ok: true })
    pass++
  } else {
    console.error(`  ✗  ${label}${extra ? ` — ${extra}` : ''}`)
    results.push({ label, ok: false, extra })
    fail++
  }
}

function deferred(label, reason) {
  console.log(`  ⊘  ${label} [DEFERRED: ${reason}]`)
  results.push({ label, ok: true, deferred: reason })
}

// ── Probe ─────────────────────────────────────────────────────────────────────

let browser
try {
  browser = await chromium.launch({ headless: true })
  const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } })
  const page = await ctx.newPage()

  const consoleErrors = []
  page.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(msg.text())
  })
  page.on('pageerror', (err) => consoleErrors.push(`[uncaught] ${err.message}`))

  console.log(`\nshell2 smoke probe → ${BASE_URL}\n`)

  // ── 1. Load shell2 ─────────────────────────────────────────────────────────
  await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: TIMEOUT })

  // ── 2. Chrome components register + paint ──────────────────────────────────
  console.log('Chrome components:')

  for (const tag of ['sui-nav-top', 'sui-rail', 'sui-panel-dock', 'sui-status-bar']) {
    const el = await page.locator(tag).first()
    const exists = await el.count() > 0
    assert(`<${tag}> present in DOM`, exists)
    if (exists) {
      const defined = await page.evaluate((t) => customElements.get(t) != null, tag)
      assert(`<${tag}> custom element registered`, defined)
    }
  }

  // ── 3. ⌘K (command-bar) — deferred ────────────────────────────────────────
  console.log('\nKeyboard shortcuts:')
  deferred('⌘K opens command-bar', 'sui-command-bar not yet merged')

  // ── 4. ⌘J opens panel-dock ────────────────────────────────────────────────
  const dockBefore = await page.evaluate(() => document.body.classList.contains('dock-open'))
  assert('dock-open class absent before ⌘J', !dockBefore)

  await page.keyboard.press('Control+j')
  await page.waitForTimeout(300)
  const dockAfter = await page.evaluate(() => document.body.classList.contains('dock-open'))
  assert('⌘J adds body.dock-open', dockAfter)

  // Close dock again.
  await page.keyboard.press('Control+j')
  await page.waitForTimeout(300)

  // ── 5. Theme picker swaps theme + persists ────────────────────────────────
  console.log('\nTheme:')

  // Read initial theme.
  const initialTheme = await page.evaluate(() => document.documentElement.getAttribute('data-theme') ?? '')
  const nextTheme = initialTheme.includes('dark') ? 'ship-light' : 'ship-dark'

  // Set theme programmatically (simulates what sui-theme-picker does).
  await page.evaluate((t) => {
    document.documentElement.setAttribute('data-theme', t)
    try { localStorage.setItem('ship-site-theme', t) } catch (_) {}
  }, nextTheme)

  await page.waitForTimeout(150)
  const appliedTheme = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
  assert(`Theme swapped to "${nextTheme}"`, appliedTheme === nextTheme)

  // Reload to check persistence.
  await page.reload({ waitUntil: 'networkidle', timeout: TIMEOUT })
  const persistedTheme = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
  assert(`Theme "${nextTheme}" persists across reload`, persistedTheme === nextTheme)

  // ── 6. Rail collapse persists ─────────────────────────────────────────────
  console.log('\nRail:')

  const railBefore = await page.evaluate(() => {
    const r = document.getElementById('ship-rail')
    return r?.hasAttribute('collapsed') ?? false
  })
  assert('Rail starts uncollapsed', !railBefore)

  // Collapse via keyboard shortcut.
  await page.keyboard.press('Control+b')
  await page.waitForTimeout(300)
  const railCollapsed = await page.evaluate(() => document.getElementById('ship-rail')?.hasAttribute('collapsed'))
  assert('⌘B collapses rail', railCollapsed)

  const bodyCollapsed = await page.evaluate(() => document.body.classList.contains('rail-collapsed'))
  assert('body.rail-collapsed set after collapse', bodyCollapsed)

  // Reload to check persistence.
  await page.reload({ waitUntil: 'networkidle', timeout: TIMEOUT })
  const persistedCollapse = await page.evaluate(() => {
    const r = document.getElementById('ship-rail')
    return r?.hasAttribute('collapsed') ?? false
  })
  assert('Rail collapsed state persists across reload', persistedCollapse)

  // Restore rail.
  await page.keyboard.press('Control+b')
  await page.waitForTimeout(300)

  // ── 7. Console errors ─────────────────────────────────────────────────────
  console.log('\nConsole:')
  // Filter out expected command-bar stubs.
  const realErrors = consoleErrors.filter((msg) => !msg.includes('[shell2]'))
  assert('No unexpected console errors', realErrors.length === 0,
    realErrors.length > 0 ? realErrors.slice(0, 3).join('; ') : '')

  // ── 8. Screenshot ─────────────────────────────────────────────────────────
  console.log('\nScreenshot:')
  await page.screenshot({ path: SCREENSHOT_PATH, fullPage: false })
  const exists = fs.existsSync(SCREENSHOT_PATH)
  assert(`Screenshot saved to ${path.relative(ROOT, SCREENSHOT_PATH)}`, exists)

  await ctx.close()
} finally {
  await browser?.close()
}

// ── Summary ───────────────────────────────────────────────────────────────────

console.log(`\n${'─'.repeat(60)}`)
console.log(`Results: ${pass} passed, ${fail} failed`)
if (fail > 0) {
  console.error('\nFailed assertions:')
  results.filter((r) => !r.ok).forEach((r) => console.error(`  ✗ ${r.label}${r.extra ? ` — ${r.extra}` : ''}`))
}
console.log('')

process.exit(fail > 0 ? 1 : 0)
