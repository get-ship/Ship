import { test, expect } from '@playwright/test'
import fs from 'node:fs'

const TOKEN = fs.readFileSync(`${process.env.HOME}/.ship/daemon.token`, 'utf8').trim()

// Each view's root element id, populated by its SDK shell template.
// If the wrapper is working and Datastar fetched the shell, this
// element is present with non-empty children — no "Connecting…" stall.
const VIEWS: Array<{ name: string; rootId: string }> = [
  { name: 'pipelines', rootId: 'pipelines-root' },
  { name: 'specs', rootId: 'specs-root' },
  { name: 'git', rootId: 'git-root' },
  { name: 'settings', rootId: 'settings-root' },
  { name: 'agents', rootId: 'agents-root' },
  { name: 'skills', rootId: 'skills-root' },
  { name: 'chat', rootId: 'chat-root' },
  { name: 'events', rootId: 'events-root' },
]

test.beforeEach(async ({ page }) => {
  // Seed the daemon session token so PlatformProvider skips the pairing flow.
  // The token is shared across browser contexts, mirroring the real dev setup.
  await page.addInitScript((t) => {
    try { localStorage.setItem('ship-studio:session-token', t) } catch { /* ignore */ }
  }, TOKEN)
})

for (const { name, rootId } of VIEWS) {
  test(`${name} via Studio populates`, async ({ page }) => {
    const consoleErrors: string[] = []
    page.on('console', (msg) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })
    page.on('pageerror', (err) => consoleErrors.push(err.message))
    page.on('requestfailed', (req) => {
      if (req.url().includes('/api/runtime')) consoleErrors.push(`FAIL ${req.url()} ${req.failure()?.errorText}`)
    })

    await page.goto(`/studio/view/${name}`)

    // Studio embeds each view as an iframe sourced from the daemon.
    const frame = page.frameLocator(`iframe[title="${name}"]`)
    await expect(frame.locator(`#${rootId}`)).toBeAttached({ timeout: 15000 })
    await expect(frame.locator(`#${rootId}`)).not.toHaveText(/Connecting/i)
    const childCount = await frame.locator(`#${rootId} *`).count()
    expect(childCount, `${name} root must have children`).toBeGreaterThan(0)

    await page.screenshot({ path: `../../.artifacts/qa-screenshots/studio-${name}.png`, fullPage: true })
    if (consoleErrors.length) console.log(`\n--- ${name} console/network errors ---\n${consoleErrors.join('\n')}\n`)
  })

  test(`${name} direct daemon populates`, async ({ page }) => {
    await page.goto(`http://127.0.0.1:9315/api/runtime/view/${name}/raw?token=${TOKEN}`)
    await expect(page.locator(`#${rootId}`)).toBeAttached({ timeout: 5000 })
    await expect(page.locator(`#${rootId}`)).not.toHaveText(/Connecting/i)
    await page.screenshot({ path: `../../.artifacts/qa-screenshots/direct-${name}.png`, fullPage: true })
  })
}
