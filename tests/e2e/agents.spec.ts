/**
 * Agents view e2e tests (tier-2, controller + live data).
 *
 * The agents view (`views/agents/view.sui`) uses a `<data live>` subscription
 * to `ship.agents.summaries()`. Server-side it renders with `agents.kind ===
 * 'loading'`; the client runtime opens an SSE channel to `/_sui/data`, the
 * daemon resolves the expression, and a boundary swap replaces the loading
 * branch with either the agents grid or the empty state.
 *
 * If the SSE → swap chain is broken this suite surfaces it:
 *   - SSR tests are unconditional (must pass).
 *   - Attach tests will fail if the runtime does not complete the swap,
 *     which is exactly the signal the user wants.
 */
import { test, expect } from '@playwright/test'
import { screenshot, waitForAttach } from './_helpers'

// ── 1. SSR structure ─────────────────────────────────────────────────────────

test('SSR: agents page returns HTML 200', async ({ request }) => {
  const res = await request.get('/sui/agents')
  expect(res.status()).toBe(200)
  expect(res.headers()['content-type']).toMatch(/text\/html/)
})

test('SSR: loading branch is rendered server-side', async ({ request }) => {
  // The `<while-loading bind="agents">` compiles to a `<sui-if>` boundary.
  // When agents.kind === 'loading' the walker emits it with display:contents.
  const body = await (await request.get('/sui/agents')).text()
  expect(body).toContain('<sui-if')
  expect(body).toContain('data-test="loading"')
  expect(body).toContain('Loading agents')
})

test('SSR: __set_init__ seeds agents as {kind: "loading"}', async ({ request }) => {
  const body = await (await request.get('/sui/agents')).text()
  expect(body).toContain('"agents":{"kind":"loading"}')
})

test('SSR: page wrapper has correct view name', async ({ request }) => {
  const body = await (await request.get('/sui/agents')).text()
  expect(body).toContain('data-view="agents"')
  expect(body).toContain('id="sui-root"')
})

// ── 2. Attach + SSE resolution ────────────────────────────────────────────────

test('attach: page loads and attach() completes without JS errors', async ({ page }) => {
  const jsErrors: string[] = []
  page.on('pageerror', (err) => jsErrors.push(err.message))

  await page.goto('/sui/agents')
  await waitForAttach(page)

  // Filter noise from third-party deps; only care about errors in our code.
  const ours = jsErrors.filter(
    (e) => !e.includes('ResizeObserver') && !e.includes('Non-Error promise rejection'),
  )
  expect(ours).toHaveLength(0)
})

test('attach: SSE resolves loading → agents grid or empty state', async ({ page }) => {
  // This test will FAIL if the SSE → boundary-swap chain is broken.
  // Failure here means the runtime is not completing the control-flow swap
  // from `loading` to `ok` — that is the bug to fix.
  await page.goto('/sui/agents')
  await waitForAttach(page)

  // After the SSE fires and the swap runs, one of these must become visible.
  // The loading state disappears when agents.kind changes from 'loading' to 'ok'.
  const resolved = page.locator('[data-test="grid"], [data-test="empty"]')
  await expect(resolved).toBeVisible({ timeout: 15_000 })

  await screenshot(page, 'agents-resolved')
})

test('attach: resolved state is consistent (grid xor empty)', async ({ page }) => {
  await page.goto('/sui/agents')
  await waitForAttach(page)
  await page.locator('[data-test="grid"], [data-test="empty"]').waitFor({ timeout: 15_000 })

  // Exactly one branch should be visible at a time
  const gridVisible  = await page.locator('[data-test="grid"]').isVisible()
  const emptyVisible = await page.locator('[data-test="empty"]').isVisible()
  expect(gridVisible || emptyVisible).toBe(true)
  expect(gridVisible && emptyVisible).toBe(false)
})
