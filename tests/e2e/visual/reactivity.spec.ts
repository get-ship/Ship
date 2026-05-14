/**
 * End-to-end smoke: compile → SSR splice → attach → reactive update.
 *
 * The home view carries the canonical reactive primitive — a `seen()`
 * signal incremented by `data-test="bump"`. If the compile / mount /
 * hydrate pipeline regresses, this test surfaces it before any of the
 * pixel-based view baselines run.
 */
import { test, expect } from '@playwright/test'
import { waitForAttach, injectStability, pinTheme } from './helpers/render'

const BUMP = '[data-test="bump"]'

test('SSR payload: home view JSON carries reactive sui markers', async ({ request }) => {
  // The host shell fetches /_sui/views/<name> as JSON; the /sui/<name>
  // route returns the generic shell HTML for any name, so we have to look
  // inside the JSON to verify what the compiler actually produced.
  const res = await request.get('/_sui/views/home')
  expect(res.status()).toBe(200)
  const payload = (await res.json()) as { name: string; html: string; css?: string }
  expect(payload.name).toBe('home')
  expect(payload.html).toContain('data-test="bump"')
  // The walker emits `<!--sui:text:bN-->...<!--/sui-->` markers around the
  // dynamic text node. Presence of the marker proves the compiler wired
  // the reactive binding; the live value is populated post-attach.
  expect(payload.html).toMatch(/<!--sui:text:b\d+-->[\s\S]*?<!--\/sui-->/)
})

test('reactive: clicking Bump updates the session counter text', async ({ page }) => {
  await pinTheme(page, 'paper')
  await page.goto('/sui/home')
  await waitForAttach(page)
  await injectStability(page)

  const text = page.locator('section.counter p')
  // Initial state — the home view stores the count in a session signal,
  // so on a fresh page load it's 0.
  await expect(text).toContainText(/0 times/)

  await page.locator(BUMP).click()
  await expect(text).toContainText(/1 times/)

  await page.locator(BUMP).click()
  await page.locator(BUMP).click()
  await expect(text).toContainText(/3 times/)
})
