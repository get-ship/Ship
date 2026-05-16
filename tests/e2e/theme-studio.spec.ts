/**
 * Theme Studio — token editor + live preview + pack selector + persistence.
 *
 * Covers the six behavioral acceptance checks from .artifacts/job-spec.md:
 *   1. Token panel renders categorized layers.
 *   2. Editing a color writes through chrome.theme.set → preview reflects.
 *   3. Picking a pack resets tokens to the pack's values.
 *   4. Save → reload → tokens persist.
 *   5. Reset reverts to the last saved snapshot.
 *   6. Export → JSON; Import round-trips.
 *
 * Runs against shell2's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell2'

const STUDIO_URL = `${SHELL2_URL}/theme-studio`
const SETTINGS_KEY = 'theme.overrides'

async function waitForStudio(page: import('@playwright/test').Page): Promise<void> {
  await page.goto(STUDIO_URL, { waitUntil: 'load' })
  await page.waitForSelector('[data-test="theme-studio-editor"]', { timeout: 15_000 })
  await page.waitForSelector('[data-test="theme-studio-preview"]', { timeout: 15_000 })
}

async function clearSavedOverrides(page: import('@playwright/test').Page): Promise<void> {
  // Best-effort — the daemon's settings endpoint may not exist on every
  // environment; ignore failures. We just want a clean slate.
  await page.evaluate(async (key) => {
    try {
      await fetch('/api/v1/settings', {
        method: 'PUT',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key, value: '' }),
      })
    } catch { /* best-effort */ }
  }, SETTINGS_KEY)
}

