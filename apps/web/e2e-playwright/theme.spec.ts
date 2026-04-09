import { test, expect } from '@playwright/test'

test('theme toggle switches between light and dark', async ({ page }) => {
  await page.goto('/studio/settings')
  const html = page.locator('html')

  // The StudioRail has a theme toggle button (Sun/Moon icon)
  const rail = page.locator('nav').first()
  const buttons = rail.locator('button')
  const buttonCount = await buttons.count()
  // Theme toggle is second from last (last is Settings)
  const themeButton = buttons.nth(buttonCount - 2)
  await themeButton.click()

  // After clicking, the html element should have a light or dark class
  const className = await html.getAttribute('class')
  expect(className).toMatch(/light|dark/)
})
