/**
 * shell Artifacts view.
 *
 * `views/artifacts/page.sui` renders a header with an upload button and a
 * primary surface that is one of: a table, an empty state, an error
 * state, or a loading state. The upload dialog opens from the header and
 * closes on Escape. This spec ports the load-bearing checks from
 * `artifacts-probe.mjs`.
 *
 * Runs against shell's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

test.describe('shell artifacts view', () => {
  skipUnlessShell2()

  test('renders the header and a primary surface (table / empty / error)', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))

    await page.goto(`${SHELL2_URL}/artifacts`, { waitUntil: 'load' })

    await expect(page.locator('.ship-view-artifacts')).toBeVisible({ timeout: 15_000 })
    await expect(page.locator('.art-title')).toContainText('Artifacts')
    await expect(page.locator('[data-test="upload-button"]')).toBeVisible()

    // Exactly one of the live-data branches must be on screen.
    await page.waitForFunction(
      () =>
        Boolean(
          document.querySelector('[data-test="table"]') ||
          document.querySelector('[data-test="empty"]') ||
          document.querySelector('[data-test="error"]'),
        ),
      null,
      { timeout: 15_000 },
    )

    expect(errors.filter((e) => !/net::ERR_/i.test(e)), errors.join('\n')).toHaveLength(0)
  })

  test('upload dialog opens from the header and closes on Escape', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/artifacts`, { waitUntil: 'load' })
    await expect(page.locator('[data-test="upload-button"]')).toBeVisible({ timeout: 15_000 })

    // Drive the click through the element's own handler: the closed
    // detail-drawer renders a 0×0 portal node that Playwright's
    // actionability check treats as an interception layer, and a
    // `force` click bypasses the actionability check without
    // dispatching through `<sui-button>`'s shadow-DOM event path.
    await page.locator('[data-test="upload-button"]').evaluate((el) => {
      (el as HTMLElement).click()
    })

    // `<sui-dialog>` reflects open state via `data-state`, not a bare
    // `open` attribute.
    const dialog = page.locator('[data-test="upload-dialog"]')
    await expect(dialog).toHaveAttribute('data-state', 'open', { timeout: 5_000 })
    await expect(page.locator('[data-test="upload-files"]')).toBeAttached()
    await expect(page.locator('[data-test="upload-submit"]')).toBeAttached()

    await page.keyboard.press('Escape')
    await expect(dialog).toHaveAttribute('data-state', 'closed')
  })
})
