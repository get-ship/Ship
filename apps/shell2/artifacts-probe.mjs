// Probe /artifacts view: table or empty state, filter row controls,
// detail drawer wiring, upload dialog, zero console errors.
//
// Assumes shell2 is running at http://127.0.0.1:${VITE_PORT||3270} with
// a live daemon (so /api/v1/artifacts responds 200). Exits non-zero if
// any acceptance check fails.

import { chromium } from 'playwright'
import { mkdirSync } from 'node:fs'

const PORT = process.env.VITE_PORT ?? '3270'
const BASE = `http://127.0.0.1:${PORT}`
const OUT = '/tmp/shell2-artifacts-probe'
mkdirSync(OUT, { recursive: true })

const failures = []
function check(label, cond, detail) {
  if (cond) {
    console.log(`  ok  ${label}`)
  } else {
    console.log(`  FAIL ${label}${detail ? ' — ' + detail : ''}`)
    failures.push(label)
  }
}

const b = await chromium.launch({ headless: true })
const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } })
const page = await ctx.newPage()

const consoleErrors = []
page.on('console', (m) => {
  if (m.type() === 'error') consoleErrors.push(m.text().slice(0, 400))
})
page.on('pageerror', (e) => consoleErrors.push('[pe] ' + e.message.slice(0, 400)))

await page.goto(`${BASE}/artifacts`, { waitUntil: 'load', timeout: 8000 })
await page.waitForTimeout(1200)
await page.screenshot({ path: `${OUT}/01-initial.png`, fullPage: false })

const initial = await page.evaluate(() => {
  const root = document.querySelector('.ship-view-artifacts')
  return {
    rootPresent: !!root,
    title: document.querySelector('.art-title')?.textContent ?? '',
    hasTable: !!document.querySelector('[data-test="table"]'),
    hasEmpty: !!document.querySelector('[data-test="empty"]'),
    hasError: !!document.querySelector('[data-test="error"]'),
    hasLoading: !!document.querySelector('[data-test="loading"]'),
    uploadBtn: !!document.querySelector('[data-test="upload-button"]'),
    rowCount: document.querySelectorAll('[data-test="row"]').length,
  }
})
console.log('Initial:', JSON.stringify(initial, null, 2))

check('view root present', initial.rootPresent)
check('header title rendered', initial.title.includes('Artifacts'))
check('upload button present', initial.uploadBtn)
check('a primary surface (table or empty) is visible',
  initial.hasTable || initial.hasEmpty || initial.hasError)

// ── Filters smoke: only when the table is on screen ─────────────────────────
if (initial.hasTable) {
  const filterShape = await page.evaluate(() => ({
    search: !!document.querySelector('[data-test="filter-search"]'),
    mime: !!document.querySelector('[data-test="filter-mime"]'),
    collection: !!document.querySelector('[data-test="filter-collection"]'),
  }))
  check('filter row: search input', filterShape.search)
  check('filter row: mime select', filterShape.mime)
  check('filter row: collection select', filterShape.collection)

  // Type into search — the row count should fall to 0 quickly for a junk query.
  const before = initial.rowCount
  await page.evaluate(() => {
    const s = document.querySelector('[data-test="filter-search"]')
    if (!s) return
    s.value = '__no_match_query__'
    s.dispatchEvent(new Event('input', { bubbles: true }))
  })
  await page.waitForTimeout(300)
  const afterFilter = await page.evaluate(() => ({
    rowCount: document.querySelectorAll('[data-test="row"]').length,
    emptyFiltered: !!document.querySelector('[data-test="empty-filtered"]'),
  }))
  check('search filter narrows or empties the table',
    afterFilter.rowCount < before || afterFilter.emptyFiltered,
    `before=${before} after=${afterFilter.rowCount}`)

  // Clear search for follow-up upload-dialog assertions.
  await page.evaluate(() => {
    const s = document.querySelector('[data-test="filter-search"]')
    if (!s) return
    s.value = ''
    s.dispatchEvent(new Event('input', { bubbles: true }))
  })
  await page.waitForTimeout(150)
  await page.screenshot({ path: `${OUT}/02-filters.png`, fullPage: false })
}

// ── Upload dialog opens, closes ─────────────────────────────────────────────
await page.click('[data-test="upload-button"]', { timeout: 2000 }).catch(() => {})
await page.waitForTimeout(400)
const upload = await page.evaluate(() => {
  const dlg = document.querySelector('[data-test="upload-dialog"]')
  return {
    open: dlg?.hasAttribute('open') ?? false,
    hasFileInput: !!document.querySelector('[data-test="upload-files"]'),
    hasCollectionSelect: !!document.querySelector('[data-test="upload-collection"]'),
    hasSubmit: !!document.querySelector('[data-test="upload-submit"]'),
  }
})
check('upload dialog opens', upload.open)
check('upload form has file picker', upload.hasFileInput)
check('upload form has collection select', upload.hasCollectionSelect)
check('upload form has submit button', upload.hasSubmit)
await page.screenshot({ path: `${OUT}/03-upload-dialog.png`, fullPage: false })

// Close dialog (ESC).
await page.keyboard.press('Escape')
await page.waitForTimeout(300)
const closed = await page.evaluate(() => {
  const dlg = document.querySelector('[data-test="upload-dialog"]')
  return !dlg?.hasAttribute('open')
})
check('upload dialog closes on Escape', closed)

// ── Detail drawer: only if there's at least one row ─────────────────────────
if (initial.rowCount > 0) {
  await page.evaluate(() => {
    const row = document.querySelector('[data-test="row"] td:not(.art-cell-action)')
    row?.click()
  })
  await page.waitForTimeout(500)
  const drawer = await page.evaluate(() => {
    const dr = document.querySelector('[data-test="detail-drawer"]')
    return {
      open: dr?.hasAttribute('open') ?? false,
      hasPreview: !!document.querySelector('[data-test="detail-preview"]'),
    }
  })
  check('row-click opens detail drawer', drawer.open)
  check('detail drawer has a preview region', drawer.hasPreview)
  await page.screenshot({ path: `${OUT}/04-detail-drawer.png`, fullPage: false })
}

// ── Console hygiene ─────────────────────────────────────────────────────────
const filtered = consoleErrors.filter((e) => !/favicon|net::ERR_/i.test(e))
check('zero console errors', filtered.length === 0,
  filtered.length ? filtered.slice(0, 3).join(' | ') : '')

await b.close()

console.log(`\nScreenshots: ${OUT}/`)
if (failures.length) {
  console.log(`\nFAILED ${failures.length}: ${failures.join('; ')}`)
  process.exit(1)
}
console.log('\nPASSED all checks')
