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

/**
 * Drill-in parity — click a commit on each side, then snapshot the full
 * diff viewer so we can compare the React DiffViewer against the
 * <ship-git-diff>-family rendered by the daemon.
 */
test('git detail view side by side', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 1600 })

  // React side: first commit in the log.
  await page.goto('/studio/git')
  await page.waitForTimeout(2500)
  const reactCommit = page.locator('button:has(span.font-mono)').first()
  if (await reactCommit.count()) {
    await reactCommit.click()
    await page.waitForTimeout(1500)
  }
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/git-detail-react.png',
    fullPage: true,
  })

  // Daemon side: click a commit in the log. Loop from top until we land on
  // one whose diff actually parses into files — merge commits are rendered
  // by `git show` as a combined-diff which the parser intentionally skips.
  await page.goto('/studio/view/git')
  await page.waitForTimeout(2500)
  for (let i = 0; i < 5; i++) {
    const c = page.locator('.ship-git-commit').nth(i)
    if (!(await c.count())) break
    await c.click()
    await page.waitForTimeout(1500)
    const files = await page.locator('ship-git-diff-file').count()
    if (files > 0) break
    // Return to the log and try the next commit.
    await page.locator('.ship-git-back').click()
    await page.waitForTimeout(800)
  }
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/git-detail-daemon.png',
    fullPage: true,
  })
})
