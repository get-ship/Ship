import { test, expect } from '@playwright/test'

test('studio layout shows the navigation rail', async ({ page }) => {
  await page.goto('/studio/agents')
  // The StudioRail renders a nav element
  const rail = page.locator('nav').first()
  await expect(rail).toBeVisible()
})

test('studio rail contains interactive elements', async ({ page }) => {
  await page.goto('/studio/agents')
  // The rail nav contains clickable buttons wrapped in tooltips
  const rail = page.locator('nav').first()
  await expect(rail).toBeVisible()
  // The rail should contain at least one button element
  const buttons = rail.locator('button')
  await expect(buttons.first()).toBeVisible()
})
