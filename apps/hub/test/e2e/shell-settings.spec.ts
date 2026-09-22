/**
 * shell Settings view.
 *
 * `views/settings/page.sui` renders a `<data live>` settings document as
 * a column of `<sui-card>` section forms. This spec ports the load-bearing
 * checks from `settings-probe.mjs`: the sections render, every form input
 * is labelled, and the theme choice persists across a reload (the theme
 * picker lives in the top bar and owns `data-theme` on `<html>`).
 *
 * Runs against shell's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

test.describe('shell settings view', () => {
  skipUnlessShell2()

  test('renders settings section cards once the live doc resolves', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))

    await page.goto(`${SHELL2_URL}/settings`, { waitUntil: 'load' })

    // Wait for the live settings signal to resolve to either the
    // section cards or an error state.
    await page.waitForFunction(
      () => {
        const wrap = document.querySelector('.settings-wrap')
        if (!wrap) return false
        return Boolean(
          wrap.querySelector('.s-sections') || wrap.querySelector('sui-error-state'),
        )
      },
      null,
      { timeout: 15_000 },
    )

    const sectionCards = await page.locator('.s-sections > sui-card').count()
    expect(sectionCards).toBeGreaterThanOrEqual(4)

    expect(errors, errors.join('\n')).toHaveLength(0)
  })

  test('every settings form input carries a label or aria-label', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/settings`, { waitUntil: 'load' })
    await page.waitForFunction(
      () => Boolean(document.querySelector('.settings-wrap .s-sections')),
      null,
      { timeout: 15_000 },
    )

    const unlabeled = await page.evaluate(() => {
      const out: string[] = []
      for (const form of document.querySelectorAll('.settings-wrap form')) {
        for (const inp of form.querySelectorAll('sail-input, sui-select')) {
          const hasAria =
            inp.hasAttribute('aria-label') || inp.hasAttribute('aria-labelledby')
          if (!hasAria) out.push(inp.tagName)
        }
      }
      return out
    })
    expect(unlabeled, `unlabeled inputs: ${unlabeled.join(', ')}`).toHaveLength(0)
  })

  test('theme choice persists across a reload (localStorage → data-theme)', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/settings`, { waitUntil: 'load' })
    await page.waitForFunction(
      () => Boolean(document.querySelector('.settings-wrap')),
      null,
      { timeout: 15_000 },
    )

    const before = await page.evaluate(() => document.documentElement.dataset.theme)
    const alt = before === 'ship-light' ? 'ship-dark' : 'ship-light'
    await page.evaluate((id) => localStorage.setItem('ship-site-theme', id), alt)

    await page.reload({ waitUntil: 'load' })
    await expect
      .poll(() => page.evaluate(() => document.documentElement.dataset.theme))
      .toBe(alt)

    // Restore so subsequent specs start from a clean theme.
    await page.evaluate(
      (id) => localStorage.setItem('ship-site-theme', id),
      before ?? 'ship-light',
    )
  })
})
