/**
 * Harness smoke — every harness view must mount via /view-host/<name>
 * without uncaught page errors and render real DOM. Captures a
 * screenshot per view to tests/e2e/screenshots/harness-<name>.png so
 * regressions show up in the diff.
 */
import { test, expect } from '@playwright/test'
import { getDaemonUrl, screenshot } from './_helpers'

const IGNORED = /ResizeObserver|Non-Error promise rejection/

const VIEWS = [
  'harness',
  'harness-board',
  'harness-inbox',
  'harness-sync',
  'harness-lab-motion',
  'harness-lab-forms',
  'harness-lab-virt',
  'harness-lab-modals',
  'harness-lab-charts',
  'harness-lab-theme',
] as const

test.describe('harness smoke', () => {
  for (const view of VIEWS) {
    test(`/view-host/${view} mounts`, async ({ page }) => {
      const errors: string[] = []
      page.on('pageerror', (err) => errors.push(err.message))

      await page.goto(`${getDaemonUrl()}/view-host/${view}`, { waitUntil: 'load' })

      // Wait until SUI mount drops real content into the page body.
      await page.waitForFunction(
        () => document.querySelector('.ship-view') !== null,
        null,
        { timeout: 15_000 },
      )

      const present = await page.locator('.ship-view').count()
      expect(present).toBeGreaterThan(0)

      // Give live-source tiles + animations a beat so the screenshot
      // captures the steady state.
      await page.waitForTimeout(700)
      await screenshot(page, `harness-${view}`)

      const real = errors.filter((e) => !IGNORED.test(e))
      expect(real, `page errors on /view-host/${view}:\n${real.join('\n')}`).toHaveLength(0)
    })
  }
})
