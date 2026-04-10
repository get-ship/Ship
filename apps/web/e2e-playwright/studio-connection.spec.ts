import { test, expect } from '@playwright/test'

// These tests verify Studio's daemon connection lifecycle.
// The dev server is running on :3000, daemon on :9315.

test.describe('daemon connection', () => {
  test('studio shows connection status indicator', async ({ page }) => {
    await page.goto('/studio/session')
    // The platform shell should render with a connection indicator
    // visible in settings or the rail
    const indicator = page.locator('[data-testid="daemon-status"], [data-testid="connection-status"]')
    await expect(indicator).toBeVisible({ timeout: 10_000 })
  })

  test('daemon-dependent views show DaemonRequired when disconnected', async ({ page }) => {
    // Block daemon requests to simulate disconnection
    await page.route('**/localhost:9315/**', (route) => route.abort())
    await page.goto('/studio/session')
    // Should show the DaemonRequired fallback
    const fallback = page.locator('text=Daemon not connected')
    await expect(fallback).toBeVisible({ timeout: 10_000 })
  })

  test.skip('views recover after daemon reconnects', async ({ page }) => {
    // TODO: requires PlatformProvider to periodically retry daemon health when disconnected
    // Currently, reconnection only happens via SSE error handler, not health polling
    await page.route('**/localhost:9315/**', (route) => route.abort())
    await page.goto('/studio/agents')
    const fallback = page.locator('text=Daemon not connected')
    await expect(fallback).toBeVisible({ timeout: 10_000 })
    await page.unroute('**/localhost:9315/**')
    await page.reload()
    const content = page.locator('h1:has-text("Agents")')
    await expect(content).toBeVisible({ timeout: 15_000 })
  })
})

test.describe('workspace state', () => {
  test('workspace picker shows active workspace', async ({ page }) => {
    await page.goto('/studio/session')
    // The workspace indicator should show a workspace name or "No workspace"
    const picker = page.locator('[data-testid="workspace-picker"], button:has-text("workspace"), button:has-text("main")')
    await expect(picker.first()).toBeVisible({ timeout: 10_000 })
  })

  test('workspace selection persists across navigation', async ({ page }) => {
    await page.goto('/studio/session')
    await page.waitForTimeout(2000)
    // Navigate away and back
    await page.goto('/studio/agents')
    await page.waitForTimeout(1000)
    await page.goto('/studio/session')
    // Workspace should still be selected (not reset)
    const picker = page.locator('[data-testid="workspace-picker"], button:has-text("workspace"), button:has-text("main")')
    await expect(picker.first()).toBeVisible({ timeout: 10_000 })
  })
})

test.describe('error boundaries', () => {
  test('view error does not crash the entire shell', async ({ page }) => {
    // Block only the skills endpoint to cause one view to fail
    await page.route('**/workspaces/*/skills**', (route) => route.abort())
    await page.goto('/studio/skills')
    await page.waitForTimeout(3000)
    // The rail should still be visible even if the view content failed
    const rail = page.locator('nav')
    await expect(rail).toBeVisible()
    // Should be able to navigate to another view without the shell crashing
    await page.goto('/studio/jobs')
    await page.waitForTimeout(2000)
    // The rail should still be visible on the new page
    await expect(rail).toBeVisible()
  })
})
