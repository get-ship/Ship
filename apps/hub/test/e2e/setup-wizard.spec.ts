/**
 * Behavioral coverage for the first-run setup wizard.
 *
 * Drives shell (vite dev server) so we exercise the SUI-mounted
 * `views/setup/` view + the shell `setup-wizard.ts` first-run hook
 * in a real browser. Skips when shell isn't reachable.
 *
 * The wizard talks to two daemon endpoints:
 *   - GET /api/v1/providers/installed  (provider validation)
 *   - POST /api/v1/projects             (project creation)
 *
 * These are mocked at the page level so the tests run deterministically
 * regardless of what's actually installed on the test machine.
 */
import { test, expect, type Page } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

const COMPLETED_KEY = 'ship-setup-completed'
const STATE_KEY = 'ship-setup-state'

async function resetSetupState(page: Page): Promise<void> {
  await page.addInitScript(([completedKey, stateKey]) => {
    try {
      localStorage.removeItem(completedKey)
      localStorage.removeItem(stateKey)
    } catch { /* ignore */ }
  }, [COMPLETED_KEY, STATE_KEY])
}

async function mockProviders(page: Page, providers: string[]): Promise<void> {
  await page.route('**/api/v1/harnesses/installed', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ harnesses: providers }),
    })
  })
}

async function mockProjectCreate(page: Page, ok = true): Promise<void> {
  await page.route('**/api/v1/projects', async (route) => {
    if (route.request().method() !== 'POST') {
      return route.fallback()
    }
    if (ok) {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          projects: [{ id: 'p1', name: 'first', path: '/tmp/first' }],
          active: 'p1',
        }),
      })
      return
    }
    await route.fulfill({
      status: 400,
      contentType: 'application/json',
      body: JSON.stringify({ ok: false, error: { message: 'mock failure' } }),
    })
  })
}

async function gotoWizard(page: Page, path = '/'): Promise<void> {
  await page.goto(`${SHELL2_URL}${path}`, { waitUntil: 'load' })
  await page.waitForSelector('[data-test="setup-wizard"]', { timeout: 15_000 })
}

async function fillIdentity(page: Page, name: string): Promise<void> {
  await page.locator('[data-test="step-identity-name"] input').fill(name)
  await page.click('[data-test="step-identity-next"]')
  await page.waitForSelector('[data-test="step-provider"]', { timeout: 5_000 })
}

async function pickProvider(page: Page, id: string): Promise<void> {
  await page.click(`[data-test="provider-${id}"]`)
}

async function fillProject(page: Page, name: string, projPath: string): Promise<void> {
  await page.locator('[data-test="step-project-name"] input').fill(name)
  await page.locator('[data-test="step-project-path"] input').fill(projPath)
  await page.click('[data-test="step-project-next"]')
  await page.waitForSelector('[data-test="step-sample"]', { timeout: 5_000 })
}

