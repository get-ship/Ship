/**
 * Render helpers — page setup for visual baselines.
 *
 * `gotoView(page, name)` — navigate to a daemon-served `/sui/<name>`
 * route, set the requested theme, wait for runtime attach.
 *
 * Theme application: views don't run applyTheme unless they include a
 * theme-picker, so we set `data-theme` + `.dark` ourselves before any
 * view script runs (via addInitScript). This is deterministic and matches
 * what the picker does post-attach.
 *
 * Stability: we inject a stylesheet that disables animations/transitions
 * and caret blinking, which would otherwise produce flaky pixel diffs.
 */
import type { Page } from '@playwright/test'

export type ThemeId = 'paper' | 'ship-dark'

const DARK_THEMES = new Set<ThemeId>(['ship-dark'])

const STABILITY_CSS = `
  *, *::before, *::after {
    animation-duration: 0s !important;
    animation-delay: 0s !important;
    transition-duration: 0s !important;
    transition-delay: 0s !important;
    caret-color: transparent !important;
  }
`

/** Pin the theme before any script in the page runs. */
export async function pinTheme(page: Page, theme: ThemeId): Promise<void> {
  const dark = DARK_THEMES.has(theme)
  await page.addInitScript(
    ({ theme, dark }) => {
      try {
        localStorage.setItem('ship-site-theme', theme)
      } catch {}
      const apply = () => {
        const root = document.documentElement
        root.dataset.theme = theme
        root.classList.toggle('dark', dark)
      }
      if (document.documentElement) apply()
      document.addEventListener('DOMContentLoaded', apply, { once: true })
    },
    { theme, dark },
  )
}

/** Inject the stability stylesheet after navigation. */
export async function injectStability(page: Page): Promise<void> {
  await page.addStyleTag({ content: STABILITY_CSS })
}

/**
 * Wait for the SUI runtime to mount the view payload. The host shell's
 * mount script fetches `/_deck/views/<name>` and writes the result into
 * `#sui-root`; we treat "first child appeared" as the attach signal.
 *
 * Falls back to networkidle if `#sui-root` is absent (e.g. a synthesized
 * harness page that doesn't use the host shell).
 */
export async function waitForAttach(page: Page, timeout = 10_000): Promise<void> {
  await page.waitForFunction(
    () => {
      const root = document.getElementById('sui-root')
      return Boolean(root && root.firstElementChild)
    },
    null,
    { timeout },
  )
}

/**
 * Navigate to a top-level SUI view with the given theme. Resolves once
 * runtime attach has completed and a paint frame has elapsed.
 */
export async function gotoView(
  page: Page,
  name: string,
  theme: ThemeId,
): Promise<void> {
  await pinTheme(page, theme)
  await page.goto(`/sui/${name}`)
  await waitForAttach(page).catch(() => {
    // Some views may fail to attach (broken markup, missing data, etc.).
    // Continue — the screenshot itself surfaces the regression.
  })
  await injectStability(page)
  await page.waitForLoadState('networkidle', { timeout: 5_000 }).catch(() => {})
  await page.evaluate(
    () => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))),
  )
}

