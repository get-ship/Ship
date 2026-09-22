/**
 * Behavioral coverage for the rail-anchored `<sui-view-picker>` — the
 * port-forward of OG j2 shell's combined rail-customizer + view-browser
 * + install entry popover.
 *
 * Tests drive the chrome surface (shell vite dev server). They skip
 * if the dev server isn't reachable so `bun run test:e2e` keeps
 * passing on machines that haven't started it.
 */
import { test, expect, type Page } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

async function openPicker(page: Page): Promise<void> {
  await page.waitForFunction(() => Boolean(customElements.get('sui-view-picker')))
  await page
    .locator('sui-rail#ship-rail')
    .evaluate((rail) => {
      const root = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const btn = root.querySelector('[part~="view-picker-btn"]') as HTMLElement | null
      btn?.click()
    })
  // Picker has open attribute reflected back from its native popover toggle.
  await page.waitForFunction(() => {
    const rail = document.querySelector('sui-rail#ship-rail')
    const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null })?.shadowRoot
    const picker = sr?.querySelector('sui-view-picker') as HTMLElement | null
    return picker?.hasAttribute('open') === true
  }, null, { timeout: 5000 })
}

function pickerSurface(page: Page) {
  return page
    .locator('sui-rail#ship-rail')
    .evaluateHandle((rail) => {
      const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
      return picker.shadowRoot
    })
}

test.describe('<sui-view-picker> behavioral surface', () => {
  skipUnlessShell2()

  test('rail mounts a view-picker trigger button', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`)
    await page.waitForSelector('sui-rail#ship-rail')
    await page.waitForFunction(() => Boolean(customElements.get('sui-view-picker')))
    const btnVisible = await page
      .locator('sui-rail#ship-rail')
      .evaluate((rail) => {
        const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
        const btn = sr.querySelector('[part~="view-picker-btn"]') as HTMLElement | null
        if (!btn) return false
        const r = btn.getBoundingClientRect()
        return r.width > 0 && r.height > 0
      })
    expect(btnVisible).toBe(true)
  })

  test('clicking trigger opens picker with project header + 2 sections', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`)
    await page.waitForSelector('sui-rail#ship-rail')
    await openPicker(page)
    const shape = await page
      .locator('sui-rail#ship-rail')
      .evaluate((rail) => {
        const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
        const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
        const ps = picker.shadowRoot!
        return {
          hasHeader: !!ps.querySelector('[part~="header"]'),
          hasFilter: !!ps.querySelector('[part~="filter"]'),
          hasPinned: !!ps.querySelector('[part~="section-pinned"]'),
          hasAll: !!ps.querySelector('[part~="section-all"]'),
          hasFooter: !!ps.querySelector('[part~="footer"]'),
          projectName: ps.querySelector('[part~="project-name"]')?.textContent?.trim() ?? '',
          installHref:
            (ps.querySelector('[part~="install-link"]') as HTMLAnchorElement | null)?.getAttribute('href') ?? '',
        }
      })
    expect(shape.hasHeader).toBe(true)
    expect(shape.hasFilter).toBe(true)
    expect(shape.hasFooter).toBe(true)
    // Either section may be empty depending on pin state; both elements should mount.
    expect(shape.hasPinned || shape.hasAll).toBe(true)
    expect(shape.installHref).toBe('/studio/components')
    expect(shape.projectName.length).toBeGreaterThan(0)
  })

  test('typing in filter narrows both sections in real-time', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`)
    await page.waitForSelector('sui-rail#ship-rail')
    await openPicker(page)
    const total = async () => page
      .locator('sui-rail#ship-rail')
      .evaluate((rail) => {
        const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
        const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
        return picker.shadowRoot!.querySelectorAll('.vp-row').length
      })
    const before = await total()
    expect(before).toBeGreaterThan(0)
    // Type a deliberately narrowing string into the filter input.
    await page.locator('sui-rail#ship-rail').evaluate((rail) => {
      const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
      const input = picker.shadowRoot!.querySelector('.vp-filter input') as HTMLInputElement
      input.value = '__nope__nothing__matches__'
      input.dispatchEvent(new Event('input', { bubbles: true }))
    })
    await expect.poll(total, { timeout: 2000 }).toBe(0)
  })

  test('clicking a row navigates and closes the picker', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`)
    await page.waitForSelector('sui-rail#ship-rail')
    await openPicker(page)
    // Find a target view name from the picker rows.
    const target = await page.locator('sui-rail#ship-rail').evaluate((rail) => {
      const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
      const rows = picker.shadowRoot!.querySelectorAll<HTMLElement>('.vp-row')
      return rows[0]?.getAttribute('data-view') ?? null
    })
    expect(target).toBeTruthy()
    await page.locator('sui-rail#ship-rail').evaluate((rail, name) => {
      const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
      ;(picker.shadowRoot!.querySelector(`.vp-row[data-view="${name}"]`) as HTMLElement)?.click()
    }, target)
    await expect(page).toHaveURL(new RegExp(`/${target === 'dashboard' ? '' : target}(\\?|#|$)`))
    const isOpen = await page.locator('sui-rail#ship-rail').evaluate((rail) => {
      const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const picker = sr.querySelector('sui-view-picker') as HTMLElement
      return picker.hasAttribute('open')
    })
    expect(isOpen).toBe(false)
  })

  test('Escape closes the picker', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`)
    await page.waitForSelector('sui-rail#ship-rail')
    await openPicker(page)
    await page.keyboard.press('Escape')
    await expect
      .poll(async () => page.locator('sui-rail#ship-rail').evaluate((rail) => {
        const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
        const picker = sr.querySelector('sui-view-picker') as HTMLElement
        return picker.hasAttribute('open')
      }))
      .toBe(false)
  })

  test('"+ Install view" link targets /studio/components', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`)
    await page.waitForSelector('sui-rail#ship-rail')
    await openPicker(page)
    const href = await page.locator('sui-rail#ship-rail').evaluate((rail) => {
      const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
      return (picker.shadowRoot!.querySelector('[part~="install-link"]') as HTMLAnchorElement).getAttribute('href')
    })
    expect(href).toBe('/studio/components')
  })

  test('active view marks the matching row with is-active + aria-current', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/agents`)
    await page.waitForSelector('sui-rail#ship-rail')
    await openPicker(page)
    const result = await page.locator('sui-rail#ship-rail').evaluate((rail) => {
      const sr = (rail as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const picker = sr.querySelector('sui-view-picker') as HTMLElement & { shadowRoot: ShadowRoot | null }
      const ps = picker.shadowRoot!
      // The picker drives `is-active` on the row whose name matches the
      // first path segment — `agents` here.
      const row = ps.querySelector('.vp-row[data-view="agents"]') as HTMLElement | null
      return {
        present: Boolean(row),
        active: row?.classList.contains('is-active') ?? false,
        aria: row?.getAttribute('aria-current') ?? null,
      }
    })
    if (!result.present) test.skip(true, 'agents view not in catalogue for this build')
    expect(result.active).toBe(true)
    expect(result.aria).toBe('true')
  })
})
