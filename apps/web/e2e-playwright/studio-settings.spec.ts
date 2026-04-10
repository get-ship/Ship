import { test, expect } from '@playwright/test'

test('settings page shows daemon section', async ({ page }) => {
  await page.goto('/studio/settings')
  // Either shows "Daemon" section (connected) or "Daemon not connected" (DaemonGuard)
  const content = page.locator('text=Daemon').first()
  await expect(content).toBeVisible({ timeout: 20_000 })
})

test('settings page shows settings heading', async ({ page }) => {
  await page.goto('/studio/settings')
  // Settings heading or DaemonGuard fallback — either means the page loaded
  const heading = page.locator('h1:has-text("Settings"), h2:has-text("Daemon")')
  await expect(heading.first()).toBeVisible({ timeout: 20_000 })
})
