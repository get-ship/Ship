import { test } from '@playwright/test'

/**
 * Side-by-side visual parity check for the git view. Renders the React
 * reference (`/studio/git`) and the daemon-rendered variant
 * (`/studio/view/git`) and writes both full-page screenshots so a human
 * (or another agent iteration) can diff them.
 */
test('git views side by side', async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 900 })

  await page.goto('/studio/git')
  await page.waitForTimeout(2500)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/git-react.png',
    fullPage: true,
  })

  await page.goto('/studio/view/git')
  await page.waitForTimeout(2500)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/git-daemon.png',
    fullPage: true,
  })
})
