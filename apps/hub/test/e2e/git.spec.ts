/**
 * Git view e2e tests (tier-2, controller + live data, complex).
 *
 * The git view (`views/git/view.sui`) subscribes to `ship.git.overview()`.
 * On SSR the overview is `{kind: 'loading'}`. After attach, the SSE channel
 * fires and the control-flow swap reveals the full git UI (header, tabs,
 * changes/history/branches panels).
 *
 * Interactive coverage: tab switching is driven by `<state name="tab">` +
 * `on:click` signal mutations — a tier-2 interaction that exercises the
 * full attach → signal → DOM update pipeline.
 *
 * Failures here indicate the runtime swap or signal update is broken.
 */
import { test, expect } from '@playwright/test'
import { screenshot, waitForAttach } from './_helpers'

// ── 1. SSR structure ─────────────────────────────────────────────────────────

test('SSR: git page returns HTML 200', async ({ request }) => {
  const res = await request.get('/sui/git')
  expect(res.status()).toBe(200)
  expect(res.headers()['content-type']).toMatch(/text\/html/)
})

test('SSR: loading branch is rendered server-side', async ({ request }) => {
  // `<while-loading bind="overview">` renders with display:contents since
  // overview starts as {kind: 'loading'}.
  const body = await (await request.get('/sui/git')).text()
  expect(body).toContain('data-test="loading"')
  expect(body).toContain('Loading repo overview')
})

test('SSR: git view wrapper present', async ({ request }) => {
  const body = await (await request.get('/sui/git')).text()
  expect(body).toContain('class="ship-view git-view"')
  expect(body).toContain('data-view="git"')
  expect(body).toContain('id="sui-root"')
})

// ── 2. Attach + SSE resolution ────────────────────────────────────────────────

test('attach: page loads without JS errors', async ({ page }) => {
  const jsErrors: string[] = []
  page.on('pageerror', (err) => jsErrors.push(err.message))

  await page.goto('/sui/git')
  await waitForAttach(page)

  const ours = jsErrors.filter(
    (e) => !e.includes('ResizeObserver') && !e.includes('Non-Error promise rejection'),
  )
  expect(ours).toHaveLength(0)
})

test('attach: SSE resolves loading → git header visible', async ({ page }) => {
  // This test FAILS if the runtime does not complete the control-flow swap
  // from loading to the ok state. That failure is the regression to fix.
  await page.goto('/sui/git')
  await waitForAttach(page)

  // The git header only renders when overview.kind === 'ok'
  await expect(page.locator('.git-head')).toBeVisible({ timeout: 20_000 })
  await screenshot(page, 'git-resolved')
})

test('attach: tab switching — Changes → History → Branches', async ({ page }) => {
  await page.goto('/sui/git')
  await waitForAttach(page)

  // Wait for the data to resolve first (tabs only appear in the ok branch)
  await page.locator('.git-head').waitFor({ timeout: 20_000 })

  // The tab row must be present
  await expect(page.locator('.git-tabs-row')).toBeVisible()

  // Switch to History tab — on:click fires tab.set('history')
  await page.locator('.git-tabs-row [name="history"]').click()

  // The history panel should become visible and changes panel hidden
  await expect(page.locator('.git-history')).toBeVisible({ timeout: 5_000 })

  // Switch to Branches tab
  await page.locator('.git-tabs-row [name="branches"]').click()
  await expect(page.locator('.git-branches')).toBeVisible({ timeout: 5_000 })

  // Return to Changes tab
  await page.locator('.git-tabs-row [name="changes"]').click()
  await expect(page.locator('.git-body .git-clean, .git-body .git-changes')).toBeVisible({ timeout: 5_000 })

  await screenshot(page, 'git-tab-changes')
})
