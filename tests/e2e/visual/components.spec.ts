/**
 * Component baselines. One screenshot per (component, theme) rendered in
 * the synthesized harness (helpers/render.ts → gotoComponentHarness).
 *
 * The slot HTML is curated per component in views.json so each baseline
 * exercises a representative state (variants, tones, disabled, etc.)
 * rather than a default-everything render that drifts to ambiguity.
 */
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { test, expect } from '@playwright/test'
import { gotoComponentHarness, type ThemeId } from './helpers/render'

const REGISTRY = JSON.parse(
  readFileSync(path.join(__dirname, 'views.json'), 'utf8'),
) as { components: Array<{ tag: string; slot: string }> }

const THEMES: ThemeId[] = ['paper', 'ship-dark']

for (const { tag, slot } of REGISTRY.components) {
  for (const theme of THEMES) {
    test(`component: ${tag} [${theme}]`, async ({ page }) => {
      await gotoComponentHarness(page, slot, theme)
      const harness = page.locator('#harness')
      await expect(harness).toHaveScreenshot(`components/${tag}.${theme}.png`)
    })
  }
}
