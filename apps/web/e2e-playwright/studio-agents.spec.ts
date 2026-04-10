import { test, expect } from '@playwright/test'

test('agents page loads without errors', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (err) => {
    // WebSocket errors are expected when the daemon is not running
    if (err.message.includes('WebSocket') || err.message.includes('websocket')) return
    errors.push(err.message)
  })
  await page.goto('/studio/agents')
  await expect(page).toHaveURL(/studio\/agents/)
  await page.waitForTimeout(2000)
  expect(errors).toHaveLength(0)
})
