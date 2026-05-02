/**
 * Git view live-data probe — validates the `<data name="overview" live />`
 * SSE channel end-to-end.
 *
 * Acceptance criteria from job-spec:
 *   - Page renders git header, NOT "Loading repo overview…".
 *   - `__subscribe__` URL is emitted into the JSON island.
 *   - SSE endpoint returns patch-signals events with ok-kind envelopes.
 */
import { test, expect } from '@playwright/test'
import { waitForAttach, screenshot } from './_helpers'

test('live-data: SSR island contains __subscribe__ URL', async ({ request }) => {
  const res = await request.get('/_sui/render/git?format=json')
  expect(res.status()).toBe(200)
  const body = await res.json() as { init: Record<string, unknown> }
  expect(body.init.__subscribe__).toBe('/_sui/views/git/sse')
})

test('live-data: /_sui/views/git/sse returns patch-signals event with ok value', async ({ page, baseURL }) => {
  // Navigate first so page.evaluate runs in a real browsing context.
  await page.goto('/sui/git')

  // Use page.evaluate + fetch with AbortController so we can cancel the
  // long-lived SSE connection once we've read the first event.
  const url = `${baseURL}/_sui/views/git/sse`
  const result = await page.evaluate(async (sseUrl: string) => {
    const controller = new AbortController()
    const resp = await fetch(sseUrl, {
      headers: { Accept: 'text/event-stream' },
      signal: controller.signal,
    })
    if (!resp.ok || !resp.body) return { status: resp.status, chunk: '' }
    const reader = resp.body.getReader()
    const decoder = new TextDecoder()
    let text = ''
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      text += decoder.decode(value, { stream: true })
      // Stop once we have a complete event (ends with double newline).
      if (text.includes('\n\n')) break
    }
    controller.abort()
    return { status: resp.status, chunk: text }
  }, url)

  expect(result.status).toBe(200)
  expect(result.chunk).toContain('event: patch-signals')

  const dataLine = result.chunk.split('\n').find((l: string) => l.startsWith('data: '))
  expect(dataLine).toBeDefined()
  const payload = JSON.parse(dataLine!.slice('data: '.length)) as {
    name: string
    value: { kind: string; value?: unknown }
  }
  expect(payload.name).toBe('overview')
  expect(payload.value.kind).toBe('ok')
  expect(typeof (payload.value.value as { branch: string }).branch).toBe('string')
})

test('live-data: git page resolves from loading to full overview', async ({ page }) => {
  const jsErrors: string[] = []
  page.on('pageerror', (err) => jsErrors.push(err.message))

  await page.goto('/sui/git')
  await waitForAttach(page)

  // The git-head div only renders when overview.kind === 'ok'.
  await expect(page.locator('.git-head')).toBeVisible({ timeout: 20_000 })

  // The loading placeholder must be gone.
  await expect(page.locator('[data-test="loading"]')).toHaveCount(0)

  await screenshot(page, 'git-live-resolved')

  const ourErrors = jsErrors.filter(
    (e) => !e.includes('ResizeObserver') && !e.includes('Non-Error promise rejection'),
  )
  expect(ourErrors).toHaveLength(0)
})
