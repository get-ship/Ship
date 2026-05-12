/**
 * Top-level view baselines. One screenshot per (view, theme).
 *
 * The view list lives in views.json so adding/removing a view requires
 * editing one file rather than this spec. `ship-ui-studio` is omitted
 * intentionally — that worktree is in flight, baselining a moving target
 * is noise.
 */
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { test, expect } from '@playwright/test'
import { gotoView, type ThemeId } from './helpers/render'

const REGISTRY = JSON.parse(
  readFileSync(path.join(__dirname, 'views.json'), 'utf8'),
) as { views: string[] }

const THEMES: ThemeId[] = ['paper', 'ship-dark']

for (const view of REGISTRY.views) {
  for (const theme of THEMES) {
    test(`view: ${view} [${theme}]`, async ({ page }) => {
      await gotoView(page, view, theme)
      await expect(page).toHaveScreenshot(`views/${view}.${theme}.png`, {
        fullPage: false,
      })
    })
  }
}
