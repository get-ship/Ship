#!/usr/bin/env bun
// Web-QA probe for /git — exercises the depth-pass features against a
// running daemon for this worktree. Captures screenshots per theme and
// writes findings to ../report.md by the runner.
//
// Usage: bun probe.mjs
//
// Reads daemon port from ~/.ship/machine/daemons/<hash>/port (matches
// path_hash = sha256(realpath(workspace))[..16]). Override with SHIPD_URL.

import { chromium } from '@playwright/test'
import crypto from 'node:crypto'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

const ROOT = path.resolve(import.meta.dirname, '../../..')
const OUT  = import.meta.dirname

function daemonUrl() {
  if (process.env.SHIPD_URL) return process.env.SHIPD_URL
  const real = fs.realpathSync(ROOT)
  // Same algo as runtime::home::path_hash (sha256 of path bytes, first 8 hex).
  const hash = crypto.createHash('sha256').update(real).digest('hex').slice(0, 8)
  const portFile = path.join(os.homedir(), '.ship', 'machine', 'daemons', hash, 'port')
  const port = fs.readFileSync(portFile, 'utf8').trim()
  return `http://localhost:${port}`
}

const findings = []
let warnings = 0
let errors = 0

function note(level, label, detail = '') {
  findings.push({ level, label, detail, ts: new Date().toISOString() })
  if (level === 'error') errors++
  if (level === 'warn') warnings++
  console.log(`[${level}] ${label}${detail ? ' — ' + detail : ''}`)
}

async function shotEach(page, basename) {
  const out = path.join(OUT, `${basename}.png`)
  await page.screenshot({ path: out, fullPage: true })
  note('info', `screenshot ${basename}.png`)
}

async function setTheme(page, theme) {
  await page.evaluate((t) => {
    document.documentElement.setAttribute('data-theme', t)
  }, theme)
  await page.waitForTimeout(150)
}

