// Drive /ship-ui-studio across every published SUI component and grade each one.
//
// For each catalog entry:
//   - navigate to /ship-ui-studio/<id>
//   - capture console + pageerror noise
//   - assert host element exists, has shadowRoot (or non-empty light tree), occupies >0 px
//   - count matrix cells rendered live
//   - probe the copy-code button (sets clipboard / button text flips)
//   - interactive smoke: focus inputs, click buttons (without producing extra errors)
//
// Writes a JSON report + markdown table to /tmp/studio-audit/.
import { chromium } from 'playwright'
import { writeFileSync, mkdirSync } from 'node:fs'

const OUT = '/tmp/studio-audit'
mkdirSync(OUT, { recursive: true })

const BASE = process.env.STUDIO_BASE ?? 'http://127.0.0.1:3270'

const b = await chromium.launch({ headless: true })
const ctx = await b.newContext({
  viewport: { width: 1440, height: 900 },
  permissions: ['clipboard-read', 'clipboard-write'],
})
const p = await ctx.newPage()

// Per-route console + page error collection
let curRoute = '/'
const noiseByRoute = new Map()
function pushNoise(level, txt) {
  const arr = noiseByRoute.get(curRoute) ?? []
  arr.push({ level, txt: String(txt).slice(0, 280) })
  noiseByRoute.set(curRoute, arr)
}
p.on('console', m => {
  if (m.type() === 'error' || m.type() === 'warning') pushNoise(m.type(), m.text())
})
p.on('pageerror', e => pushNoise('pageerror', e.message))

// Stage 1: hit the index, read the catalog list
curRoute = '/ship-ui-studio'
noiseByRoute.set(curRoute, [])
await p.goto(`${BASE}/ship-ui-studio`, { waitUntil: 'load', timeout: 15000 })
await p.waitForFunction(
  () => document.querySelectorAll('[data-test="catalog-item"]').length > 0,
  { timeout: 12000 },
)
await p.waitForTimeout(400)

const catalog = await p.evaluate(() => {
  const items = Array.from(document.querySelectorAll('[data-test="catalog-item"]'))
  return items.map(a => {
    const code = a.querySelector('.catalog-item-name')
    const badge = a.querySelector('sui-badge')
    return {
      id: code?.textContent?.trim() ?? '',
      kind: badge?.textContent?.trim() ?? '',
      href: a.getAttribute('href') ?? '',
    }
  })
})
console.log(`Found ${catalog.length} catalog items.`)
await p.screenshot({ path: `${OUT}/00-index.png` })

const indexNoise = noiseByRoute.get('/ship-ui-studio') ?? []

