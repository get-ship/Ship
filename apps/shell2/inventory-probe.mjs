// Systematic inventory of shell + every view.
// Output: a JSON report classifying each surface by working state.
import { chromium } from 'playwright'
import { writeFileSync, mkdirSync } from 'node:fs'

const OUT = '/tmp/shell2-inventory'
mkdirSync(OUT, { recursive: true })

const b = await chromium.launch({ headless: true })
const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } })
const p = await ctx.newPage()

const errsByRoute = new Map()
let curRoute = '/'
p.on('console', m => {
  if (m.type() === 'error') {
    const arr = errsByRoute.get(curRoute) ?? []
    arr.push(m.text().slice(0, 250))
    errsByRoute.set(curRoute, arr)
  }
})
p.on('pageerror', e => {
  const arr = errsByRoute.get(curRoute) ?? []
  arr.push('[pe] ' + e.message.slice(0, 250))
  errsByRoute.set(curRoute, arr)
})

await p.goto('http://127.0.0.1:3270/', { waitUntil: 'load', timeout: 8000 })
await p.waitForTimeout(800)
await p.screenshot({ path: `${OUT}/00-shell.png`, fullPage: false })

// Chrome inventory
const chrome = await p.evaluate(() => {
  const out = {}
  // rail
  const rail = document.querySelector('sui-rail')
  out.rail = {
    visible: rail ? rail.offsetWidth > 0 : false,
    width: rail?.offsetWidth ?? 0,
    items: Array.from(document.querySelectorAll('sui-rail-item')).map(r => ({
      icon: r.getAttribute('icon'),
      label: r.getAttribute('label'),
      href: r.getAttribute('href'),
    })),
  }
  // nav-top
  const nav = document.querySelector('sui-nav-top')
  out.nav = {
    visible: nav ? nav.offsetWidth > 0 : false,
    height: nav?.offsetHeight ?? 0,
    children: nav?.shadowRoot
      ? Array.from(nav.shadowRoot.querySelectorAll('sui-button, sui-theme-picker, a.brand'))
          .map(c => c.getAttribute('part') || c.tagName.toLowerCase())
      : [],
  }
  // panel-dock
  const dock = document.querySelector('sui-panel-dock')
  out.dock = { present: !!dock, height: dock?.offsetHeight ?? 0, hidden: dock?.hidden ?? false }
  // command bar
  const cb = document.querySelector('sui-command-bar')
  out.commandBar = { present: !!cb, width: cb?.offsetWidth ?? 0 }
  return out
})
console.log('CHROME:', JSON.stringify(chrome, null, 2))

// Per-view probe via direct goto + screenshot
const views = [
  '/dashboard', '/home', '/agents', '/overview', '/sessions', '/workspaces',
  '/skills', '/chat', '/mesh', '/events', '/git', '/canvas', '/specs',
  '/collections', '/pipelines', '/pipelines-board', '/theme-studio', '/setup',
  '/ship-ui-studio',
]
const viewReport = []
for (const v of views) {
  curRoute = v
  errsByRoute.set(v, [])
  const start = Date.now()
  let status = 'err'
  try {
    const resp = await p.goto(`http://127.0.0.1:3270${v}`, { waitUntil: 'load', timeout: 5000 })
    status = String(resp?.status() ?? 'err')
  } catch (e) {
    status = `timeout`
  }
  await p.waitForTimeout(800)
  const slug = v.replace(/\//g, '_') || 'root'
  await p.screenshot({ path: `${OUT}/view${slug}.png`, fullPage: false })

  const dom = await p.evaluate(() => {
    const main = document.querySelector('#ship-content') || document.body
    return {
      // Element count under main
      els: main.querySelectorAll('*').length,
      // Visible text content (rough)
      text: main.innerText?.slice(0, 200) || '',
      // Loading state markers
      hasSkeleton: !!main.querySelector('sui-skeleton'),
      hasError: !!main.querySelector('sui-error-state, .shell-error'),
      hasEmpty: !!main.querySelector('sui-empty-state'),
      // Data-bound elements still showing "Loading..."
      loadingText: main.innerText?.match(/Loading/i) ? true : false,
    }
  })
  viewReport.push({
    view: v,
    status,
    timeMs: Date.now() - start,
    ...dom,
    errors: errsByRoute.get(v) ?? [],
  })
}

writeFileSync(`${OUT}/report.json`, JSON.stringify({ chrome, views: viewReport }, null, 2))

// Print compact table
console.log('\n=== VIEW INVENTORY ===')
console.log('view\t\t\tstatus\tels\tload\terr\tnotes')
viewReport.forEach(r => {
  const flag =
    r.errors.length > 0 ? '⚠' :
    r.hasError ? '❌' :
    r.loadingText ? '⏳' :
    r.els < 20 ? '·' :
    '✓'
  console.log(`${r.view.padEnd(24)}\t${r.status}\t${r.els}\t${r.loadingText ? 'L' : '-'}\t${r.errors.length}\t${flag} ${r.text.slice(0, 50).replace(/\s+/g, ' ')}`)
})

await b.close()
console.log(`\nReport: ${OUT}/report.json`)
console.log(`Screenshots: ${OUT}/`)
