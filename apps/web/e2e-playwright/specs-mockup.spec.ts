import { test } from '@playwright/test'

test('specs matches mockup IA', async ({ page }) => {
  await page.goto('/studio/view/specs')
  await page.waitForTimeout(2500)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/specs-daemon.png',
    fullPage: true,
  })
})

test('specs detail pane with selected spec', async ({ page }) => {
  await page.goto('/studio/view/specs')
  await page.waitForTimeout(2500)
  // Click the ping spec row (title starts with "Add fn ping()").
  const row = page.locator('ship-list-tile').filter({ hasText: 'Add fn ping' }).first()
  await row.click()
  await page.waitForSelector('.specs-detail-inner', { timeout: 10_000 })
  await page.waitForTimeout(1500)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/specs-daemon-selected.png',
    fullPage: true,
  })
})

test('specs mockup reference', async ({ page }) => {
  await page.goto(
    'file:///home/madvib/dev/ship-worktrees/web-components/.artifacts/ui-proposal/specs.html',
  )
  await page.waitForTimeout(500)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/specs-mockup.png',
    fullPage: true,
  })
})