// Stage 2: per-component probes
const report = []
for (const item of catalog) {
  const route = `/ship-ui-studio/${item.id}`
  curRoute = route
  noiseByRoute.set(route, [])

  let nav = 'ok'
  try {
    await p.goto(`${BASE}${route}`, { waitUntil: 'load', timeout: 8000 })
  } catch (e) {
    nav = 'timeout'
  }
  // Wait for either the rendered host or a not-found marker
  try {
    await p.waitForFunction(
      () => {
        const slot = document.querySelector('[data-render-slot]')
        const nf = document.querySelector('.detail-not-found')
        return (slot && slot.children.length > 0) || nf
      },
      { timeout: 4500 },
    )
  } catch {
    // fall through; probe will record empty slot
  }
  // Wait for the host element to upgrade. Island-loaded heavy
  // components (code-editor, markdown-editor) lazy-import on first
  // render and need longer than the default settle.
  try {
    await p.waitForFunction(
      (tag) => {
        if (!tag) return true
        const slot = document.querySelector('[data-render-slot]')
        const host = slot?.firstElementChild
        if (!host) return false
        return customElements.get(host.tagName.toLowerCase()) !== undefined
      },
      item.id,
      { timeout: 2500 },
    )
  } catch { /* leave un-upgraded; the observed-block flags it */ }
  await p.waitForTimeout(120)

  let observed
  try {
    observed = await p.evaluate((tag) => {
    const slot = document.querySelector('[data-render-slot]')
    const host = slot?.firstElementChild ?? null
    const isExpected = host ? host.tagName.toLowerCase() === tag : false
    let shadowSize = 0
    let lightSize = 0
    let widthPx = 0
    let heightPx = 0
    let upgraded = false
    if (host) {
      upgraded = customElements.get(host.tagName.toLowerCase()) !== undefined
      widthPx = host.getBoundingClientRect().width
      heightPx = host.getBoundingClientRect().height
      lightSize = host.children.length
      try {
        shadowSize = host.shadowRoot ? host.shadowRoot.children.length : 0
      } catch { /* closed */ }
    }
    const matrixCells = document.querySelectorAll('[data-matrix-cell]').length
    const matrixHosts = Array.from(document.querySelectorAll('[data-matrix-cell]'))
      .map(h => h.firstElementChild)
      .filter(c => c && customElements.get(c.tagName.toLowerCase()) !== undefined).length
    const props = document.querySelectorAll('[data-test="prop-row"]').length
    const copyBtn = document.querySelector('[data-test="copy-code"]')
    const sourceLink = document.querySelector('[data-test="source-link"]')
    const notFound = !!document.querySelector('.detail-not-found')
    const frame = document.querySelector('iframe.detail-view-frame')
    return {
      isExpected,
      hostTag: host?.tagName?.toLowerCase() ?? null,
      upgraded,
      shadowSize,
      lightSize,
      widthPx,
      heightPx,
      matrixCells,
      matrixHosts,
      props,
      hasCopy: !!copyBtn,
      hasSource: !!sourceLink,
      notFound,
      hasFrame: !!frame,
      frameSrc: frame?.getAttribute('src') ?? null,
    }
    }, item.id)
  } catch (e) {
    observed = { evalError: String(e).slice(0, 200) }
  }

  if (observed.evalError) {
    report.push({
      id: item.id, kind: item.kind, status: 'fail',
      notes: 'eval-error', observed, errors: [observed.evalError],
    })
    process.stdout.write('E')
    continue
  }

  // Interaction smoke: click copy-code if present (for sui kind)
  let copyOk = null
  if (observed.hasCopy && item.kind === 'sui') {
    try {
      // Use force + no-wait to avoid clicks that trigger popovers redirecting.
      await p.evaluate(() => {
        const b = document.querySelector('[data-test="copy-code"]')
        b?.click()
      })
      await p.waitForTimeout(150)
      copyOk = await p.evaluate(() => {
        const b = document.querySelector('[data-test="copy-code"]')
        return b?.getAttribute('data-copied') === 'true' || (b?.textContent ?? '').includes('Copied')
      })
    } catch {
      copyOk = false
    }
  }

  // Toggle first prop to confirm reactivity (only for sui kind w/ props)
  let propToggleOk = null
  if (item.kind === 'sui' && observed.props > 0) {
    try {
      propToggleOk = await p.evaluate(() => {
        const row = document.querySelector('[data-test="prop-row"]')
        if (!row) return false
        const sel = row.querySelector('select.prop-select')
        const inp = row.querySelector('input.prop-input')
        const cb = row.querySelector('input.prop-checkbox')
        const host = document.querySelector('[data-render-slot]')?.firstElementChild
        if (!host) return false
        const before = host.outerHTML.slice(0, 400)
        if (sel && sel.options.length > 1) {
          const newIdx = (sel.selectedIndex + 1) % sel.options.length
          sel.selectedIndex = newIdx
          sel.dispatchEvent(new Event('change', { bubbles: true }))
        } else if (cb) {
          cb.checked = !cb.checked
          cb.dispatchEvent(new Event('change', { bubbles: true }))
        } else if (inp) {
          inp.value = inp.value + 'x'
          inp.dispatchEvent(new Event('input', { bubbles: true }))
        } else {
          return false
        }
        // Allow microtask
        return new Promise(r => setTimeout(() => {
          const after = host.outerHTML.slice(0, 400)
          r(before !== after)
        }, 60))
      })
    } catch {
      propToggleOk = false
    }
  }

  try {
    await p.screenshot({ path: `${OUT}/${item.id}.png` })
  } catch { /* navigation can disrupt screenshot */ }

  const noise = noiseByRoute.get(route) ?? []
  // Shell chrome (sui-panel-dock + sui-nav-top, both prebuilt under
  // assets/sui-islands/) imports `fade` + `registerTransition` from
  // @ship/sui-runtime — neither export exists in the current runtime,
  // so every page in the shell logs these on first paint regardless
  // of which view is mounted. They're a pre-existing chrome bug, not
  // a per-component signal. Filter them so the audit's "errors" field
  // reflects actual problems with the route under test.
  const errors = noise
    .filter(n => n.level === 'pageerror' || n.level === 'error')
    .filter(n => !/does not provide an export named '(fade|registerTransition)'/.test(n.txt))

  // Grade
  let status = 'pass'
  const notes = []
  if (observed.notFound) {
    status = 'fail'
    notes.push('not-found')
  }
  if (item.kind === 'sui') {
    if (!observed.hostTag) {
      status = 'fail'
      notes.push('no-host')
    } else if (!observed.isExpected) {
      status = 'fail'
      notes.push(`wrong-host:${observed.hostTag}`)
    } else if (!observed.upgraded) {
      status = 'fail'
      notes.push('not-upgraded')
    } else if (observed.shadowSize === 0 && observed.lightSize === 0 && observed.widthPx < 4 && observed.heightPx < 4) {
      status = status === 'fail' ? 'fail' : 'partial'
      notes.push('empty-host')
    }
    if (copyOk === false) {
      status = status === 'fail' ? 'fail' : 'partial'
      notes.push('copy-broken')
    }
    if (propToggleOk === false) {
      status = status === 'fail' ? 'fail' : 'partial'
      notes.push('prop-noop')
    }
  } else if (item.kind === 'view') {
    if (!observed.hasFrame) {
      status = 'fail'
      notes.push('no-frame')
    }
  }
  if (errors.length > 0) {
    if (status === 'pass') status = 'partial'
    notes.push(`${errors.length}err`)
  }

  report.push({
    id: item.id,
    kind: item.kind,
    status,
    notes: notes.join(','),
    observed,
    copyOk,
    propToggleOk,
    nav,
    errors: errors.map(e => `${e.level}: ${e.txt}`),
  })
  process.stdout.write(`${status === 'pass' ? '.' : status === 'partial' ? '~' : 'X'}`)
}
process.stdout.write('\n')

