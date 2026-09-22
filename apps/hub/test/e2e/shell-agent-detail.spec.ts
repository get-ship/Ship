/**
 * shell agent-detail route — `/agents/:id`.
 *
 * The detail view (`views/agents/[id]/page.sui`) reads `route.params.id()`
 * via the shared route signal the shell wires into mount opts. An earlier
 * regression had this resolve to an empty string (dual `@ship/sail-runtime`
 * module instances); main.ts now pre-registers the `/agents/:id` pattern
 * with `defineRoutes` and passes the shell's `routeSignals` into mount, so
 * a direct hit on `/agents/<id>` seeds `params.id` correctly.
 *
 * This spec locks that in: a known agent renders its detail panel, and an
 * unknown id renders the not-found state (NOT a blank "id ''" panel).
 *
 * Ported from `agent-detail-probe.mjs`. Runs against shell's vite dev
 * server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

test.describe('shell agent detail route', () => {
  skipUnlessShell2()

  test('/agents/commander renders the detail panel for a real agent', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))

    await page.goto(`${SHELL2_URL}/agents/commander`, { waitUntil: 'load' })

    // The detail panel only appears once route.params.id() resolved to a
    // non-empty id AND the agent lookup succeeded.
    const detail = page.locator('[data-test="agent-detail"]')
    await expect(detail).toHaveCount(1, { timeout: 15_000 })

    // Not-found state must NOT be present for a real agent.
    await expect(page.locator('[data-test="agent-not-found"]')).toHaveCount(0)

    // The agent name renders — proves the id resolved and the lookup hit.
    await expect(page.locator('[data-test="agent-name"]').first()).toHaveText(/commander/i)

    expect(errors, errors.join('\n')).toHaveLength(0)
  })

  test('/agents/<unknown-id> renders the not-found state, not a blank panel', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/agents/zzz-nonexistent-agent`, { waitUntil: 'load' })

    const notFound = page.locator('[data-test="agent-not-found"]')
    await expect(notFound).toHaveCount(1, { timeout: 15_000 })
    await expect(page.locator('[data-test="agent-detail"]')).toHaveCount(0)

    // The id must appear in the message — confirms params.id() resolved
    // to the real (if unknown) segment, not an empty string.
    await expect(notFound).toContainText('zzz-nonexistent-agent')
  })
})
