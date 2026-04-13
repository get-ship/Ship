import { test } from '@playwright/test'
import fs from 'node:fs'

const TOKEN = fs.readFileSync(`${process.env.HOME}/.ship/daemon.token`, 'utf8').trim()

test('pipelines debug', async ({ page }) => {
  const logs: string[] = []
  page.on('console', (msg) => logs.push(`[${msg.type()}] ${msg.text()}`))
  page.on('pageerror', (err) => logs.push(`[ERR] ${err.message}`))
  page.on('response', (resp) => {
    if (resp.url().includes('/api/')) logs.push(`[net] ${resp.status()} ${resp.url().slice(0, 120)}`)
  })

  await page.goto(`http://127.0.0.1:9315/api/runtime/view/pipelines/raw?token=${TOKEN}`)
  await page.waitForTimeout(4000)
  const rootHtml = await page.$eval('#pipelines-root', (el) => el.outerHTML.slice(0, 400))
  console.log('=== console + network ===')
  logs.forEach((l) => console.log(l))
  console.log('\n=== pipelines-root outerHTML (first 400 chars) ===')
  console.log(rootHtml)
})
