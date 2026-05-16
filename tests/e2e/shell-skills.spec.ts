/**
 * shell Skills view.
 *
 * `views/skills/page.sui` renders a list of `.se-skill` entries; picking
 * one populates a detail pane (`.sd-name`, description, an invoke button,
 * and tabbed content/metadata/runs). This spec ports the load-bearing
 * checks from `skills-detail-probe.mjs` without hard-coding the skill
 * count (the probe assumed exactly five — the live workspace has many
 * more).
 *
 * Runs against shell's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

test.describe('shell skills view', () => {
  skipUnlessShell2()

  test('skills list renders and selecting one populates the detail pane', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))

    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })

    // The list resolves from a live data source — wait for at least one
    // skill entry to appear.
    const skills = page.locator('.se-skill')
    await expect(skills.first()).toBeVisible({ timeout: 15_000 })
    expect(await skills.count()).toBeGreaterThan(0)

    // Select the first skill; the detail pane should populate.
    await skills.first().click()

    await expect(page.locator('.sd-name')).not.toBeEmpty({ timeout: 10_000 })
    await expect(page.locator('.sd-invoke-btn')).toBeVisible()

    expect(errors, errors.join('\n')).toHaveLength(0)
  })

  test('skill detail metadata tab exposes id / kind / source rows', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })
    await expect(page.locator('.se-skill').first()).toBeVisible({ timeout: 15_000 })
    await page.locator('.se-skill').first().click()
    await expect(page.locator('.sd-name')).not.toBeEmpty({ timeout: 10_000 })

    await page.locator('sui-tab[name="metadata"]').click()

    const dts = await page
      .locator('.sd-meta-list dt')
      .evaluateAll((els) => els.map((d) => d.textContent?.trim() ?? ''))
    expect(dts).toContain('id')
    expect(dts).toContain('kind')
    expect(dts).toContain('source')
  })
})
