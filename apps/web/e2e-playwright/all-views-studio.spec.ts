import { test } from '@playwright/test'
import fs from 'node:fs'

const TOKEN = fs.readFileSync(`${process.env.HOME}/.ship/daemon.token`, 'utf8').trim()

const VIEWS = ['pipelines', 'specs', 'git', 'settings']

for (const name of VIEWS) {
  test(`${name} via Studio`, async ({ page }) => {
    const consoleErrors: string[] = []
    page.on('console', (msg) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })
    page.on('pageerror', (err) => consoleErrors.push(err.message))
    page.on('requestfailed', (req) => {
      if (req.url().includes('/api/runtime')) consoleErrors.push(`FAIL ${req.url()} ${req.failure()?.errorText}`)
    })

    await page.goto(`/studio/view/${name}`)
    await page.waitForTimeout(3500)
    await page.screenshot({ path: `../../.artifacts/qa-screenshots/studio-${name}.png`, fullPage: true })
    if (consoleErrors.length) console.log(`\n--- ${name} console/network errors ---\n${consoleErrors.join('\n')}\n`)
  })

  test(`${name} direct daemon`, async ({ page }) => {
    await page.goto(`http://127.0.0.1:9315/api/runtime/view/${name}/raw?token=${TOKEN}`)
    await page.waitForTimeout(2500)
    await page.screenshot({ path: `../../.artifacts/qa-screenshots/direct-${name}.png`, fullPage: true })
  })
}
