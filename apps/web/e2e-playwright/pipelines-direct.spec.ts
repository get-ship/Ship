import { test } from '@playwright/test'
import fs from 'node:fs'

const TOKEN = fs.readFileSync(`${process.env.HOME}/.ship/daemon.token`, 'utf8').trim()

test('pipelines daemon direct', async ({ page }) => {
  await page.goto(`http://127.0.0.1:9315/api/runtime/view/pipelines/raw?token=${TOKEN}`)
  await page.waitForTimeout(3000)
  await page.screenshot({
    path: '../../.artifacts/qa-screenshots/pipelines-daemon-direct.png',
    fullPage: true,
  })
})
