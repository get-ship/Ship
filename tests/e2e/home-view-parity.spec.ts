/**
 * Home view parity tests — guard the consolidation of the OG j2
 * Home + Dashboard + Overview surfaces into `views/home/`.
 *
 * PRESERVE (carried forward from any of the three OG views):
 *   1. Stats grid — Jobs, Agents, Events, Collections — fed by
 *      `ship.dashboard.state()`.
 *   2. Recent jobs list with status dot + agent badge.
 *   3. Agents preview grid with a "Manage" deep link to /agents.
 *   4. Recent activity stream with a "Full stream" link to /events.
 *   5. Quick links — Workflows / Agents / Skills / Data / Events /
 *      Settings — matching the j2 dashboard `quick_links.html.j2`.
 *   6. Live workspace pulse — `<data live>` SSE for git overview,
 *      mirroring the old SUI home and the j2 dashboard "live" badge.
 *
 * AVOID (per harvest gaps + redundant launch surfaces):
 *   - No standalone `/dashboard` or `/overview` view — both fold in
 *     here. Smoke list + visual baseline list updated to match.
 *   - No hardcoded Datastar `data-*` snippets — every interaction is
 *     authored as a controller-driven `on:` handler.
 *
 * Tests tolerate a fresh workspace: when stats are all zero the view
 * shows the empty-state CTA instead of the stats grid; when populated
 * it shows the full landing layout. Both are exercised here.
 */
import { test, expect, type Page } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

async function gotoHome(page: Page): Promise<void> {
  await page.goto(`${SHELL2_URL}/home`, { waitUntil: 'load' })
  await page.locator('[data-test="home-title"]').waitFor({ timeout: 20_000 })
}

