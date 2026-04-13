import { test } from '@playwright/test'

test('pipelines mockup reference', async ({ page }) => {
  await page.goto(
    'file:///home/madvib/dev/ship-worktrees/web-components/.artifacts/ui-proposal/pipelines.html',
  )
  await page.waitForTimeout(500)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/pipelines-mockup.png',
    fullPage: true,
  })
})

test('pipelines daemon view', async ({ page }) => {
  await page.goto('/studio/view/pipelines')
  await page.waitForTimeout(2500)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/pipelines-daemon.png',
    fullPage: true,
  })
})