async function main() {
  const base = daemonUrl()
  note('info', 'daemon', base)

  // Pair so the in-browser SDK has an authenticated session for mutation
  // calls. Dev mode auto-approves: pair_init returns a session_token we
  // forward to subsequent calls via the Authorization header.
  let bearer = ''
  try {
    const r = await fetch(`${base}/auth/pair/init`, {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ client_label: 'web-qa' }),
    })
    const j = await r.json()
    if (j?.dev_mode && j?.claim_token) {
      const c = await fetch(`${base}/auth/pair/status?claim_token=${encodeURIComponent(j.claim_token)}`)
      const cj = await c.json()
      if (cj?.status === 'approved' && cj?.session_token) {
        bearer = cj.session_token
        note('info', 'paired session via dev auto-approve')
      } else {
        note('warn', 'pair status not approved', JSON.stringify(cj))
      }
    } else {
      note('warn', 'pair_init no dev_mode', JSON.stringify(j))
    }
  } catch (e) {
    note('warn', 'pairing failed', String(e))
  }

  const browser = await chromium.launch()
  console.log('browser launched')
  const ctx = await browser.newContext({
    viewport: { width: 1400, height: 900 },
    extraHTTPHeaders: bearer ? { Authorization: `Bearer ${bearer}` } : {},
  })
  const page = await ctx.newPage()

  const jsErrors = []
  page.on('pageerror', (e) => jsErrors.push(e.message))
  const consoleErrs = []
  page.on('console', (m) => {
    if (m.type() === 'error') consoleErrs.push(m.text())
  })
  const failedRequests = []
  page.on('requestfailed', (r) => failedRequests.push(`${r.method()} ${r.url()} — ${r.failure()?.errorText ?? ''}`))

  // ── 1. /sui/git boots ───────────────────────────────────────────────
  await page.goto(`${base}/sui/git`)
  await page.waitForFunction(() => typeof window.__suiDispose !== 'undefined', { timeout: 15_000 })
  note('info', 'attach: __suiDispose set')

  // Surface any early errors before waiting on the live source.
  if (jsErrors.length) note('error', 'pre-resolve pageerror', jsErrors.join(' | '))
  if (consoleErrs.length) note('warn', 'pre-resolve console error', consoleErrs.slice(0,3).join(' | '))

  // Wait for live overview to resolve
  try {
    await page.locator('.git-head').waitFor({ timeout: 20_000 })
    note('info', 'overview resolved → .git-head visible')
  } catch (e) {
    note('error', 'overview did not resolve', String(e))
    if (jsErrors.length) note('error', 'pageerrors', jsErrors.join('\n  | '))
    if (consoleErrs.length) note('error', 'console errors', consoleErrs.slice(0, 20).join('\n  | '))
    if (failedRequests.length) note('error', 'failed requests', failedRequests.slice(0, 10).join('\n  | '))
    // Dump the live DOM around #sui-root for triage.
    const html = await page.evaluate(() => document.getElementById('sui-root')?.outerHTML ?? '')
    fs.writeFileSync(path.join(OUT, 'failure-dom.html'), html)
    await shotEach(page, 'failure-overview')
    await browser.close()
    return
  }

  // ── 2. Captures: theme matrix ──────────────────────────────────────
  for (const theme of ['ship-light', 'ship-dark', 'console']) {
    await setTheme(page, theme)
    await shotEach(page, `git-${theme}`)
  }
  await setTheme(page, 'ship-dark')

  // ── 3. Status panel sanity ─────────────────────────────────────────
  const branch = await page.locator('.git-head .git-sub .git-mono').first().textContent()
  note('info', `branch=${branch?.trim()}`)
  const totalText = await page.locator('.git-head sui-chip').first().textContent()
  note('info', `status chip=${totalText?.trim()}`)

  // ── 4. File rows present (we have uncommitted work) ────────────────
  const fileRows = page.locator('[data-test="git-file-row"]')
  const fileCount = await fileRows.count()
  if (fileCount === 0) {
    note('warn', 'no file rows — repo is clean, skipping stage/commit')
  } else {
    note('info', `file rows=${fileCount}`)
    await fileRows.first().click()
    try {
      await page.locator('[data-test="diff-body"]').waitFor({ timeout: 8_000 })
      note('info', 'diff body opened after file row click')
    } catch {
      note('error', 'diff body did not appear')
    }
    await shotEach(page, 'git-diff-open')

    // ── 5. Stage a file — should toggle staged count ────────────────
    const stageBtn = page.locator('[data-test="git-stage-file"]').first()
    if (await stageBtn.count()) {
      const stagedBefore = await page.locator('[data-test="git-open-commit"]').textContent()
      await stageBtn.click()
      try {
        await page.waitForFunction(
          (prev) => document.querySelector('[data-test="git-open-commit"]')?.textContent !== prev,
          stagedBefore, { timeout: 8_000 },
        )
        const stagedAfter = await page.locator('[data-test="git-open-commit"]').textContent()
        note('info', `commit btn label: ${stagedBefore?.trim()} → ${stagedAfter?.trim()}`)
      } catch {
        note('warn', 'commit button label did not change after stage — overview may not have refreshed')
      }
    } else {
      note('warn', 'no untracked/modified file to stage')
    }
    await shotEach(page, 'git-after-stage')

    // ── 6. Open commit dialog ───────────────────────────────────────
    const commitBtn = page.locator('[data-test="git-open-commit"]')
    if (!(await commitBtn.getAttribute('disabled'))) {
      await commitBtn.click()
      try {
        // sui-dialog reflects `open` as both attribute and DOM property.
        // Probe via DOM property to be safe.
        await page.waitForFunction(
          () => {
            const d = document.querySelector('[data-test="git-commit-dialog"]')
            return !!(d && (d.hasAttribute('open') || d.open))
          },
          null, { timeout: 5_000 },
        )
        note('info', 'commit dialog opened')
        await shotEach(page, 'git-commit-dialog')
        await page.keyboard.press('Escape')
      } catch {
        note('error', 'commit dialog did not open')
      }
    } else {
      note('warn', 'commit button disabled — no staged files')
    }
  }

  // ── 7. Switch to Branches tab + verify checkout button visible ────
  await page.locator('.git-tabs-row [name="branches"]').click()
  try {
    await page.locator('[data-test="git-branch-tile"]').first().waitFor({ timeout: 5_000 })
    note('info', 'branches tab renders tiles')
    // Click a non-active branch (last one), check Checkout button appears
    const tiles = page.locator('[data-test="git-branch-tile"]')
    const n = await tiles.count()
    if (n > 1) {
      await tiles.nth(n - 1).click()
      try {
        await page.locator('[data-test="git-checkout"]').waitFor({ timeout: 3_000 })
        note('info', 'Checkout button appears for non-active branch')
      } catch {
        note('warn', 'Checkout button not found for non-active branch')
      }
    }
    await shotEach(page, 'git-branches')
  } catch {
    note('error', 'branches tab failed to render tiles')
  }

  // ── 8. History tab ─────────────────────────────────────────────────
  await page.locator('.git-tabs-row [name="history"]').click()
  try {
    await page.locator('[data-test="history-rows"]').waitFor({ timeout: 5_000 })
    note('info', 'history tab renders rows')
    await shotEach(page, 'git-history')
  } catch {
    note('warn', 'history rows not present (maybe empty)')
  }

  // ── 9. Page-level console errors ──────────────────────────────────
  if (jsErrors.length > 0) note('error', `js errors: ${jsErrors.length}`, jsErrors.join('\n'))
  if (consoleErrs.length > 0) note('warn', `console errors: ${consoleErrs.length}`, consoleErrs.slice(0, 5).join('\n'))
  if (failedRequests.length > 0) note('warn', `failed requests: ${failedRequests.length}`, failedRequests.slice(0, 5).join('\n'))

  await browser.close()

  if (jsErrors.length) note('error', 'final pageerrors', jsErrors.slice(0, 20).join('\n  | '))
  if (consoleErrs.length) note('warn', 'final console errors', consoleErrs.slice(0, 20).join('\n  | '))
  if (failedRequests.length) note('warn', 'final failed requests', failedRequests.slice(0, 20).join('\n  | '))

  // Emit a summary JSON for the runner.
  fs.writeFileSync(path.join(OUT, 'findings.json'), JSON.stringify({ findings, warnings, errors }, null, 2))
  console.log(`\nfindings=${findings.length} warnings=${warnings} errors=${errors}`)
}

main().catch((e) => { console.error(e); process.exit(2) })
