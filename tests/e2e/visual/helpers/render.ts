/**
 * Render helpers — page setup for visual baselines.
 *
 * Two render modes:
 *   • `gotoView(page, name)` — navigate to a daemon-served `/sui/<name>`
 *     route, set the requested theme, wait for runtime attach.
 *   • `gotoComponentHarness(page, slot)` — intercept a daemon-origin URL
 *     and serve a synthesized harness page that registers every <sui-*>
 *     custom element and renders the slot content. Origin stays on the
 *     daemon so /_sui/* + /api/assets/* resolve normally.
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
 * mount script fetches `/_sui/views/<name>` and writes the result into
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

/**
 * Build the synthesized harness HTML. Mirrors the daemon's view shell
 * (importmap + components.js + tokens) so component primitives boot the
 * same way they do inside a real view.
 */
function harnessHtml(slot: string): string {
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>SUI harness</title>
  <link rel="stylesheet" href="/api/assets/view-tokens.css" />
  <script type="importmap">
  {
    "imports": {
      "@ship/sui-runtime": "/_sui/runtime.js",
      "@ship/sui-dom":     "/_sui/dom.js",
      "@ship/sdk-client":  "/_sui/sdk-client.js"
    }
  }
  </script>
  <script type="module" src="/_sui/components.js"></script>
  <style>
    html, body { margin: 0; padding: 0; background: var(--background, #fff); color: var(--foreground, #111); font-family: var(--font-sans, system-ui, sans-serif); }
    #harness { padding: 32px; display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-start; }
  </style>
</head>
<body>
  <div id="harness">${slot}</div>
  <script>
    (function () {
      var deadline = Date.now() + 4000
      function tick () {
        var pending = Array.prototype.slice
          .call(document.querySelectorAll('#harness *'))
          .filter(function (el) { return el.tagName.includes('-') && !customElements.get(el.tagName.toLowerCase()) })
        if (pending.length === 0 || Date.now() > deadline) {
          window.__harnessReady = true
          return
        }
        requestAnimationFrame(tick)
      }
      tick()
    })()
  </script>
</body>
</html>`
}

/**
 * Render a single-component harness page. Intercepts a daemon-origin URL
 * so the synthesized HTML executes with the correct origin for /_sui/*.
 */
export async function gotoComponentHarness(
  page: Page,
  slot: string,
  theme: ThemeId,
): Promise<void> {
  await pinTheme(page, theme)
  const html = harnessHtml(slot)
  await page.route('**/__sui_harness', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'text/html; charset=utf-8',
      body: html,
    }),
  )
  await page.goto('/__sui_harness')
  await page.waitForFunction(
    () => (window as unknown as { __harnessReady?: boolean }).__harnessReady === true,
    null,
    { timeout: 10_000 },
  )
  await injectStability(page)
  await page.evaluate(
    () => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))),
  )
}
