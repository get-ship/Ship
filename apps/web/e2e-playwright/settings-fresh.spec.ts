import { test, chromium } from '@playwright/test'
import fs from 'node:fs'

const TOKEN = fs.readFileSync(`${process.env.HOME}/.ship/daemon.token`, 'utf8').trim()

test('settings fresh context', async () => {
  const browser = await chromium.launch()
  const ctx = await browser.newContext({ bypassCSP: true })
  const page = await ctx.newPage()

  const logs: string[] = []
  page.on('console', (msg) => logs.push(`[${msg.type()}] ${msg.text()}`))
  page.on('pageerror', (err) => logs.push(`[ERR] ${err.message}`))
  page.on('request', (req) => {
    if (req.url().includes('/api/')) logs.push(`[req] ${req.method()} ${req.url().slice(0, 150)}`)
  })
  page.on('response', (resp) => {
    if (resp.url().includes('/api/')) logs.push(`[res ${resp.status()}] ${resp.url().slice(0, 150)}`)
  })

  await page.goto(`http://127.0.0.1:9315/api/runtime/view/settings/raw?token=${TOKEN}`)
  await page.waitForTimeout(6000)

  const hasConnecting = await page.locator('text=Connecting').isVisible()
  const hasSettingsH1 = await page.locator('h1:has-text("Settings")').count()
  console.log('=== logs ===')
  logs.forEach((l) => console.log(l))
  console.log(`\n=== state ===\nconnecting visible: ${hasConnecting}\nh1 Settings count: ${hasSettingsH1}`)

  await browser.close()
})