test.describe('setup wizard — first-run flow', () => {
  skipUnlessShell2()

  test.beforeEach(async ({ page }) => {
    await resetSetupState(page)
    await mockProviders(page, ['claude'])
    await mockProjectCreate(page, true)
  })

  test('fresh state → opening Ship triggers wizard', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/`, { waitUntil: 'load' })
    await page.waitForSelector('[data-test="setup-wizard"]', { timeout: 15_000 })
    expect(page.url()).toContain('/setup')
    await expect(page.locator('[data-test="step-welcome"]')).toBeVisible()
  })

  test('fill all steps → land on Home view of default project', async ({ page }) => {
    await gotoWizard(page)
    // Step 1 → 2
    await page.click('[data-test="step-welcome-next"]')
    await page.waitForSelector('[data-test="step-identity"]', { timeout: 5_000 })
    // Step 2 → 3
    await fillIdentity(page, 'Ada Lovelace')
    // Step 3 → 4
    await pickProvider(page, 'claude')
    await page.waitForSelector('[data-test="provider-ok"]', { timeout: 5_000 })
    await page.click('[data-test="step-provider-next"]')
    await page.waitForSelector('[data-test="step-project"]', { timeout: 5_000 })
    // Step 4 → 5
    await fillProject(page, 'first', '/tmp/first')
    // Step 5 → 6
    await page.click('[data-test="step-sample-next"]')
    await page.waitForSelector('[data-test="step-done"]', { timeout: 5_000 })
    // Step 6 → finish, lands on Home (`/` resolves to /agents in shell)
    await page.click('[data-test="step-done-finish"]')
    await page.waitForFunction(() => !location.pathname.startsWith('/setup'), null, { timeout: 5_000 })
    const completed = await page.evaluate((k) => localStorage.getItem(k), COMPLETED_KEY)
    expect(completed).toBe('1')
  })

  test('mid-flow close → reopen Ship → resumes at the same step with prior values', async ({ page }) => {
    await gotoWizard(page)
    await page.click('[data-test="step-welcome-next"]')
    await page.waitForSelector('[data-test="step-identity"]')
    await fillIdentity(page, 'Grace Hopper')
    // Now on Provider (step 3). Reload — wizard should resume at step 3
    // with identityName preserved.
    await page.reload({ waitUntil: 'load' })
    await page.waitForSelector('[data-test="setup-wizard"]', { timeout: 15_000 })
    await expect(page.locator('[data-test="step-provider"]')).toBeVisible()
    // Walk back to verify the name persisted.
    await page.click('text=Back')
    await page.waitForSelector('[data-test="step-identity"]')
    const name = await page.locator('[data-test="step-identity-name"] input').inputValue()
    expect(name).toBe('Grace Hopper')
  })

  test('provider validation fail → inline error, user can retry', async ({ page }) => {
    // Override the providers mock to return empty so the chosen provider
    // never validates — surfaces the inline error + retry button.
    await page.unroute('**/api/v1/providers/installed')
    await mockProviders(page, [])
    await gotoWizard(page)
    await page.click('[data-test="step-welcome-next"]')
    await page.waitForSelector('[data-test="step-identity"]')
    await fillIdentity(page, 'Margaret Hamilton')
    await pickProvider(page, 'claude')
    await expect(page.locator('[data-test="provider-error"]')).toBeVisible({ timeout: 5_000 })
    // Now swap the mock to a successful response, retry, expect OK.
    await page.unroute('**/api/v1/providers/installed')
    await mockProviders(page, ['claude'])
    await page.click('[data-test="provider-retry"]')
    await expect(page.locator('[data-test="provider-ok"]')).toBeVisible({ timeout: 5_000 })
  })

  test('skip-to-end (Cmd+.) → confirms, applies defaults, exits to Home', async ({ page }) => {
    await gotoWizard(page)
    // Trigger Cmd+. (Meta+.)
    await page.keyboard.press('Meta+.')
    await expect(page.locator('[data-test="setup-confirm-exit"]')).toBeVisible({ timeout: 5_000 })
    // Confirm dialog: click the Confirm button (default footer slot).
    await page.locator('[data-test="setup-confirm-exit"] sui-button[variant="solid"]').click()
    await page.waitForFunction(() => !location.pathname.startsWith('/setup'), null, { timeout: 5_000 })
    const completed = await page.evaluate((k) => localStorage.getItem(k), COMPLETED_KEY)
    expect(completed).toBe('1')
  })

  test('re-entry from Settings → opens wizard fresh', async ({ page }) => {
    // Mark setup completed, then visit Settings → click "Run setup again".
    await page.addInitScript((k) => {
      try { localStorage.setItem(k, '1') } catch { /* ignore */ }
    }, COMPLETED_KEY)
    await page.goto(`${SHELL2_URL}/settings`, { waitUntil: 'load' })
    await page.waitForSelector('[data-test="settings-rerun-setup"]', { timeout: 15_000 })
    await page.click('[data-test="settings-rerun-setup"]')
    await page.waitForSelector('[data-test="setup-wizard"]', { timeout: 15_000 })
    await expect(page.locator('[data-test="step-welcome"]')).toBeVisible()
    const completed = await page.evaluate((k) => localStorage.getItem(k), COMPLETED_KEY)
    expect(completed).toBeNull()
  })
})
