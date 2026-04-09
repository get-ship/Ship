import { test, expect } from '@playwright/test'

test('settings page shows daemon section', async ({ page }) => {
  await page.goto('/studio/settings')
  // The CLIConnectionSection renders with title "Daemon"
  await expect(page.locator('text=Daemon')).toBeVisible()
})

test('settings page shows settings heading', async ({ page }) => {
  await page.goto('/studio/settings')
  await expect(page.locator('h1:has-text("Settings")')).toBeVisible()
})
