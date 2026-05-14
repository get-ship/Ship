// agent-detail-probe.mjs — verifies the /agents/[id] overview page.
//
// Probes 4 routes:
//   /agents/commander        — should render h1 + sections
//   /agents/gatekeeper       — should render h1 + sections
//   /agents/rust-lane        — should render h1 + sections
//   /agents/nonexistent-id   — should show "No such agent" empty state
//
// Captures: h1 text, section presence, console + page errors.
// Writes a JSON report to /tmp/agent-detail-probe-out/report.json.

import { chromium } from 'playwright'
import { mkdirSync, writeFileSync, existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { execFileSync } from 'node:child_process'

const OUT = '/tmp/agent-detail-probe-out'
mkdirSync(OUT, { recursive: true })

function resolveBaseUrl() {
  if (process.env.SHELL2_URL) return process.env.SHELL2_URL.replace(/\/$/, '')
  const repoRoot = process.cwd().replace(/\/apps\/shell2$/, '')
  try {
    const bin = `${repoRoot}/target/debug/ship`
    if (existsSync(bin)) {
      const dir = execFileSync(bin, ['daemon', 'dir'], { encoding: 'utf8' }).trim()
      const port = readFileSync(join(dir, 'port'), 'utf8').trim()
      const vite = 3200 + (Number(port) - 9315)
      return `http://127.0.0.1:${vite}`
    }
  } catch { /* ignore */ }
  return 'http://127.0.0.1:3270'
}

const BASE = resolveBaseUrl()
const TARGETS = ['commander', 'gatekeeper', 'rust-lane', 'nonexistent-id']

async function probe(page, id) {
  const consoleErrors = []
  const pageErrors = []
  const onConsole = (msg) => { if (msg.type() === 'error') consoleErrors.push(msg.text()) }
  const onPageError = (err) => { pageErrors.push(err.message) }
  page.on('console', onConsole)
  page.on('pageerror', onPageError)

  await page.goto(`${BASE}/agents/${id}`, { waitUntil: 'domcontentloaded', timeout: 15000 })
  // Give SUI a beat to finish the live() subscription render. SSE keeps
  // the network "busy" so we can't rely on networkidle.
  await page.waitForTimeout(2500)

  const result = {
    id,
    url: page.url(),
    h1: await page.locator('h1').first().textContent().catch(() => null),
    hasDetail: await page.locator('[data-test="agent-detail"]').count() > 0,
    hasNotFound: await page.locator('[data-test="agent-not-found"]').count() > 0,
    name: await page.locator('[data-test="agent-name"]').first().textContent().catch(() => null),
    source: await page.locator('[data-test="agent-source"]').first().textContent().catch(() => null),
    description: await page.locator('[data-test="agent-description"]').first().textContent().catch(() => null),
    stats: {
      skills: await page.locator('[data-test="stat-skills"] .ad-stat-val').first().textContent().catch(() => null),
      mcp: await page.locator('[data-test="stat-mcp"] .ad-stat-val').first().textContent().catch(() => null),
      rules: await page.locator('[data-test="stat-rules"] .ad-stat-val').first().textContent().catch(() => null),
      providers: await page.locator('[data-test="stat-providers"] .ad-stat-val').first().textContent().catch(() => null),
    },
    sections: {
      skills: await page.locator('[data-test="section-skills"]').count() > 0,
      mcp: await page.locator('[data-test="section-mcp"]').count() > 0,
      rules: await page.locator('[data-test="section-rules"]').count() > 0,
      permissions: await page.locator('[data-test="section-permissions"]').count() > 0,
      meta: await page.locator('[data-test="section-meta"]').count() > 0,
    },
    skillLinks: await page.locator('[data-test="skill-link"]').count(),
    mcpRows: await page.locator('[data-test="mcp-row"]').count(),
    ruleRows: await page.locator('[data-test="rule-row"]').count(),
    consoleErrors,
    pageErrors,
  }

  page.off('console', onConsole)
  page.off('pageerror', onPageError)
  await page.screenshot({ path: join(OUT, `${id}.png`), fullPage: true }).catch(() => null)
  return result
}

async function main() {
  console.log(`[probe] base=${BASE}`)
  const results = []
  for (const id of TARGETS) {
    // Fresh browser per target so a single hang or close doesn't poison
    // the rest of the run.
    const browser = await chromium.launch()
    try {
      const page = await browser.newPage()
      const r = await probe(page, id)
      results.push(r)
      console.log(`[probe] ${id}: h1=${JSON.stringify(r.h1)} detail=${r.hasDetail} notFound=${r.hasNotFound} errs=${r.consoleErrors.length}/${r.pageErrors.length}`)
    } catch (e) {
      results.push({ id, error: e.message })
      console.log(`[probe] ${id}: ERROR ${e.message}`)
    } finally {
      await browser.close().catch(() => null)
    }
  }
  writeFileSync(join(OUT, 'report.json'), JSON.stringify(results, null, 2))
  console.log(`[probe] wrote ${join(OUT, 'report.json')}`)
}

main().catch((e) => { console.error(e); process.exit(1) })
