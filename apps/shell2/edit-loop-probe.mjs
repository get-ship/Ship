// End-to-end edit-loop probe for shell2 + the SUI HMR pipeline.
//
// What it verifies, against a live daemon + shell2 dev server:
//
//   1. Edit views/home/page.sui   → /_sui/dev/hmr fires `module-changed`
//      for `views/home/view`, shell2's HMR glue re-mounts the home view
//      in place (no full page reload), and the new string appears in the
//      DOM.
//   2. Edit views/home/controller.ts → same path (controller.ts is part of
//      the `views/home/view` bundle).
//   3. Edit views/home/style.css → expected to NOT fire HMR — `.css` is
//      outside the bundler watcher's extension list. The probe records
//      this as a documented gap, NOT as an unexpected failure.
//   4. Create views/edit-probe-tmp/{page.sui,controller.ts,manifest.jsonc}
//      → `/api/v1/views` should list it, direct navigation to
//      `/edit-probe-tmp` should resolve (server falls back to on-the-fly
//      compile because the bundler only discovers view targets at
//      startup), and the rail (hardcoded in apps/shell2/index.html) does
//      NOT pick it up without a daemon restart + rail edit. Documented.
//   5. `pnpm -F shell2 build` produces an apps/shell2/dist/ artifact.
//
// Run:  node apps/shell2/edit-loop-probe.mjs
// Requires: a running daemon (`just dev`) + shell2 vite dev server
//           (`pnpm -F shell2 dev`). Auto-resolves the vite port from the
//           per-worktree daemon dir; override with SHELL2_URL=…
//
// Side-effects + safety:
//   - Every file mutation is wrapped in try/finally that restores the
//     original bytes (or deletes the new dir for scenario 4). Even on
//     crash, the worst case is a partial restore — the probe prints the
//     paths it touched so you can `git checkout --` them.

import { chromium } from 'playwright'
import {
  mkdirSync, writeFileSync, readFileSync, existsSync,
  rmSync, statSync,
} from 'node:fs'
import { join, dirname } from 'node:path'
import { execFileSync, spawnSync } from 'node:child_process'

const REPO = dirname(dirname(new URL('.', import.meta.url).pathname)).replace(/\/$/, '')
const OUT = '/tmp/shell2-edit-loop-out'
mkdirSync(OUT, { recursive: true })

function resolveBaseUrl() {
  if (process.env.SHELL2_URL) return process.env.SHELL2_URL.replace(/\/$/, '')
  const candidates = [
    join(REPO, 'target', 'debug', 'ship'),
    join(REPO, 'target', 'release', 'ship'),
  ]
  for (const bin of candidates) {
    if (!existsSync(bin)) continue
    try {
      const dir = execFileSync(bin, ['daemon', 'dir'], { encoding: 'utf8' }).trim()
      const port = Number(readFileSync(join(dir, 'port'), 'utf8').trim())
      return `http://127.0.0.1:${3200 + (port - 9315)}`
    } catch { /* ignore */ }
  }
  return 'http://127.0.0.1:3200'
}

const BASE = resolveBaseUrl()
console.log(`probe target: ${BASE}`)

// ── filesystem helpers ───────────────────────────────────────────────

function backupOf(absPath) {
  const orig = readFileSync(absPath, 'utf8')
  return { absPath, orig }
}
function restoreAll(snapshots) {
  for (const s of snapshots) {
    try { writeFileSync(s.absPath, s.orig) } catch (e) {
      console.error(`!! restore failed for ${s.absPath}: ${e?.message}`)
    }
  }
}

