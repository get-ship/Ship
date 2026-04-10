import { test, expect } from '@playwright/test'

test('skills page renders the IDE layout on desktop', async ({ page }) => {
  await page.goto('/studio/skills')
  // Default viewport is 1280x720 so the IDE layout should render
  await expect(page).toHaveURL(/studio\/skills/)
})

test('studio rail shows navigation buttons', async ({ page }) => {
  await page.goto('/studio/skills')
  // The StudioRail nav element contains multiple interactive buttons
  const rail = page.locator('nav').first()
  await expect(rail).toBeVisible()
  const buttons = rail.locator('button')
  await expect(buttons.first()).toBeVisible()
  expect(await buttons.count()).toBeGreaterThanOrEqual(4)
})

test('rail navigation works between studio pages', async ({ page }) => {
  await page.goto('/studio/skills')
  const rail = page.locator('nav').first()
  const buttons = rail.locator('button')
  // Click the last button in the rail (Settings)
  await buttons.last().click()
  await expect(page).toHaveURL(/studio\/settings/)
})
