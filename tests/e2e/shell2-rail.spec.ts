/**
 * Behavioral coverage for shell2's left rail and view router.
 *
 * `<sui-rail>` reads its `<sui-rail-item>` light-DOM children as config
 * and each item upgrades into a custom element whose shadow root holds
 * the actual clickable `<a class="item">`. Both `sui-rail` and
 * `sui-rail-item` are defined by the `/_sui/islands/sui-rail/island.js`
 * bundle — shell2's `index.html` must load it explicitly (it is not in
 * `/_sui/components.js`). When that bundle is missing, `sui-rail-item`
 * never upgrades, every item renders at height 0, and the rail shows
 * no navigation at all.
 *
 * This spec locks in:
 *   - the rail-item shadow `<a>` is the real nav target,
 *   - clicking it swaps the view in-app (no full reload),
 *   - the rail collapse toggle (expanded ↔ hidden) driven by the
 *     top-nav rail-toggle button,
 *   - rail-item labels surface as native `title` tooltips,
 *   - the top-nav has no chat-toggle button.
 *
 * Runs against shell2's vite dev server. Skipped when that server isn't
 * reachable so `bun run test:e2e` keeps passing on machines that
 * haven't started it.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell2'

test.describe('shell2 rail + nav fundamentals', () => {
  skipUnlessShell2()

  test('rail-item upgrades into a custom element with a clickable shadow <a>', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.waitForSelector('sui-rail#ship-rail')

    // sui-rail-item must be a registered custom element — if the
    // sui-rail island bundle is missing, this is false and the rail
    // renders no navigation.
    await page.waitForFunction(
      () => Boolean(customElements.get('sui-rail-item')),
      null,
      { timeout: 10_000 },
    )

    const item = page.locator('sui-rail-item[data-view="agents"]')
    await item.waitFor()

    const readShape = () =>
      item.evaluate((el) => {
        const root = (el as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
        const anchor = root?.querySelector('a.item') as HTMLAnchorElement | null
        return {
          hasShadowAnchor: Boolean(anchor),
          href: anchor?.getAttribute('href') ?? null,
          height: el.getBoundingClientRect().height,
        }
      })

    // The item upgrades and lays out asynchronously — poll until it
    // becomes a real, visible row (not the 0-height un-upgraded element).
    await expect.poll(async () => (await readShape()).height).toBeGreaterThan(0)

    const shape = await readShape()
    expect(shape.hasShadowAnchor).toBe(true)
    expect(shape.href).toBe('/agents')
  })

  test('rail item click swaps the view without a full reload', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.waitForSelector('sui-rail#ship-rail')
    await page.waitForFunction(() => Boolean(customElements.get('sui-rail-item')))
    await expect(page.locator('main#ship-content')).not.toBeEmpty()

    // Sentinel: a hard reload wipes this window property. If it
    // survives the navigation, the click was intercepted in-app.
    await page.evaluate(() => { (window as unknown as { __noReload?: boolean }).__noReload = true })

    // Click the real anchor inside the rail-item's shadow root.
    await page.locator('sui-rail-item[data-view="agents"]').evaluate((el) => {
      const root = (el as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
      const anchor = root?.querySelector('a.item') as HTMLAnchorElement | null
      anchor?.click()
    })

    await expect(page).toHaveURL(/\/agents(\?|#|$)/)
    await page.waitForFunction(
      () => document.querySelector('main#ship-content')?.innerHTML?.includes('agents-view'),
      null,
      { timeout: 10_000 },
    )
    const survived = await page.evaluate(
      () => (window as unknown as { __noReload?: boolean }).__noReload === true,
    )
    expect(survived).toBe(true)
  })

  test('rail collapse toggle: expanded ↔ hidden via the top-nav button', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    const rail = page.locator('sui-rail#ship-rail')
    await rail.waitFor()
    await page.waitForFunction(() => Boolean(customElements.get('sui-rail-item')))

    const railWidth = () => rail.evaluate((el) => el.getBoundingClientRect().width)

    // Default: expanded, [collapsed] absent. The rail paints at its
    // collapsed icon width first, then settles to the labelled
    // full width once the rail-items upgrade — poll for that.
    await expect(rail).not.toHaveAttribute('collapsed', '')
    await expect.poll(railWidth).toBeGreaterThan(80)

    const clickRailToggle = () =>
      page.locator('sui-nav-top#ship-nav-top').evaluate((nav) => {
        const root = (nav as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
        const btn = root.querySelector('[part~="rail-toggle"]') as HTMLElement | null
        btn?.click()
      })

    // Collapse → hidden (width 0).
    await clickRailToggle()
    await expect(rail).toHaveAttribute('collapsed', '')
    await expect.poll(railWidth).toBe(0)

    // Restore → expanded again.
    await clickRailToggle()
    await expect(rail).not.toHaveAttribute('collapsed', '')
    await expect.poll(railWidth).toBeGreaterThan(80)
  })

  test('rail-item labels surface as native title tooltips', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.waitForFunction(() => Boolean(customElements.get('sui-rail-item')))
    const item = page.locator('sui-rail-item[data-view="agents"]')
    await item.waitFor()
    const title = await item.evaluate((el) => {
      const root = (el as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
      return root?.querySelector('a.item')?.getAttribute('title') ?? null
    })
    expect(title).toBe('Agents')
  })

  test('no chat button in top-nav', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.locator('sui-nav-top#ship-nav-top').waitFor()
    const hasChat = await page.locator('sui-nav-top#ship-nav-top').evaluate((nav) => {
      const root = (nav as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
      return Boolean(root?.querySelector('[part~="chat-toggle"]'))
    })
    expect(hasChat).toBe(false)
  })
})
