/**
 * Tutorial visual baselines.
 *
 * One screenshot per (step, theme) of the steady-state tutorial
 * page — splash chrome dropped, prose + editor + preview all
 * mounted and the iframe has rendered the starter code.
 *
 * Theme is applied via a pre-script that stamps `data-theme` +
 * `dark` class on <html> BEFORE any other script runs, matching
 * how Starlight's theme switcher works.
 *
 * The iframe inside the preview pane has its own load cycle; we
 * wait for it to post `tutorial:ready` to the parent and for the
 * tutorial app to flip its status text away from "ready" → either
 * "running" or "error", so the screenshot captures the mounted
 * preview rather than the loading skeleton.
 */
import { test, expect } from '@playwright/test'

type ThemeId = 'ship-light' | 'ship-dark'
const THEMES: ThemeId[] = ['ship-light', 'ship-dark']
const STEPS = ['01-hello-signal', '02-derived', '03-bindfor'] as const

// Disable animations + caret blink so diffs stay deterministic.
const STABILITY_CSS = `
  *, *::before, *::after {
    animation-duration: 0s !important;
    animation-delay: 0s !important;
    transition-duration: 0s !important;
    transition-delay: 0s !important;
    caret-color: transparent !important;
  }
`

for (const step of STEPS) {
  for (const theme of THEMES) {
    test(`tutorial/${step} [${theme}]`, async ({ page }) => {
      await page.addInitScript(({ theme }) => {
        const isDark = theme.includes('dark')
        document.documentElement.dataset.theme = theme
        document.documentElement.classList.toggle('dark', isDark)
      }, { theme })

      await page.goto(`/tutorial/${step}/`, { waitUntil: 'load' })
      await page.addStyleTag({ content: STABILITY_CSS })

      // Wait for the editor + iframe to be mounted and the preview
      // to have rendered something (status leaves 'ready').
      await page.waitForSelector('.cm-editor', { state: 'attached' })
      await page.waitForFunction(
        () => {
          const status = document.querySelector('.tut-status-text')?.textContent ?? ''
          return status === 'running' || status.startsWith('opts JSON') || status.includes('error')
        },
        { timeout: 15_000 },
      )

      // Give the iframe one more beat to paint the SUI preview.
      await page.waitForTimeout(400)

      await expect(page).toHaveScreenshot(`tutorial-${step}-${theme}.png`, { fullPage: false })
    })
  }
}