// ── HMR event capture via browser EventSource ────────────────────────
//
// Attached as a pre-mount init script so the listener exists before the
// first navigation. window.__hmrEvents accumulates {kind,target,ts,…}
// records the probe inspects later via page.evaluate / waitForFunction.
async function installHmrCapture(page) {
  // Two parallel taps so we can attribute failures correctly:
  //
  //   window.__bundlerEvents  ← independent EventSource the probe opens
  //                              up front; confirms shipd's `/_sui/dev/hmr`
  //                              actually emits a module-changed event for
  //                              our edit. Decouples the "did the daemon
  //                              react?" question from the "did the shell
  //                              react?" question.
  //
  //   window.__shellEvents    ← mirrored from shell2's own EventSource
  //                              via a patched constructor. Stays empty if
  //                              shell2's `initHmr()` short-circuits.
  await page.addInitScript(() => {
    const w = window
    if (w.__hmrInstalled) return
    w.__hmrInstalled = true
    w.__bundlerEvents = []
    w.__shellEvents = []
    w.__esUrls = []
    w.__hmrReloads = 0

    const NativeES = w.EventSource
    w.EventSource = function PatchedES(url, init) {
      w.__esUrls.push(String(url))
      const es = new NativeES(url, init)
      if (String(url).includes('/_sui/dev/hmr')) {
        es.addEventListener('module-changed', (ev) => {
          try {
            const d = JSON.parse(ev.data)
            w.__shellEvents.push({ kind: 'module-changed', target: d.target, ts: Date.now() })
          } catch { /* ignore */ }
        })
      }
      return es
    }
    w.EventSource.prototype = NativeES.prototype

    try {
      const probe = new NativeES('/_sui/dev/hmr')
      probe.addEventListener('module-changed', (ev) => {
        try {
          const d = JSON.parse(ev.data)
          w.__bundlerEvents.push({ kind: 'module-changed', target: d.target, ts: Date.now() })
        } catch { /* ignore */ }
      })
      probe.addEventListener('compile-error', (ev) => {
        try {
          const d = JSON.parse(ev.data)
          w.__bundlerEvents.push({ kind: 'compile-error', target: d.target, message: d.message, ts: Date.now() })
        } catch { /* ignore */ }
      })
    } catch { /* ignore */ }

    window.addEventListener('beforeunload', () => { w.__hmrReloads += 1 })
  })
}

async function snap(page, name) {
  const file = `${OUT}/${name}.png`
  await page.screenshot({ path: file, fullPage: false })
  return file
}

async function waitForBundlerEvent(page, predicate, timeoutMs = 10000) {
  return page.waitForFunction((predSrc) => {
    const pred = eval(`(${predSrc})`)
    return (window.__bundlerEvents ?? []).some(pred)
  }, predicate.toString(), { timeout: timeoutMs }).then(() => true).catch(() => false)
}
async function shellSawEvent(page, predicate) {
  return page.evaluate((predSrc) => {
    const pred = eval(`(${predSrc})`)
    return (window.__shellEvents ?? []).some(pred)
  }, predicate.toString())
}

async function getDocId(page) {
  return page.evaluate(() => window.performance?.timeOrigin ?? Date.now())
}

// ── scenario runner ──────────────────────────────────────────────────

const results = []
function record(name, status, notes) {
  results.push({ name, status, notes })
  const stamp = status === 'pass' ? 'PASS' : status === 'fail' ? 'FAIL' : status.toUpperCase()
  console.log(`  [${stamp}] ${name} — ${notes}`)
}

const snapshots = []
const createdPaths = []

const browser = await chromium.launch({ headless: true })
const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } })
const page = await ctx.newPage()

page.on('console', (m) => {
  const t = m.text()
  if (t.startsWith('[sui:hmr]') || t.startsWith('[hmr]') || t.startsWith('[probe]')) {
    console.log(`  · ${m.type()}: ${t}`)
  }
})
page.on('requestfinished', (req) => {
  const u = req.url()
  if (u.includes('/_sui/views/home') || u.includes('/_sui/dev/hmr')) {
    console.log(`  · net: ${req.method()} ${u}`)
  }
})
page.on('pageerror', (e) => console.error(`  ! pageerror: ${e.message}`))

