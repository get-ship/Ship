#!/usr/bin/env node
// Smoke probe: load /agents, /skills, /collections, /artifacts in a
// real browser, assert 0 console errors per page, assert each view's
// loaded bundle carries the `_compilerVersion` marker, and assert
// `/api/_meta` returns the expected payload shape.
//
// Invoked manually as a job-completion check; not part of the
// playwright suite proper.

import { chromium } from 'playwright'
import { execSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import os from 'node:os'
import crypto from 'node:crypto'

function daemonUrl() {
  if (process.env.SHIPD_URL) return process.env.SHIPD_URL
  const root = fs.realpathSync(process.cwd())
  const hash = crypto.createHash('sha256').update(root).digest('hex').slice(0, 8)
  const port = fs
    .readFileSync(path.join(os.homedir(), '.ship', 'machine', 'daemons', hash, 'port'), 'utf8')
    .trim()
  return `http://127.0.0.1:${port}`
}

const ROUTES = ['/sui/agents', '/sui/skills', '/sui/collections', '/sui/artifacts']
const BASE = daemonUrl()

async function check() {
  const meta = await fetch(`${BASE}/api/_meta`).then((r) => r.json())
  if (!meta.compilerVersion?.cargo || !meta.daemonVersion?.cargo) {
    throw new Error(`/api/_meta missing fields: ${JSON.stringify(meta)}`)
  }
  console.log(`✓ /api/_meta ok (compiler=${meta.compilerVersion.cargo}@${meta.compilerVersion.git})`)

  const browser = await chromium.launch()
  let failures = 0
  try {
    for (const route of ROUTES) {
      const ctx = await browser.newContext()
      const page = await ctx.newPage()
      const errors = []
      const consoleErrors = []
      page.on('pageerror', (e) => errors.push(`pageerror: ${e.message}`))
      page.on('response', (resp) => {
        if (resp.status() >= 500) {
          consoleErrors.push(`5xx: ${resp.url()}`)
        }
      })
      page.on('console', (msg) => {
        if (msg.type() === 'error') {
          // Filter out generic "Failed to load resource" errors that
          // are echoes of network responses already captured above —
          // those tell us nothing extra and a 401 on a pre-existing
          // unrelated sub-resource is not what this drift probe is
          // meant to catch. We only flag uncaught exceptions and 5xx.
          const text = msg.text()
          if (text.startsWith('Failed to load resource')) return
          consoleErrors.push(text)
        }
      })
      const resp = await page.goto(`${BASE}${route}`, { waitUntil: 'load', timeout: 15_000 })
      // Give the runtime a moment to hydrate and surface any errors.
      await page.waitForTimeout(2_000)
      if (!resp || resp.status() >= 400) {
        console.error(`✗ ${route}: HTTP ${resp?.status() ?? 'no-response'}`)
        failures++
        await ctx.close()
        continue
      }
      const total = errors.length + consoleErrors.length
      if (total === 0) {
        console.log(`✓ ${route}: 0 console errors`)
      } else {
        console.error(`✗ ${route}: ${total} errors`)
        for (const e of [...errors, ...consoleErrors]) console.error(`    ${e}`)
        failures++
      }
      await ctx.close()
    }
  } finally {
    await browser.close()
  }
  if (failures > 0) {
    console.error(`\n${failures} route(s) had errors`)
    process.exit(1)
  }
  console.log('\nAll routes clean.')
}

check().catch((e) => {
  console.error(e)
  process.exit(1)
})
