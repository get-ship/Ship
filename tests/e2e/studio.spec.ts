/**
 * ship-ui-studio for-iteration regression — Phase 2 acceptance.
 *
 * Pre-Phase 2: the studio's `{#for item of filtered() key={item.id}}`
 * loop rendered tiles whose inner `{item.id}`, `{item.kind}`, and
 * `{item.description}` text-bindings came back empty because the SSR
 * walker emitted them as text-comment markers and the
 * fragment-fetch swap couldn't fill them — the daemon couldn't
 * evaluate `item.id` against the per-iteration scope.
 *
 * Phase 2 moves control flow to client-side. The compiler's
 * `{#for}` emit produces a per-iteration template factory that
 * captures `item` as a closure variable; the inline `bindText` call
 * inside the iteration evaluates against the right scope. Tiles
 * populate.
 *
 * This spec is the ratchet: if any future regression breaks
 * per-iteration scope propagation, this test fails.
 */
import { test, expect } from '@playwright/test'
import { waitForAttach } from './_helpers'

test('studio: tiles render with item.id text content', async ({ page }) => {
  await page.goto('/sui/ship-ui-studio')
  await waitForAttach(page)

  const tiles = page.locator('[data-test="tile"]')
  await expect(tiles.first()).toBeVisible({ timeout: 20_000 })

  // At least a handful of tiles must be present (one per component
  // catalog entry — should be 100+ in the live workspace).
  const count = await tiles.count()
  expect(count).toBeGreaterThan(10)

  // Every visible tile must have non-empty text in its `<code>`
  // (rendered from `{item.id}`). Pre-Phase-2 these were blank.
  const ids = await tiles.locator('code.tile-name').allTextContents()
  expect(ids.length).toBe(count)
  const empty = ids.filter((s) => s.trim() === '')
  expect(empty.length).toBe(0)

  // Sanity check: the badge text from `{item.kind}` is also populated
  // for at least the first tile.
  const firstBadge = tiles.first().locator('ship-badge')
  await expect(firstBadge).not.toHaveText(/^\s*$/)
})

test('studio: render-fragment endpoint returns 404', async ({ request }) => {
  // Phase 2 deletes the POST /_sui/render-fragment/{view} endpoint.
  // Confirm the route is gone.
  const res = await request.post('/_sui/render-fragment/git', {
    data: {},
    failOnStatusCode: false,
  })
  expect(res.status()).toBe(404)
})
