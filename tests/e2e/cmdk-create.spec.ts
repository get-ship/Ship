/**
 * Cmdk create-mode → POST /api/v1/create/skill.
 *
 * Opens the command bar in create mode, picks the Skill tile, fills name +
 * shape, submits, then verifies the daemon scaffolded the SKILL.md by
 * reading it back via GET /api/v1/skills/<id>. Cleans up the skill after,
 * since this writes into the live ship dir of whatever daemon shell2 is
 * pointed at.
 *
 * Skipped when shell2's vite dev server isn't reachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell2'
import { getDaemonUrl } from './_helpers'

test.describe('cmdk create-mode → skill scaffold', () => {
  skipUnlessShell2()

  test('Skill tile + submit creates a SKILL.md the daemon can read back', async ({ page }) => {
    const daemon = getDaemonUrl()
    // Unique per run so we don't collide with a leftover from a prior
    // failed test or with a real user skill.
    const skillId = `e2e-cmdk-${Date.now().toString(36)}`

    await page.goto(`${SHELL2_URL}/`, { waitUntil: 'load' })

    // Open the bar in create mode through the same imperative surface
    // the rail "+" button uses (see apps/shell2/src/main.ts:646).
    await page.waitForFunction(() => !!document.querySelector('sui-command-bar'))
    await page.evaluate(() => {
      const bar = document.querySelector('sui-command-bar') as (HTMLElement & { show: (m: string) => void }) | null
      bar?.show('create')
    })

    // Pick the Skill surface tile inside the palette's shadow root.
    const skillBtn = page.locator('sui-command-bar')
      .locator('sui-command-palette')
      .locator('button.cb-surface-btn[data-surface="skill"]')
    await expect(skillBtn).toBeVisible({ timeout: 10_000 })
    await expect(skillBtn).toBeEnabled()
    await skillBtn.click()
    await expect(skillBtn).toHaveAttribute('aria-pressed', 'true')

    const palette = page.locator('sui-command-bar').locator('sui-command-palette')
    await palette.locator('input.cb-create-name').fill(skillId)
    await palette.locator('select.cb-create-shape').selectOption('task')

    const submit = palette.locator('button.cb-create-submit')
    await expect(submit).toBeEnabled()
    await submit.click()

    try {
      // Poll the skill detail endpoint until the scaffold lands. The
      // daemon writes synchronously then emits an event in tokio::spawn,
      // so this resolves on the first try in practice.
      await expect
        .poll(
          async () => {
            const res = await fetch(`${daemon}/api/v1/skills/${skillId}`)
            return res.status
          },
          { timeout: 10_000, intervals: [200, 400, 800] },
        )
        .toBe(200)

      const detail = await fetch(`${daemon}/api/v1/skills/${skillId}`).then((r) => r.json())
      expect(detail.ok).toBe(true)
      expect(detail.skill?.id).toBe(skillId)
      expect(String(detail.skill?.frontmatter?.name ?? '')).toBe(skillId)
    } finally {
      // Best-effort cleanup so a repeat run starts clean.
      await fetch(`${daemon}/api/v1/skills/${skillId}`, { method: 'DELETE' }).catch(() => {})
    }
  })

  test('non-launch surface tiles (pipeline, trigger) render disabled with a post-launch hint', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`, { waitUntil: 'load' })
    await page.waitForFunction(() => !!document.querySelector('sui-command-bar'))
    await page.evaluate(() => {
      const bar = document.querySelector('sui-command-bar') as (HTMLElement & { show: (m: string) => void }) | null
      bar?.show('create')
    })

    const palette = page.locator('sui-command-bar').locator('sui-command-palette')
    for (const id of ['pipeline', 'trigger']) {
      const tile = palette.locator(`button.cb-surface-btn[data-surface="${id}"]`)
      await expect(tile).toBeVisible({ timeout: 10_000 })
      await expect(tile).toBeDisabled()
      await expect(tile).toContainText('post-launch')
    }
    for (const id of ['view', 'skill', 'agent', 'theme']) {
      const tile = palette.locator(`button.cb-surface-btn[data-surface="${id}"]`)
      await expect(tile).toBeEnabled()
    }
  })
})