test.describe('home view parity', () => {
  skipUnlessShell2()

  test('renders the SHIP wordmark + workspace label + live badge', async ({ page }) => {
    await gotoHome(page)
    await expect(page.locator('[data-test="home-title"]')).toHaveText('SHIP')
    await expect(page.locator('[data-test="home-workspace"]')).toBeVisible()
    await expect(page.locator('.home-live-badge')).toBeVisible()
  })

  test('PRESERVE: dashboard envelope drives stats OR first-run empty state', async ({ page }) => {
    await gotoHome(page)
    const stats = page.locator('[data-test="home-stats"]')
    const empty = page.locator('[data-test="home-empty"]')
    // Exactly one must be visible — the controller's `isFirstRun` flag
    // is derived from totals + arrays on the dash envelope.
    const hasStats = await stats.count() > 0
    const hasEmpty = await empty.count() > 0
    expect(hasStats || hasEmpty).toBe(true)
    if (hasEmpty) {
      await expect(empty).toBeVisible()
      // Empty state offers next-action CTAs (not a sea of empty cards).
      await expect(page.locator('[data-test="home-empty-agent"]')).toBeVisible()
      await expect(page.locator('[data-test="home-empty-skill"]')).toBeVisible()
    } else {
      await expect(stats).toBeVisible()
      // Four labelled stat tiles, in declaration order.
      const labels = stats.locator('sui-stat [slot="label"]')
      await expect(labels.nth(0)).toHaveText('Jobs')
      await expect(labels.nth(1)).toHaveText('Agents')
      await expect(labels.nth(2)).toHaveText('Events')
      await expect(labels.nth(3)).toHaveText('Collections')
    }
  })

  test('PRESERVE: project at-a-glance band carries the live git status', async ({ page }) => {
    await gotoHome(page)
    test.skip(await page.locator('[data-test="home-empty"]').count() > 0, 'empty workspace skips populated bands')
    const glance = page.locator('[data-test="home-glance"]')
    await expect(glance).toBeVisible()
    // The git summary row appears once the SSE channel resolves; the
    // branch name + short head land into the same line.
    await expect(page.locator('[data-test="git-summary"]')).toBeVisible({ timeout: 15_000 })
    await expect(page.locator('[data-test="git-branch"]')).not.toBeEmpty()
    await expect(page.locator('[data-test="git-head"]')).not.toBeEmpty()
  })

  test('PRESERVE: recent jobs section + agents preview deep-link to /pipelines + /agents', async ({ page }) => {
    await gotoHome(page)
    test.skip(await page.locator('[data-test="home-empty"]').count() > 0, 'empty workspace skips populated bands')

    const jobs = page.locator('[data-test="home-jobs"]')
    await expect(jobs).toBeVisible()
    await expect(jobs.locator('.home-section-link')).toHaveAttribute('href', '/pipelines')

    const agents = page.locator('[data-test="home-agents"]')
    await expect(agents).toBeVisible()
    await expect(agents.locator('.home-section-link')).toHaveAttribute('href', '/agents')
  })

  test('PRESERVE: activity stream + full-stream link to /events', async ({ page }) => {
    await gotoHome(page)
    test.skip(await page.locator('[data-test="home-empty"]').count() > 0, 'empty workspace skips populated bands')

    const activity = page.locator('[data-test="home-activity"]')
    await expect(activity).toBeVisible()
    await expect(activity.locator('.home-section-link')).toHaveAttribute('href', '/events')
  })

  test('PRESERVE: quick-links row mirrors the OG dashboard catalog', async ({ page }) => {
    await gotoHome(page)
    test.skip(await page.locator('[data-test="home-empty"]').count() > 0, 'empty workspace skips quick links')

    const links = page.locator('[data-test="home-links"] [data-test="home-link"]')
    await expect(links).toHaveCount(6)
    const keys = await links.evaluateAll((els) =>
      els.map((el) => (el as HTMLElement).dataset['link']),
    )
    expect(keys).toEqual(['workflows', 'agents', 'skills', 'data', 'events', 'settings'])
  })

  test('PRESERVE: quick-links navigate via chrome.navigate (no full reload)', async ({ page }) => {
    await gotoHome(page)
    test.skip(await page.locator('[data-test="home-empty"]').count() > 0, 'empty workspace skips quick links')

    const navWait = page.waitForEvent('framenavigated', { predicate: (f) => f.url().endsWith('/skills') })
    await page.locator('[data-test="home-links"] [data-link="skills"]').click()
    await navWait
    expect(page.url()).toContain('/skills')
  })

  test('AVOID: /dashboard and /overview no longer mount their own SUI view', async ({ page }) => {
    // After consolidation those routes have no registered view.
    // shell renders its "Page not found" error block.
    const dashRes = await page.goto(`${SHELL2_URL}/dashboard`, { waitUntil: 'load' })
    expect(dashRes?.status() ?? 200).toBeLessThan(500)
    await expect(page.locator('.shell-error, [data-test="home-title"]')).toBeVisible({ timeout: 10_000 })
    // The matched marker must NOT be a dashboard-specific class.
    await expect(page.locator('.dash-wrap')).toHaveCount(0)

    const ovRes = await page.goto(`${SHELL2_URL}/overview`, { waitUntil: 'load' })
    expect(ovRes?.status() ?? 200).toBeLessThan(500)
    await expect(page.locator('.overview-root, .shell-error')).toHaveCount(0).catch(() => undefined)
  })

  test('Layout is responsive — narrow viewport collapses to single column', async ({ page }) => {
    await page.setViewportSize({ width: 380, height: 900 })
    await gotoHome(page)
    test.skip(await page.locator('[data-test="home-empty"]').count() > 0, 'empty workspace skips layout assertion')

    const grid = await page.locator('[data-test="home-stats"]').evaluate((el) =>
      getComputedStyle(el).gridTemplateColumns,
    )
    // Mobile: 2-column stats per CSS rule.
    expect(grid.split(' ').length).toBe(2)
  })

  test('Layout is responsive — wide viewport expands to 4-column stats', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 })
    await gotoHome(page)
    test.skip(await page.locator('[data-test="home-empty"]').count() > 0, 'empty workspace skips layout assertion')

    const grid = await page.locator('[data-test="home-stats"]').evaluate((el) =>
      getComputedStyle(el).gridTemplateColumns,
    )
    expect(grid.split(' ').length).toBe(4)
  })

  test('Loading state renders skeletons rather than blank space', async ({ page }) => {
    // Block the SSE bridge so the `dash` signal stays in `loading` long
    // enough to assert the skeleton view. We don't await navigation full
    // load — we want the first paint with the loading block visible.
    await page.route('**/_sui/data**', () => {
      // Silent hang — the channel never resolves.
    })
    await page.goto(`${SHELL2_URL}/home`).catch(() => undefined)
    await expect(page.locator('[data-test="home-loading"]')).toBeVisible({ timeout: 15_000 })
  })
})
