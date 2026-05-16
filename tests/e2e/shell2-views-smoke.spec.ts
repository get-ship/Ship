/**
 * shell2 view smoke suite — every view in the left rail (plus the
 * legacy direct-route views) must mount in a real browser without
 * throwing an uncaught page error, and must render real DOM into
 * `main#ship-content`.
 *
 * This is the cheap, broad safety net: it does not assert view-specific
 * behaviour (that lives in the per-view specs), only that the shell can
 * route to the view, fetch its payload, and complete `mount()` cleanly.
 *
 * Ported from the `inventory-probe.mjs` / `probe.mjs` ad-hoc scripts,
 * which walked the same view list but only logged a report.
 *
 * Runs against shell2's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell2'

// Every view shell2 can route to. The first seven are the rail items;
// the rest are direct-route views without a rail entry.
const VIEWS = [
  'home', 'agents', 'skills', 'collections', 'artifacts', 'workspaces',
  'settings', 'ship-ui-studio', 'sessions',
  'chat', 'mesh', 'events', 'git', 'canvas', 'specs', 'pipelines',
  'pipelines-board', 'theme-studio', 'setup',
] as const

// Noise from third-party deps / browser internals that is not a shell
// or view defect. Mirrors the filter in tests/e2e/agents.spec.ts.
const IGNORED = /ResizeObserver|Non-Error promise rejection/

test.describe('shell2 view smoke', () => {
  skipUnlessShell2()

  for (const view of VIEWS) {
    test(`/${view} mounts without a page error`, async ({ page }) => {
      const errors: string[] = []
      page.on('pageerror', (err) => errors.push(err.message))

      await page.goto(`${SHELL2_URL}/${view}`, { waitUntil: 'load' })

      // mount() writes real DOM into the content slot. Wait for it to
      // gain children — a loading skeleton counts; an empty slot does
      // not. Errors that throw during mount surface via `pageerror`.
      await page.waitForFunction(
        () => {
          const slot = document.querySelector('main#ship-content')
          return Boolean(slot && slot.children.length > 0)
        },
        null,
        { timeout: 15_000 },
      )

      const slotChildren = await page.locator('main#ship-content > *').count()
      expect(slotChildren).toBeGreaterThan(0)

      const real = errors.filter((e) => !IGNORED.test(e))
      expect(real, `page errors on /${view}:\n${real.join('\n')}`).toHaveLength(0)
    })
  }
})
