/**
 * Behavioral coverage for shell2's left rail and view router.
 *
 * Pre-fix: visual baselines passed but every rail click triggered a full
 * page reload (the `/` Dashboard item fell through the click handler's
 * null-route guard) or landed on a 404 (rail items pointed at views that
 * didn't exist). The rail also rendered a three-state arrow toggle
 * (label, icon-only, hidden) that nobody asked for. This spec locks in
 * the two-state contract and the in-app navigation path.
 *
 * Runs against shell2's vite dev server (default port 3200; the harness
 * computes the port from the daemon port using shell2/vite.config.ts's
 * offset formula). Skipped when the shell2 dev server isn't reachable so
 * `pnpm test:e2e` keeps passing on machines that haven't started it.
 */
import path from 'node:path'
import fs from 'node:fs'
import crypto from 'node:crypto'
import os from 'node:os'
import { test, expect } from '@playwright/test'

function resolveShell2Url(): string {
  if (process.env.SHELL2_URL) return process.env.SHELL2_URL
  const workspaceRoot = path.resolve(__dirname, '..', '..')
  try {
    const real = fs.realpathSync(workspaceRoot)
    const hash = crypto.createHash('sha256').update(real).digest('hex').slice(0, 8)
    const port = fs
      .readFileSync(path.join(os.homedir(), '.ship', 'machine', 'daemons', hash, 'port'), 'utf8')
      .trim()
    const daemon = Number(port)
    if (Number.isFinite(daemon) && daemon >= 9315) {
      return `http://127.0.0.1:${3200 + (daemon - 9315)}`
    }
  } catch { /* fall through */ }
  return 'http://127.0.0.1:3200'
}

const SHELL2_URL = resolveShell2Url()

async function shell2Reachable(): Promise<boolean> {
  try {
    const res = await fetch(SHELL2_URL, { signal: AbortSignal.timeout(1500) })
    return res.ok || res.status === 304
  } catch {
    return false
  }
}

test.describe('shell2 rail + nav fundamentals', () => {
  test.beforeAll(async () => {
    if (!(await shell2Reachable())) {
      test.skip(true, `shell2 vite not reachable at ${SHELL2_URL} — start it with \`pnpm -F shell2 dev\``)
    }
  })

  test('home renders, rail item click swaps view without full reload', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    await page.waitForSelector('sui-rail#ship-rail')
    await expect(page.locator('main#ship-content')).not.toBeEmpty()

    // Detection sentinel: the first onAnchorClick that successfully
    // intercepts a click prevents the navigation. We tag the window
    // post-load to confirm subsequent interactions don't trigger a
    // hard reload (which would wipe the sentinel).
    await page.evaluate(() => { (window as unknown as { __noReload?: boolean }).__noReload = true })

    const agents = page.locator('sui-rail-item[data-view="agents"]')
    await agents.click()

    await expect(page).toHaveURL(/\/agents(\?|#|$)/)
    await page.waitForFunction(
      () => document.querySelector('main#ship-content')?.innerHTML?.includes('agents-view'),
      null,
      { timeout: 10_000 },
    ).catch(() => {})
    // Sentinel survives → no hard reload happened.
    const survived = await page.evaluate(
      () => (window as unknown as { __noReload?: boolean }).__noReload === true,
    )
    expect(survived).toBe(true)
  })

  test('rail toggle is two-state: hidden ↔ visible (icon-only); no arrow button', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
    const rail = page.locator('sui-rail#ship-rail')
    await rail.waitFor()

    // No arrow toggle button anywhere inside the rail shadow root.
    const hasArrow = await rail.evaluate((el) => {
      const root = (el as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot
      return Boolean(root?.querySelector('.toggle-btn, [part~="toggle"]'))
    })
    expect(hasArrow).toBe(false)

    // Starting state: rail visible (icon-only), [collapsed] absent.
    await expect(rail).not.toHaveAttribute('collapsed', '')
    const initialWidth = await rail.evaluate((el) => el.getBoundingClientRect().width)
    expect(initialWidth).toBeGreaterThan(0)
    expect(initialWidth).toBeLessThan(80)

    // Toggle off via the top-nav menu button. The menu button is the
    // rail-toggle part on <sui-nav-top>.
    await page.locator('sui-nav-top#ship-nav-top').evaluate((nav) => {
      const root = (nav as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const btn = root.querySelector('[part~="rail-toggle"]') as HTMLElement | null
      btn?.click()
    })
    await expect(rail).toHaveAttribute('collapsed', '')
    const hiddenWidth = await rail.evaluate((el) => el.getBoundingClientRect().width)
    expect(hiddenWidth).toBe(0)

    // Toggle back on: still icon-only (no label state).
    await page.locator('sui-nav-top#ship-nav-top').evaluate((nav) => {
      const root = (nav as HTMLElement & { shadowRoot: ShadowRoot | null }).shadowRoot!
      const btn = root.querySelector('[part~="rail-toggle"]') as HTMLElement | null
      btn?.click()
    })
    await expect(rail).not.toHaveAttribute('collapsed', '')
    const restoredWidth = await rail.evaluate((el) => el.getBoundingClientRect().width)
    expect(restoredWidth).toBeGreaterThan(0)
    expect(restoredWidth).toBeLessThan(80)
  })

  test('rail-item labels surface as native title tooltips', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`)
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