writeFileSync(`${OUT}/report.json`, JSON.stringify({ indexNoise, report }, null, 2))

// Markdown table
const lines = []
lines.push('| component | kind | status | notes |')
lines.push('|-----------|------|--------|-------|')
for (const r of report) {
  lines.push(`| ${r.id} | ${r.kind} | ${r.status} | ${r.notes || ''} |`)
}
writeFileSync(`${OUT}/report.md`, lines.join('\n') + '\n')

// Compact stdout summary
const pass = report.filter(r => r.status === 'pass').length
const partial = report.filter(r => r.status === 'partial').length
const fail = report.filter(r => r.status === 'fail').length
console.log(`\nTotal: ${report.length}  pass:${pass}  partial:${partial}  fail:${fail}`)
console.log('\nFAILS:')
for (const r of report.filter(r => r.status === 'fail')) {
  console.log(`  ${r.id}: ${r.notes}`)
  for (const e of r.errors.slice(0, 2)) console.log(`    ${e}`)
}
console.log('\nPARTIALS:')
for (const r of report.filter(r => r.status === 'partial')) {
  console.log(`  ${r.id}: ${r.notes}`)
  for (const e of r.errors.slice(0, 2)) console.log(`    ${e}`)
}

await b.close()
console.log(`\nReport: ${OUT}/report.json`)
console.log(`Markdown: ${OUT}/report.md`)
