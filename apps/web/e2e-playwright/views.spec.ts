import { test, expect, Page } from '@playwright/test'

interface ViewReport {
  name: string
  consoleErrors: string[]
  networkErrors: string[]
  rootPresent: boolean
  shipTagsCount: number
  connectingStuck: boolean
  tabClickable: boolean | 'n/a'
}

async function probeView(page: Page, name: string): Promise<ViewReport> {
  const consoleErrors: string[] = []
  const networkErrors: string[] = []

  page.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(msg.text())
  })
  page.on('pageerror', (err) => {
    if (!/WebSocket/i.test(err.message)) consoleErrors.push(err.message)
  })
  page.on('requestfailed', (req) => {
    networkErrors.push(`${req.method()} ${req.url()} — ${req.failure()?.errorText}`)
  })

  await page.goto(`/studio/view/${name}`)
  await page.waitForTimeout(3500) // let SSE initial patch land

  const rootPresent = (await page.locator(`#${name}-root`).count()) > 0
  const shipTagsCount = await page.evaluate(() =>
    document.querySelectorAll('[id*="-root"] *').length
      ? document.querySelectorAll('[class*="ship-view"] *').length
      : 0,
  )
  const connectingStuck = await page.getByText(/connecting/i).isVisible().catch(() => false)

  // Try clicking first tab after the active one
  let tabClickable: boolean | 'n/a' = 'n/a'
  const tabs = page.locator('ship-tab').filter({ hasNot: page.locator('[active]') })
  if ((await tabs.count()) > 0) {
    try {
      await tabs.first().click({ timeout: 1000 })
      await page.waitForTimeout(1500)
      tabClickable = networkErrors.filter((e) => e.includes('/api/runtime')).length === 0
    } catch {
      tabClickable = false
    }
  }

  await page.screenshot({ path: `../../.artifacts/qa-screenshots/view-${name}.png`, fullPage: true })

  return { name, consoleErrors, networkErrors, rootPresent, shipTagsCount, connectingStuck, tabClickable }
}

const VIEWS = ['pipelines', 'specs', 'git', 'settings']

test.describe('daemon-rendered views', () => {
  for (const name of VIEWS) {
    test(`${name} renders and interacts`, async ({ page }) => {
      const report = await probeView(page, name)
      console.log(`\n=== ${name} report ===`)
      console.log(JSON.stringify(report, null, 2))

      expect(report.rootPresent, `#${name}-root should exist`).toBe(true)
      expect(report.connectingStuck, 'should not be stuck on Connecting…').toBe(false)
      expect(report.shipTagsCount, 'should have ship-* elements rendered').toBeGreaterThan(0)
    })
  }
})
