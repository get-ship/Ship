/**
 * Counter view e2e tests (tier-1).
 *
 * The counter view (`views/counter/page.sui`) is the simplest Ship view:
 * declarative state via a minimal controller, no live data subscription.
 * It is the primary confidence gate before mass porting.
 *
 * Three tiers of coverage:
 *   1. SSR structure — verifies the server produces correct HTML markers.
 *   2. Attach + interaction — verifies the client runtime wires up signals.
 *   3. Progressive enhancement — SSR alone carries the initial value.
 */
import { test, expect } from '@playwright/test'
import { screenshot, waitForAttach } from './_helpers'

const VALUE  = '[data-test="value"]'
const INC    = '[data-test="inc"]'
const DEC    = '[data-test="dec"]'
const RESET  = '[data-test="reset"]'

// ── 1. SSR structure ─────────────────────────────────────────────────────────

test('SSR: counter page returns HTML 200', async ({ request }) => {
  const res = await request.get('/sui/counter')
  expect(res.status()).toBe(200)
  expect(res.headers()['content-type']).toMatch(/text\/html/)
})

test('SSR: contains counter wrapper and data-test selectors', async ({ request }) => {
  const body = await (await request.get('/sui/counter')).text()
  expect(body).toContain('<div class="counter">')
  expect(body).toContain('data-test="value"')
  expect(body).toContain('data-test="inc"')
  expect(body).toContain('data-test="dec"')
  expect(body).toContain('data-test="reset"')
})

test('SSR: buttons use on:click marker attributes (not data-set-*)', async ({ request }) => {
  // The SUI compiler emits `on:click="b<N>"` as the SSR marker. Datastar's
  // `data-on-click` / `data-set-*` must NOT appear — that's the old stack.
  const body = await (await request.get('/sui/counter')).text()
  expect(body).toMatch(/on:click="b\d+"/)
  expect(body).not.toContain('data-on-click')
  expect(body).not.toContain('data-set-')
})

test('SSR: __set_init__ island seeds count at 0', async ({ request }) => {
  const body = await (await request.get('/sui/counter')).text()
  expect(body).toContain('<script type="application/json" id="__set_init__">')
  expect(body).toContain('"count":0')
})

// ── 2. Progressive enhancement (no JS) ───────────────────────────────────────

test('PE: SSR emits count value 0 inline between comment markers', async ({ request }) => {
  // The walker emits `<!--sui:text:bN-->0<!--/sui-->` — the value is readable
  // without JS, confirming server-rendered initial state is correct.
  const body = await (await request.get('/sui/counter')).text()
  expect(body).toMatch(/<!--sui:text:b\d+-->0<!--\/sui-->/)
})

// ── 3. Attach + interaction ───────────────────────────────────────────────────

test('attach: count displays 0 after attach completes', async ({ page }) => {
  await page.goto('/sui/counter')
  await waitForAttach(page)
  await expect(page.locator(VALUE)).toHaveText('0')
})

test('attach: +3 → 3, - → 2, reset → 0', async ({ page }) => {
  await page.goto('/sui/counter')
  await waitForAttach(page)

  // Increment three times
  await page.locator(INC).click()
  await page.locator(INC).click()
  await page.locator(INC).click()
  await expect(page.locator(VALUE)).toHaveText('3')

  // Decrement once
  await page.locator(DEC).click()
  await expect(page.locator(VALUE)).toHaveText('2')

  // Reset to zero
  await page.locator(RESET).click()
  await expect(page.locator(VALUE)).toHaveText('0')

  await screenshot(page, 'counter-after-reset')
})

test('attach: decrement below zero works (no floor)', async ({ page }) => {
  await page.goto('/sui/counter')
  await waitForAttach(page)

  await page.locator(DEC).click()
  await expect(page.locator(VALUE)).toHaveText('-1')
})