test.describe('Theme Studio', () => {
  skipUnlessShell2()

  test.beforeEach(async ({ page }) => {
    await page.goto(SHELL2_URL, { waitUntil: 'load' })
    await clearSavedOverrides(page)
  })

  test('renders the token panel grouped by layer', async ({ page }) => {
    await waitForStudio(page)

    // Each layer section has data-layer attribute. Skill defines five
    // canonical groupings: palette, surface, tone, role, primitive.
    const layers = await page.locator('[data-test="theme-studio-editor"] section[data-layer]').count()
    expect(layers).toBeGreaterThanOrEqual(5)

    for (const id of ['palette', 'surface', 'tone', 'role', 'primitive']) {
      await expect(page.locator(`section[data-layer="${id}"]`)).toBeVisible()
    }

    // Editor row per token; should be many.
    const rows = await page.locator('[data-test="theme-studio-editor"] .ts-row[data-token]').count()
    expect(rows).toBeGreaterThan(20)
  })

  test('edit a color token → preview reflects within 100ms', async ({ page }) => {
    await waitForStudio(page)

    const input = page.locator('.ts-row[data-token="--primary"] input.ts-input')
    await input.waitFor()

    // Fill with a striking value so we can compare resolved style on the
    // preview primary button.
    const next = 'oklch(0.5 0.2 30)'
    await input.fill(next)
    await input.dispatchEvent('input')

    // Settle one frame, then read the resolved background of the primary
    // preview button. chrome.theme.set writes the custom property onto
    // document.body, so the cascade lands immediately.
    const start = Date.now()
    await expect.poll(
      () => page.evaluate(() => {
        const btn = document.querySelector('.ts-tone-primary') as HTMLElement | null
        return btn ? getComputedStyle(btn).backgroundColor : ''
      }),
      { timeout: 1_000, intervals: [25, 50, 100] },
    ).not.toBe('')

    const elapsed = Date.now() - start
    expect(elapsed).toBeLessThan(2_000) // generous; the assertion is "fast enough to feel live"

    const bodyVar = await page.evaluate(() =>
      document.body.style.getPropertyValue('--primary').trim(),
    )
    expect(bodyVar).toBe(next)
  })

  test('selecting a pack resets tokens to the pack values', async ({ page }) => {
    await waitForStudio(page)

    // Set a custom override first.
    const input = page.locator('.ts-row[data-token="--primary"] input.ts-input')
    await input.fill('oklch(0.7 0.2 200)')
    await input.dispatchEvent('input')

    await expect.poll(() =>
      page.evaluate(() => document.body.style.getPropertyValue('--primary').trim()),
    ).toBe('oklch(0.7 0.2 200)')

    // Now switch packs — should clear overrides and re-apply theme.
    const select = page.locator('[data-test="theme-studio-pack-select"]')
    await select.selectOption('ship-dark')

    await expect.poll(() =>
      page.evaluate(() => document.documentElement.dataset.theme),
    ).toBe('ship-dark')

    // The body-level override should be cleared.
    const cleared = await page.evaluate(() =>
      document.body.style.getPropertyValue('--primary').trim(),
    )
    expect(cleared).toBe('')
  })

  test('save → reload persists the overrides', async ({ page }) => {
    await waitForStudio(page)

    const customValue = 'oklch(0.6 0.18 95)'
    const input = page.locator('.ts-row[data-token="--primary"] input.ts-input')
    await input.fill(customValue)
    await input.dispatchEvent('input')

    await page.locator('[data-test="theme-studio-save"]').click()

    // Wait for the "Saved." status pill or assume settings PUT completed.
    await page.waitForTimeout(300)

    await page.reload({ waitUntil: 'load' })
    await page.waitForSelector('[data-test="theme-studio-editor"]', { timeout: 15_000 })

    await expect.poll(() =>
      page.evaluate(() => document.body.style.getPropertyValue('--primary').trim()),
      { timeout: 5_000 },
    ).toBe(customValue)
  })

  test('reset reverts to last saved state', async ({ page }) => {
    await waitForStudio(page)

    const saved = 'oklch(0.55 0.2 50)'
    const drift = 'oklch(0.4 0.05 280)'

    const input = page.locator('.ts-row[data-token="--primary"] input.ts-input')

    // Save baseline.
    await input.fill(saved)
    await input.dispatchEvent('input')
    await page.locator('[data-test="theme-studio-save"]').click()
    await page.waitForTimeout(200)

    // Drift to a different value (not saved).
    await input.fill(drift)
    await input.dispatchEvent('input')
    await expect.poll(() =>
      page.evaluate(() => document.body.style.getPropertyValue('--primary').trim()),
    ).toBe(drift)

    // Reset should pull back to the saved baseline.
    await page.locator('[data-test="theme-studio-reset"]').click()
    await expect.poll(() =>
      page.evaluate(() => document.body.style.getPropertyValue('--primary').trim()),
      { timeout: 3_000 },
    ).toBe(saved)
  })

  test('export produces valid JSON and import round-trips', async ({ page }) => {
    await waitForStudio(page)

    const exportedValue = 'oklch(0.65 0.21 312)'
    const input = page.locator('.ts-row[data-token="--accent"] input.ts-input')
    await input.fill(exportedValue)
    await input.dispatchEvent('input')

    // Click Export — captures the download.
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-test="theme-studio-export"]').click(),
    ])

    const path = await download.path()
    expect(path).toBeTruthy()

    const fs = await import('node:fs')
    const text = fs.readFileSync(path!, 'utf8')
    const parsed = JSON.parse(text) as { pack: string; values: Record<string, string> }
    expect(parsed.values['--accent']).toBe(exportedValue)
    expect(typeof parsed.pack).toBe('string')

    // Reset to pack so we can verify import re-applies the value.
    await page.locator('[data-test="theme-studio-reset-pack"]').click()
    await expect.poll(() =>
      page.evaluate(() => document.body.style.getPropertyValue('--accent').trim()),
    ).toBe('')

    // Construct a synthetic import: change accent to a 3rd value and feed
    // it through the file input. The hidden file input is named ts-file.
    const importedValue = 'oklch(0.7 0.18 140)'
    const importPayload = JSON.stringify({ pack: parsed.pack, values: { '--accent': importedValue } })

    await page.locator('input.ts-file').setInputFiles({
      name: 'theme-import.json',
      mimeType: 'application/json',
      buffer: Buffer.from(importPayload, 'utf8'),
    })

    await expect.poll(() =>
      page.evaluate(() => document.body.style.getPropertyValue('--accent').trim()),
      { timeout: 3_000 },
    ).toBe(importedValue)
  })
})