try {
  await installHmrCapture(page)
  console.log('=== 0. open /home ===')
  await page.goto(`${BASE}/home`, { waitUntil: 'load', timeout: 15000 })
  await page.waitForSelector('.home', { timeout: 10000 })
  await snap(page, '00-home-initial')
  const initialDocId = await getDocId(page)
  // Diagnostic: confirm the shell's runtime loaded and what EventSources
  // opened so far. The probe taps EventSource construction in init script
  // — if /_sui/dev/hmr never appears here, shell2's `initHmr()` returned
  // early (see findings 1+2 in the report: the `isDev()` check uses
  // aliased `import.meta`, which Vite does not transform).
  const diag = await page.evaluate(() => ({
    suiRuntimeReady: !!window.__suiRuntime,
    esUrls: window.__esUrls ?? [],
  }))
  console.log(`  · diag: ${JSON.stringify(diag)}`)

  // ── 1. edit page.sui ───────────────────────────────────────────────
  // Mutate the caption paragraph (light DOM). The home-hero title is
  // inside a shadow root; document.body.innerText skips it.
  console.log('=== 1. edit views/home/page.sui ===')
  const SUI_PATH = join(REPO, 'views', 'home', 'page.sui')
  const suiSnap = backupOf(SUI_PATH); snapshots.push(suiSnap)
  const markerSui = `EDIT-PROBE-SUI-${Date.now()}`
  const mutatedSui = suiSnap.orig.replace('Proof view for the new directory convention',
    `${markerSui} Proof view for the new directory convention`)
  if (mutatedSui === suiSnap.orig) throw new Error('marker substitution failed for page.sui')
  writeFileSync(SUI_PATH, mutatedSui)

  const sawSuiEvent = await waitForBundlerEvent(page,
    (e) => e.kind === 'module-changed' && e.target === 'views/home/view', 12000)
  await page.waitForTimeout(1200) // let the re-mount + payload fetch complete
  await snap(page, '01-after-page-sui')
  const docIdAfterSui = await getDocId(page)
  const reloaded = await page.evaluate(() => window.__hmrReloads)
  const markerVisible = await page.evaluate((m) => document.body.innerText.includes(m), markerSui)
  if (sawSuiEvent && markerVisible && reloaded === 0 && docIdAfterSui === initialDocId) {
    record('1. page.sui edit → in-place HMR', 'pass',
      'module-changed fired, marker visible in DOM, no full reload.')
  } else if (!sawSuiEvent) {
    record('1. page.sui edit → in-place HMR', 'fail',
      'bundler watcher did not emit views/home/view module-changed.')
  } else if (sawSuiEvent && !markerVisible) {
    const shellSaw = await shellSawEvent(page,
      (e) => e.kind === 'module-changed' && e.target === 'views/home/view')
    await page.reload({ waitUntil: 'load' })
    await page.waitForSelector('.home', { timeout: 10000 })
    const afterReload = await page.evaluate((m) => document.body.innerText.includes(m), markerSui)
    record('1. page.sui edit → in-place HMR', 'fail',
      `bundler emitted module-changed; shell2 received: ${shellSaw}; marker visible after manual reload: ${afterReload}. ${
        shellSaw
          ? 'Shell received the event but did not re-mount in place — check the import path or remount() wiring.'
          : 'Shell never even opened the /_sui/dev/hmr stream. Root cause: apps/shell2/src/hmr.ts `isDev()` reads `import.meta.env?.DEV` via an alias (`const meta = import.meta`); Vite\'s env transform only injects the literal form, so `meta.env` is undefined at runtime, `isDev()` returns false, `initHmr()` returns null, and the HMR EventSource is never opened.'
      }`)
  } else {
    record('1. page.sui edit → in-place HMR', 'fail',
      `hmrEvent=${sawSuiEvent} markerVisible=${markerVisible} reloaded=${reloaded} docIdChanged=${docIdAfterSui !== initialDocId}`)
  }

  // Restore page.sui before next scenario (sets up a clean delta for ctrl)
  writeFileSync(SUI_PATH, suiSnap.orig)
  await page.waitForTimeout(800)

  // ── 2. edit controller.ts ──────────────────────────────────────────
  // Replace the `branch` derived with a hardcoded marker. After re-mount
  // the live-from-the-daemon line should render that marker in the
  // `<strong data-test="live-branch">` cell.
  console.log('=== 2. edit views/home/controller.ts ===')
  const CTRL_PATH = join(REPO, 'views', 'home', 'controller.ts')
  const ctrlSnap = backupOf(CTRL_PATH); snapshots.push(ctrlSnap)
  const markerBranch = `HMR-CTRL-${Date.now().toString(36)}`
  const mutatedCtrl = ctrlSnap.orig.replace(
    'branch:    (s) => read(s).branch,',
    `branch:    (_s) => '${markerBranch}',`,
  )
  if (mutatedCtrl === ctrlSnap.orig) throw new Error('marker substitution failed for controller.ts')
  await page.evaluate(() => { window.__bundlerEvents.length = 0; window.__shellEvents.length = 0 })
  writeFileSync(CTRL_PATH, mutatedCtrl)

  const sawCtrlEvent = await waitForBundlerEvent(page,
    (e) => e.kind === 'module-changed' && e.target === 'views/home/view', 12000)
  await page.waitForTimeout(1200)
  await snap(page, '02-after-controller')
  const docIdAfterCtrl = await getDocId(page)
  const reloadedAfterCtrl = await page.evaluate(() => window.__hmrReloads)
  const seenVisible = await page.evaluate((m) =>
    document.body.innerText.includes(m), markerBranch)
  if (sawCtrlEvent && seenVisible && reloadedAfterCtrl === 0 && docIdAfterCtrl === initialDocId) {
    record('2. controller.ts edit → in-place HMR', 'pass',
      'module-changed fired, derived marker visible, no full reload.')
  } else if (sawCtrlEvent && !seenVisible) {
    const shellSaw = await shellSawEvent(page,
      (e) => e.kind === 'module-changed' && e.target === 'views/home/view')
    await page.reload({ waitUntil: 'load' })
    await page.waitForSelector('.home', { timeout: 10000 })
    const afterReload = await page.evaluate((m) => document.body.innerText.includes(m), markerBranch)
    record('2. controller.ts edit → in-place HMR', 'fail',
      `bundler emitted; shell2 received: ${shellSaw}; marker after manual reload: ${afterReload}. Same root cause as step 1.`)
  } else {
    record('2. controller.ts edit → in-place HMR', 'fail',
      `hmrEvent=${sawCtrlEvent} markerVisible=${seenVisible} reloaded=${reloadedAfterCtrl} docIdChanged=${docIdAfterCtrl !== initialDocId}`)
  }
  writeFileSync(CTRL_PATH, ctrlSnap.orig)
  await page.waitForTimeout(800)

  // ── 3. edit style.css ──────────────────────────────────────────────
  console.log('=== 3. edit views/home/style.css ===')
  const CSS_PATH = join(REPO, 'views', 'home', 'style.css')
  const cssSnap = backupOf(CSS_PATH); snapshots.push(cssSnap)
  const markerColor = 'rgb(123, 200, 78)'
  // Append a unique selector that should bind to the existing .home div.
  const mutatedCss = cssSnap.orig + `\n.home { outline: 3px solid ${markerColor}; }\n`
  await page.evaluate(() => { window.__bundlerEvents.length = 0; window.__shellEvents.length = 0 })
  writeFileSync(CSS_PATH, mutatedCss)

  const sawCssEvent = await waitForBundlerEvent(page,
    (e) => e.kind === 'module-changed' && e.target === 'views/home/view', 4000)
  await page.waitForTimeout(500)
  // Did the visual style apply? Check the live element's computed outline.
  const cssApplied = await page.evaluate((c) => {
    const el = document.querySelector('.home')
    if (!el) return null
    return getComputedStyle(el).outlineColor === c
  }, markerColor)
  await snap(page, '03-after-css')
  if (sawCssEvent && cssApplied) {
    record('3. style.css edit → HMR + CSS reapplied', 'pass',
      'module-changed fired and computed outline matched.')
  } else if (!sawCssEvent && !cssApplied) {
    record('3. style.css edit → HMR + CSS reapplied', 'fail',
      'no HMR event AND no CSS reapply. Bundler watcher ignores .css; the new style only appears after a navigation away+back (which re-fetches /_sui/views/home → fresh `css` field).')
  } else {
    record('3. style.css edit → HMR + CSS reapplied', 'partial',
      `hmrEvent=${sawCssEvent} cssApplied=${cssApplied}`)
  }
  // Verify the navigate-away-and-back recovery path as evidence.
  await page.goto(`${BASE}/dashboard`, { waitUntil: 'load', timeout: 10000 })
  await page.waitForTimeout(400)
  await page.goto(`${BASE}/home`, { waitUntil: 'load', timeout: 10000 })
  await page.waitForSelector('.home', { timeout: 10000 })
  const cssAppliedAfterNav = await page.evaluate((c) => {
    const el = document.querySelector('.home')
    return el ? getComputedStyle(el).outlineColor === c : null
  }, markerColor)
  console.log(`  · css visible after nav-away-and-back: ${cssAppliedAfterNav}`)
  writeFileSync(CSS_PATH, cssSnap.orig)
  await page.waitForTimeout(400)

  // ── 4. add a fresh view directory ──────────────────────────────────
  console.log('=== 4. add a new view directory ===')
  const NEW_DIR = join(REPO, 'views', 'edit-probe-tmp')
  mkdirSync(NEW_DIR, { recursive: true })
  createdPaths.push(NEW_DIR)
  const markerNew = `NEW-VIEW-PROBE-${Date.now()}`
  writeFileSync(join(NEW_DIR, 'page.sui'),
    `<div class="probe"><h1>${markerNew}</h1></div>\n`)
  writeFileSync(join(NEW_DIR, 'controller.ts'),
    `import { controller, signal } from '@ship/sui-runtime'\nexport default controller({ scope: () => ({ probe: signal(true) }) })\n`)
  writeFileSync(join(NEW_DIR, 'manifest.jsonc'), JSON.stringify({
    name: 'edit-probe-tmp',
    path: '/edit-probe-tmp',
    title: 'Edit Probe (tmp)',
    layout: 'default',
    auth: 'public',
    rail: { label: 'Edit Probe', icon: 'flask-conical' },
  }, null, 2) + '\n')

  // (a) does /api/v1/views list it?
  const viewsJson = await page.evaluate(async () => {
    const r = await fetch('/api/v1/views', { credentials: 'same-origin' })
    return r.ok ? r.json() : null
  })
  const listed = !!(viewsJson?.data?.views ?? viewsJson?.views ?? [])
    .some((v) => v.name === 'edit-probe-tmp')
  console.log(`  · listed in /api/v1/views: ${listed}`)

  // (b) does direct navigation render the view?
  await page.goto(`${BASE}/edit-probe-tmp`, { waitUntil: 'load', timeout: 15000 })
  await page.waitForTimeout(1200)
  await snap(page, '04-new-view')
  const newViewRendered = await page.evaluate((m) =>
    document.body.innerText.includes(m), markerNew)

  // (c) does the rail show it?
  const railHasIt = await page.evaluate(() => {
    const items = document.querySelectorAll('sui-rail-item')
    return Array.from(items).some((el) => el.getAttribute('data-view') === 'edit-probe-tmp'
      || el.getAttribute('href') === '/edit-probe-tmp')
  })

  if (newViewRendered && listed && railHasIt) {
    record('4. new view dir', 'pass',
      'listed, route renders, rail picks it up.')
  } else if (newViewRendered && listed && !railHasIt) {
    record('4. new view dir', 'partial',
      'route renders via URL and is listed in /api/v1/views, but the rail is static (hardcoded in apps/shell2/index.html) — it never re-registers from the views catalog.')
  } else {
    record('4. new view dir', 'fail',
      `rendered=${newViewRendered} listed=${listed} railHasIt=${railHasIt}`)
  }

  rmSync(NEW_DIR, { recursive: true, force: true })
  createdPaths.length = 0
  await page.waitForTimeout(400)

  // ── 5. prod build ──────────────────────────────────────────────────
  console.log('=== 5. prod build smoke: pnpm -F shell2 build ===')
  const build = spawnSync('pnpm', ['-F', 'shell2', 'build'], {
    cwd: REPO,
    encoding: 'utf8',
    env: { ...process.env, CI: '1' },
    timeout: 180_000,
  })
  const distIndex = join(REPO, 'apps', 'shell2', 'dist', 'index.html')
  const distExists = existsSync(distIndex)
  const distSize = distExists ? statSync(distIndex).size : 0
  if (build.status === 0 && distExists && distSize > 0) {
    record('5. shell2 prod build', 'pass',
      `vite build exit 0, dist/index.html ${distSize}B`)
  } else {
    const tail = (build.stderr || build.stdout || '').split('\n').slice(-8).join(' | ')
    record('5. shell2 prod build', 'fail',
      `exit=${build.status} dist=${distExists} tail=${tail}`)
  }
} finally {
  restoreAll(snapshots)
  for (const p of createdPaths) {
    try { rmSync(p, { recursive: true, force: true }) } catch { /* ignore */ }
  }
  await browser.close()
}

console.log('\n=== Summary ===')
for (const r of results) console.log(`  ${r.status.toUpperCase().padEnd(7)} ${r.name}`)
writeFileSync(join(OUT, 'results.json'), JSON.stringify(results, null, 2))
console.log(`\nartifacts: ${OUT}`)
process.exit(results.every((r) => r.status === 'pass' || r.status === 'partial') ? 0 : 1)
