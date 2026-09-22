// Direct playwright check for demo views — bypasses the @playwright/test
// conflict in node_modules by using `playwright` (chromium) directly.
//
// Usage: SHIPD_URL=http://localhost:9341 node tests/e2e/_demos-verify.mjs
import { chromium } from 'playwright'

const base = process.env.SHIPD_URL ?? 'http://localhost:9341'

const failures = []
const ok = []
function pass(msg) { ok.push(msg); console.log(`  ✓ ${msg}`) }
function fail(msg, err) { failures.push({ msg, err }); console.log(`  ✗ ${msg}\n    ${err}`) }

const browser = await chromium.launch()
const ctx = await browser.newContext({ baseURL: base })
const page = await ctx.newPage()
page.on('console', (m) => { if (m.type() === 'error') console.log('  [console error]', m.text()) })

async function attach() {
  // The standalone host page replaces #sui-root contents with the mounted
  // view; wait for that swap to complete by looking for any rendered child
  // that isn't the empty marker, then a brief idle for signals to settle.
  await page.waitForFunction(
    () => {
      const root = document.getElementById('sui-root')
      if (!root) return false
      const t = root.textContent ?? ''
      return root.children.length > 0 && !t.includes('Failed to load') && !t.includes('Mount failed')
    },
    null,
    { timeout: 10_000 },
  )
  await page.waitForLoadState('networkidle')
}

async function expectText(sel, text) {
  await page.waitForFunction(([s, t]) => document.querySelector(s)?.textContent?.trim() === t, [sel, text], { timeout: 5_000 })
}

async function expectCount(sel, n) {
  await page.waitForFunction(([s, c]) => document.querySelectorAll(s).length === c, [sel, n], { timeout: 5_000 })
}

async function run(name, fn) {
  try { await fn(); pass(name) } catch (e) { fail(name, e?.message ?? String(e)) }
}

console.log(`\nRunning demo verification against ${base}\n`)

console.log('[index]')
await page.goto('/sui/demos'); await attach()
for (const id of ['link-signals', 'link-conditional', 'link-iteration', 'link-match', 'link-lifecycle']) {
  await run(`/demos has ${id}`, () => expectCount(`[data-test="${id}"]`, 1))
}

console.log('\n[signals-and-derived]')
await page.goto('/sui/demos/signals-and-derived'); await attach()
await run('initial count = 0', () => expectText('[data-test="count"]', '0'))
await run('initial double = 0', () => expectText('[data-test="double"]', '0'))
await page.click('[data-test="inc"]')
await page.click('[data-test="inc"]')
await run('after 2× inc, count = 2', () => expectText('[data-test="count"]', '2'))
await run('after 2× inc, double = 4', () => expectText('[data-test="double"]', '4'))
await page.click('[data-test="dec"]')
await run('after dec, count = 1', () => expectText('[data-test="count"]', '1'))
await page.click('[data-test="reset"]')
await run('after reset, count = 0', () => expectText('[data-test="count"]', '0'))

console.log('\n[conditional-render]')
await page.goto('/sui/demos/conditional-render'); await attach()
await run('panel-off visible initially', () => expectCount('[data-test="panel-off"]', 1))
await run('panel-on hidden initially', () => expectCount('[data-test="panel-on"]', 0))
await page.check('[data-test="toggle"]')
await run('panel-on visible after toggle', () => expectCount('[data-test="panel-on"]', 1))
await run('panel-off hidden after toggle', () => expectCount('[data-test="panel-off"]', 0))

console.log('\n[iteration]')
await page.goto('/sui/demos/iteration'); await attach()
await run('starts with 2 items', () => expectCount('[data-test="todos"] > li', 2))
await page.fill('[data-test="todo-input"]', 'a third one')
await page.click('[data-test="todo-add"]')
await run('add → 3 items', () => expectCount('[data-test="todos"] > li', 3))
await page.locator('[data-test="todos"] > li').first().locator('[data-test="remove"]').click()
await run('remove first → 2 items', () => expectCount('[data-test="todos"] > li', 2))

console.log('\n[match]')
await page.goto('/sui/demos/match'); await attach()
await run('initial alert: info', async () => {
  const t = await page.locator('[data-test="alert"]').textContent()
  if (!t?.includes('Heads up.')) throw new Error(`got: ${t?.slice(0, 60)}`)
})
await page.click('[data-test="tone-success"]')
await run('after success: All good', async () => {
  const t = await page.locator('[data-test="alert"]').textContent()
  if (!t?.includes('All good.')) throw new Error(`got: ${t?.slice(0, 60)}`)
})
await page.click('[data-test="tone-destructive"]')
await run('after destructive: Stop', async () => {
  const t = await page.locator('[data-test="alert"]').textContent()
  if (!t?.includes('Stop.')) throw new Error(`got: ${t?.slice(0, 60)}`)
})

console.log('\n[lifecycle]')
await page.goto('/sui/demos/lifecycle'); await attach()
await run('mount line in log', async () => {
  const t = await page.locator('[data-test="log"]').textContent()
  if (!t?.includes('mount')) throw new Error(`log was: ${t?.slice(0, 80)}`)
})
await run('elapsed advances', async () => {
  await page.waitForFunction(() => Number(document.querySelector('[data-test="elapsed"]')?.textContent) >= 1, null, { timeout: 4_000 })
})

await browser.close()

console.log(`\n=== ${ok.length} passed, ${failures.length} failed ===`)
process.exit(failures.length === 0 ? 0 : 1)
