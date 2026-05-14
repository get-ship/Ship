// keyboard-probe.mjs — verifies ⌘B / ⌘J / ⌘K shortcuts in shell2.
//
// Run against a live dev server:
//   node apps/shell2/keyboard-probe.mjs
//
// Expected: three checks pass, plus the trigger-pill click test.
//
// The probe uses Ctrl on Linux (matches the global handler's
// `e.metaKey || e.ctrlKey` and the keymap shown in the top-nav tooltips).

import { chromium } from 'playwright'

const URL = process.env.PROBE_URL ?? 'http://127.0.0.1:3270/'
const MOD = process.platform === 'darwin' ? 'Meta' : 'Control'

const failures = []
function check(label, ok, detail) {
  const tag = ok ? 'PASS' : 'FAIL'
  console.log(`[${tag}] ${label}${detail ? '  ' + detail : ''}`)
  if (!ok) failures.push(label)
}

const browser = await chromium.launch({ headless: true })
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } })
const page = await ctx.newPage()

await page.goto(URL, { waitUntil: 'load' })
// Wait until the chrome islands are upgraded and main.ts has wired its keydown handler.
await page.waitForFunction(() => {
  const bar = document.getElementById('ship-command-bar')
  const dock = document.getElementById('ship-panel-dock')
  const rail = document.getElementById('ship-rail')
  return !!(bar && typeof bar.toggle === 'function'
    && dock && typeof dock.toggle === 'function'
    && rail)
}, null, { timeout: 8000 })

// Snapshot of initial state.
const initial = await page.evaluate(() => ({
  railCollapsed: document.getElementById('ship-rail').hasAttribute('collapsed'),
  dockOpen:      document.body.classList.contains('dock-open'),
  barOpen:       document.getElementById('ship-command-bar').hasAttribute('open'),
}))

// ── ⌘B / Ctrl+B → rail toggle ─────────────────────────────────────
await page.keyboard.press(`${MOD}+B`)
await page.waitForTimeout(120)
const afterB = await page.evaluate(() => ({
  collapsed: document.getElementById('ship-rail').hasAttribute('collapsed'),
}))
check('Ctrl+B toggles rail [collapsed] attribute',
  afterB.collapsed !== initial.railCollapsed,
  `before=${initial.railCollapsed} after=${afterB.collapsed}`)

// Restore rail.
await page.keyboard.press(`${MOD}+B`)
await page.waitForTimeout(120)

// ── ⌘J / Ctrl+J → dock toggle ────────────────────────────────────
await page.keyboard.press(`${MOD}+J`)
await page.waitForTimeout(150)
const afterJ = await page.evaluate(() => ({
  dockOpen: document.body.classList.contains('dock-open'),
}))
check('Ctrl+J toggles panel dock (body.dock-open)',
  afterJ.dockOpen !== initial.dockOpen,
  `before=${initial.dockOpen} after=${afterJ.dockOpen}`)

// Restore dock.
await page.keyboard.press(`${MOD}+J`)
await page.waitForTimeout(150)

// ── ⌘K / Ctrl+K → command bar open ───────────────────────────────
await page.keyboard.press(`${MOD}+K`)
await page.waitForTimeout(200)
const afterK = await page.evaluate(() => {
  const bar = document.getElementById('ship-command-bar')
  return {
    open:  bar.hasAttribute('open'),
    state: bar.getAttribute('data-state'),
  }
})
check('Ctrl+K opens command bar',
  afterK.open === true && afterK.state === 'open',
  `open=${afterK.open} data-state=${afterK.state}`)

// Close via Escape so the trigger-click test starts from a closed bar.
await page.keyboard.press('Escape')
await page.waitForTimeout(150)
const afterEsc = await page.evaluate(() => ({
  open: document.getElementById('ship-command-bar').hasAttribute('open'),
}))
check('Escape closes the command bar', afterEsc.open === false,
  `open=${afterEsc.open}`)

// ── Click the trigger pill → command bar open ────────────────────
// The pill lives inside <sui-command-bar>'s shadow root; reach in directly.
await page.evaluate(() => {
  const bar = document.getElementById('ship-command-bar')
  const btn = bar?.shadowRoot?.querySelector('button.cb-trigger')
  btn?.click()
})
await page.waitForTimeout(200)
const afterClick = await page.evaluate(() => {
  const bar = document.getElementById('ship-command-bar')
  return {
    open:  bar.hasAttribute('open'),
    mode:  bar.getAttribute('mode'),
    state: bar.getAttribute('data-state'),
  }
})
check('Clicking the cb-trigger pill opens the bar in find mode',
  afterClick.open === true && afterClick.mode === 'find' && afterClick.state === 'open',
  `open=${afterClick.open} mode=${afterClick.mode} state=${afterClick.state}`)

await browser.close()

if (failures.length) {
  console.error(`\n${failures.length} check(s) failed:`)
  for (const f of failures) console.error('  - ' + f)
  process.exit(1)
}
console.log('\nAll keyboard-shortcut checks passed.')
