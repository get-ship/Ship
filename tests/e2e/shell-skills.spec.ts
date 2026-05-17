/**
 * shell Skills view.
 *
 * `views/skills/page.sui` is a unified IDE: explorer rail on the left,
 * editor + info panel on the right. Selecting an explorer row routes to
 * `/skills/<id>` without remounting the shell (SAME_SHELL_VIEWS in
 * apps/shell/src/main.ts), so the editor + collapsible info panel
 * appear in place.
 *
 * Runs against shell's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

test.describe('shell skills view', () => {
  skipUnlessShell2()

  test('skills list renders and selecting one populates the editor pane', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))

    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })

    const skills = page.locator('.se-skill')
    await expect(skills.first()).toBeVisible({ timeout: 15_000 })
    expect(await skills.count()).toBeGreaterThan(0)

    await skills.first().click()

    await expect(page.locator('.skills-info')).toBeVisible({ timeout: 10_000 })
    await expect(page.locator('.si-name')).not.toBeEmpty()
    await expect(page.locator('[data-testid="skill-editor"]')).toBeVisible()

    expect(errors, errors.join('\n')).toHaveLength(0)
  })

  test('skill info panel exposes kind + source rows and an editable description', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })
    await expect(page.locator('.se-skill').first()).toBeVisible({ timeout: 15_000 })
    await page.locator('.se-skill').first().click()
    await expect(page.locator('.skills-info')).toBeVisible({ timeout: 10_000 })

    // Overview section is open by default.
    await expect(page.locator('#si-desc')).toBeVisible()
    await expect(page.locator('#si-tags')).toBeVisible()

    const rowKeys = await page.locator('.si-row-key').evaluateAll(
      (els) => els.map((el) => el.textContent?.trim() ?? ''),
    )
    expect(rowKeys).toContain('kind')
    expect(rowKeys).toContain('source')
  })

  test('navigating between skill rows does not full-reload the document', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })
    await expect(page.locator('.se-skill').first()).toBeVisible({ timeout: 15_000 })

    // Stamp the document; if the shell remounts the whole view, our
    // stamped attribute on <html> disappears.
    await page.evaluate(() => document.documentElement.setAttribute('data-spa-stamp', '1'))

    const rows = page.locator('.se-skill')
    const count = await rows.count()
    if (count < 2) test.skip(true, 'fewer than two skills available')

    await rows.nth(0).click()
    await expect(page.locator('.skills-info')).toBeVisible({ timeout: 10_000 })
    await rows.nth(1).click()
    await expect(page.locator('.skills-info')).toBeVisible()

    const stamp = await page.evaluate(() => document.documentElement.getAttribute('data-spa-stamp'))
    expect(stamp).toBe('1')
  })
})
