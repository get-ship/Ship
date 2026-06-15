/**
 * Top-level view behavior smoke. P1.1 rewrite: dropped the
 * data-driven `toHaveScreenshot()` pixel diff (the 32-view × 2-theme
 * baseline matrix that asserted only visual fidelity). The pixel
 * baselines in `./baselines/` are kept as a manual artifact; this
 * spec is now the behavior gate:
 *
 *   1. Mount the view via the daemon's `/_deck/views/<name>` route.
 *   2. Wait for the SUI runtime attach (#sui-root gets a child).
 *   3. Assert the page has no console errors and no thrown page
 *      errors (the two surfaces a regression would surface on).
 *   4. Assert the page body has SOME content (the runtime replaced
 *      the loading shell; we don't care what the content is).
 *
 * Adding a view requires editing `views.json`; the test is a
 * content gate, not a visual gate.
 */
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { test, expect, type ConsoleMessage } from '@playwright/test'
import { gotoView, type ThemeId } from './helpers/render'

const REGISTRY = JSON.parse(
  readFileSync(path.join(__dirname, 'views.json'), 'utf8'),
) as { views: string[] }

const THEMES: ThemeId[] = ['paper', 'ship-dark']

for (const view of REGISTRY.views) {
  for (const theme of THEMES) {
    test(`view: ${view} [${theme}] attaches without console errors`, async ({ page }) => {
      const consoleErrors: string[] = []
      const pageErrors: Error[] = []
      page.on('console', (m: ConsoleMessage) => {
        if (m.type() === 'error') consoleErrors.push(m.text())
      })
      page.on('pageerror', (e) => pageErrors.push(e))

      await gotoView(page, view, theme)

      // Attach signal: the runtime has populated #sui-root with content.
      // gotoView already waits for this; the assertion is the gate.
      const root = page.locator('#sui-root')
      await expect(root).not.toBeEmpty()

      // The page didn't throw a hard error during mount.
      expect(
        pageErrors,
        `page error during ${view}/${theme}: ${pageErrors.map((e) => e.message).join('\n')}`,
      ).toHaveLength(0)

      // The SUI runtime reports errors via `console.error`. (Mock-ship
      // a controller that throws will surface here; a broken template
      // compile will too.)
      expect(
        consoleErrors,
        `console errors during ${view}/${theme}: ${consoleErrors.join('\n')}`,
      ).toHaveLength(0)
    })
  }
}
