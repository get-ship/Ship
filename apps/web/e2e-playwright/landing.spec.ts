import { test, expect } from '@playwright/test'

test('landing page renders Ship branding', async ({ page }) => {
  await page.goto('/')
  // The landing nav renders "SHIP" text
  await expect(page.locator('nav').locator('text=SHIP')).toBeVisible()
})

test('Studio link navigates to /studio', async ({ page }) => {
  await page.goto('/')
  await page.locator('nav a:has-text("Studio")').first().click()
  await expect(page).toHaveURL(/studio/)
})

test('Registry link navigates to /registry', async ({ page }) => {
  await page.goto('/')
  await page.locator('nav a:has-text("Registry")').first().click()
  await expect(page).toHaveURL(/registry/)
})
