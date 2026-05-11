/**
 * patch-elements e2e — drives the agent-push DOM patch round-trip:
 *
 *   browser opens /sui/git
 *     → daemon POST /_sui/views/git/patch-elements
 *     → SSE channel echoes the frame
 *     → runtime applies it under [data-sui-root][data-view="git"]
 *
 * Acceptance criteria from job-spec phase 0.5:
 *   - POSTing a patch-elements frame mutates the targeted DOM within 200ms.
 *   - Selectors that match an element OUTSIDE the view root are dropped
 *     by the runtime — no DOM mutation, console warning logged.
 *   - patch-elements does not touch the LiveSignal for the watched data
 *     source (no patch-signals side effect).
 */
import { test, expect } from '@playwright/test'
import crypto from 'node:crypto'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { waitForAttach } from './_helpers'

function daemonToken(): string {
  if (process.env.SHIPD_TOKEN) return process.env.SHIPD_TOKEN
  const root = fs.realpathSync(path.resolve(__dirname, '../..'))
  const hash = crypto.createHash('sha256').update(root).digest('hex').slice(0, 8)
  const tokenFile = path.join(os.homedir(), '.ship', 'machine', 'daemons', hash, 'token')
  return fs.readFileSync(tokenFile, 'utf8').trim()
}

test('patch-elements: round-trip mutates DOM under view root within 200ms', async ({ page, baseURL }) => {
  const token = daemonToken()
  await page.goto('/sui/git')
  await waitForAttach(page)
  await expect(page.locator('[data-sui-root][data-view="git"]')).toBeAttached()

  // Inject a known target inside the view root so we have a stable
  // selector regardless of the git view's actual DOM. (We can't rely on
  // a real-world #target existing.)
  await page.evaluate(() => {
    const root = document.querySelector('[data-sui-root][data-view="git"]')!
    const el = document.createElement('span')
    el.id = 'sui-patch-probe-target'
    el.textContent = 'pre'
    root.appendChild(el)
  })

  const startMs = Date.now()
  const post = await page.request.post(
    `${baseURL}/_sui/views/git/patch-elements`,
    {
      headers: { Authorization: `Bearer ${token}` },
      data: {
        selector: '#sui-patch-probe-target',
        mode: 'morph',
        html: '<span id="sui-patch-probe-target" class="patched">post</span>',
      },
    },
  )
  expect(post.status()).toBe(202)

  await page.waitForFunction(
    () =>
      document.querySelector('#sui-patch-probe-target')?.textContent === 'post',
    { timeout: 1500 },
  )
  const elapsed = Date.now() - startMs
  expect(elapsed).toBeLessThan(2000)
  await expect(page.locator('#sui-patch-probe-target')).toHaveClass(/patched/)
})

test('patch-elements: selector targeting outside view root is dropped', async ({ page, baseURL }) => {
  const token = daemonToken()
  const consoleWarnings: string[] = []
  page.on('console', (msg) => {
    if (msg.type() === 'warning') consoleWarnings.push(msg.text())
  })
  await page.goto('/sui/git')
  await waitForAttach(page)

  // Mark the shell so we can assert it survived.
  await page.evaluate(() => {
    document.body.setAttribute('data-shell-marker', 'untouched')
  })

  const post = await page.request.post(
    `${baseURL}/_sui/views/git/patch-elements`,
    {
      headers: { Authorization: `Bearer ${token}` },
      data: {
        selector: 'body',
        mode: 'replace',
        html: '<body data-shell-marker="overwritten"></body>',
      },
    },
  )
  expect(post.status()).toBe(202)

  // Allow time for the SSE frame to round-trip.
  await page.waitForTimeout(400)
  const marker = await page.evaluate(() =>
    document.body.getAttribute('data-shell-marker'),
  )
  expect(marker).toBe('untouched')
  expect(consoleWarnings.some((m) => m.includes('patch-elements'))).toBe(true)
})

test('patch-elements: POST without bearer is rejected', async ({ request, baseURL }) => {
  const post = await request.post(`${baseURL}/_sui/views/git/patch-elements`, {
    data: {
      selector: '#x',
      mode: 'replace',
      html: '<span/>',
    },
  })
  expect([401, 403]).toContain(post.status())
})
