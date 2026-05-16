/**
 * Behavioral coverage for the theme-aware brand mark and favicon swap.
 *
 * The shell top bar renders <sui-brand-mark> inside <sui-nav-top>'s
 * shadow root. The mark's inline SVG paints its fills from `--primary`
 * and `--background` via `color-mix`, so every theme renders the logo
 * in its own palette without swapping SVG assets. The browser-tab
 * favicon is a separate concern (lives outside CSS scope) — main.ts
 * listens for `ship:theme:change` and swaps the SVG favicon href
 * between ship-logo.svg (light) and ship-logo-secondary.svg (dark).
 *
 * This spec locks in:
 *   - <sui-brand-mark> is rendered in the nav-top brand slot,
 *   - its SVG `c-primary` paint observably changes across themes
 *     (ship-light → ship-dark → contrast → console),
 *   - the SVG favicon href flips between primary / secondary on
 *     dark / light theme application.
 *
 * Runs against shell's vite dev server. Skipped when that server isn't
 * reachable so `bun run test:e2e` keeps passing on machines that
 * haven't started it.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

type FillProbe = { fill: string | null; primary: string | null; background: string | null }

async function readBrandPaint(page: import('@playwright/test').Page): Promise<FillProbe> {
  return page.locator('sui-nav-top#ship-nav-top').evaluate((nav) => {
    const root = (nav as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
    const mark = root?.querySelector('sui-brand-mark') as HTMLElement | null
    const inner = (mark as HTMLElement & { shadowRoot: ShadowRoot | null })?.shadowRoot
    const swatch = inner?.querySelector('.c-primary') as SVGElement | null
    const fill = swatch ? getComputedStyle(swatch).fill : null
    const cs = mark ? getComputedStyle(mark) : null
    return {
      fill,
      primary: cs?.getPropertyValue('--primary')?.trim() ?? null,
      background: cs?.getPropertyValue('--background')?.trim() ?? null,
    }
  })
}

async function applyThemeOnPage(page: import('@playwright/test').Page, id: string): Promise<void> {
  // Drive the same DOM contract <sui-theme-picker> would: data-theme +
  // `.dark` class on <html> + a `ship:theme:change` event with { id, dark }.
  // Mirrors @ship/set/themes#applyTheme so the brand-mark repaints (CSS
  // vars cascade) and main.ts#initFavicon swaps the favicon href.
  await page.evaluate((themeId) => {
    const DARK = new Set(['ship-dark', 'console', 'contrast'])
    document.documentElement.dataset.theme = themeId
    const dark = DARK.has(themeId)
    document.documentElement.classList.toggle('dark', dark)
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', { detail: { id: themeId, dark } }),
    )
  }, id)
}

test.describe('theme-aware brand mark + favicon', () => {
  skipUnlessShell2()

  test('<sui-brand-mark> mounts inside the top-nav brand slot', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.locator('sui-nav-top#ship-nav-top').waitFor()
    await page.waitForFunction(() => Boolean(customElements.get('sui-brand-mark')))

    const hasMark = await page.locator('sui-nav-top#ship-nav-top').evaluate((nav) => {
      const root = (nav as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
      return Boolean(root?.querySelector('sui-brand-mark'))
    })
    expect(hasMark).toBe(true)

    // The mark's shadow root must contain the painted SVG with the
    // `c-primary` class so paint can flow from the host's --primary.
    const hasSvg = await page.locator('sui-nav-top#ship-nav-top').evaluate((nav) => {
      const root = (nav as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
      const mark = root?.querySelector('sui-brand-mark') as HTMLElement | null
      const inner = (mark as HTMLElement & { shadowRoot: ShadowRoot | null })?.shadowRoot
      return Boolean(inner?.querySelector('svg .c-primary'))
    })
    expect(hasSvg).toBe(true)
  })

  test('brand-mark paint visibly changes across themes', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.locator('sui-nav-top#ship-nav-top').waitFor()
    await page.waitForFunction(() => Boolean(customElements.get('sui-brand-mark')))

    // Sample primary fill in four catalog themes. The mark paints
    // `.c-primary` from `var(--primary)` — different themes ship
    // distinct primaries, so the resolved fill must change.
    const samples: Record<string, FillProbe> = {}
    for (const id of ['ship-light', 'ship-dark', 'contrast', 'console']) {
      await applyThemeOnPage(page, id)
      // Wait one frame for the CSS variable repaint to settle.
      await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => r(null))))
      samples[id] = await readBrandPaint(page)
    }

    for (const id of Object.keys(samples)) {
      expect(samples[id]!.fill, `theme ${id} should resolve a paint`).toBeTruthy()
    }

    // Distinctness: across these four themes the resolved primary
    // colours are distinct, so the brand mark renders a distinct fill
    // in each. (The mark's paint *is* `var(--primary)` for `.c-primary`,
    // so this proves the per-theme swap end-to-end.)
    const fills = Object.values(samples).map((s) => s.fill!)
    const unique = new Set(fills)
    expect(unique.size).toBe(fills.length)
  })

  test('SVG favicon href flips between primary / secondary on theme change', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.locator('sui-nav-top#ship-nav-top').waitFor()

    const readFavicon = () => page.evaluate(() => {
      const link = document.querySelector<HTMLLinkElement>('link[rel="icon"][type="image/svg+xml"]')
      return link?.getAttribute('href') ?? null
    })

    // Force a known-light theme — favicon must be the primary (gold) variant.
    await applyThemeOnPage(page, 'ship-light')
    await expect.poll(readFavicon).toBe('/ship-logo.svg')

    // Switch to a dark theme — favicon flips to the secondary (purple) variant.
    await applyThemeOnPage(page, 'ship-dark')
    await expect.poll(readFavicon).toBe('/ship-logo-secondary.svg')

    // High-contrast (dark) — still secondary.
    await applyThemeOnPage(page, 'contrast')
    await expect.poll(readFavicon).toBe('/ship-logo-secondary.svg')

    // Back to light — flips back to primary.
    await applyThemeOnPage(page, 'ship-light')
    await expect.poll(readFavicon).toBe('/ship-logo.svg')
  })
})
