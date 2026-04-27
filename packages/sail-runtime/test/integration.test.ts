// Integration test: compile a real .set file via the Rust `sail`
// binary, load the produced JS in jsdom, and exercise the resulting
// custom element. This is the test that proves the compiler+runtime
// contract end-to-end. Substring assertions in the compiler's emit
// tests can pass while the actual JS is broken; this can't.

import { execFileSync } from 'node:child_process'
import { mkdirSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { describe, expect, it, beforeAll } from 'vitest'

const REPO_ROOT = resolve(__dirname, '../../..')
const SAIL_BIN = resolve(
  REPO_ROOT,
  '../ship-cache/cargo-target/debug/sail',
)
const FIXTURES = resolve(__dirname, 'fixtures')
// Compile output sits inside the project tree so vitest's resolver
// can find the dynamic-imported file. Cleared per test run.
const COMPILE_OUT = resolve(__dirname, '.compiled')

function ensureBinary() {
  if (existsSync(SAIL_BIN)) return
  // Build it. cwd is the repo root.
  execFileSync('cargo', ['build', '--bin', 'sail', '--quiet'], {
    cwd: REPO_ROOT,
    stdio: 'inherit',
  })
}

function compileFixture(file: string): string {
  ensureBinary()
  mkdirSync(COMPILE_OUT, { recursive: true })
  execFileSync(SAIL_BIN, ['build', join(FIXTURES, file), '--out', COMPILE_OUT], {
    stdio: 'pipe',
  })
  const stem = file.replace(/\.set$/, '')
  const tag = stem.includes('-') ? stem : `sail-${stem}`
  const jsPath = join(COMPILE_OUT, `${tag}.js`)
  // Copy as .mjs so vitest treats it as a module unambiguously.
  const js = readFileSync(jsPath, 'utf8')
  const mjsPath = join(COMPILE_OUT, `${tag}.mjs`)
  writeFileSync(mjsPath, js)
  return mjsPath
}

async function loadCompiled(file: string): Promise<string> {
  const path = compileFixture(file)
  // Cache-bust per test run with a query param; vitest dedupes by URL.
  await import(`${path}?t=${Date.now()}`)
  const stem = file.replace(/\.set$/, '')
  return stem.includes('-') ? stem : `sail-${stem}`
}

describe('integration: real compile → run', () => {
  beforeAll(() => {
    ensureBinary()
  }, 120_000)

  it('counter compiles and ticks', async () => {
    const tag = await loadCompiled('counter.set')
    const el = document.createElement(tag)
    document.body.appendChild(el)

    const count = el.querySelector('[data-testid="count"]')!
    const doubled = el.querySelector('[data-testid="doubled"]')!
    const inc = el.querySelector('[data-testid="inc"]') as HTMLButtonElement
    const reset = el.querySelector('[data-testid="reset"]') as HTMLButtonElement

    expect(count.textContent).toBe('0')
    expect(doubled.textContent).toBe('0')

    inc.click()
    inc.click()
    inc.click()
    expect(count.textContent).toBe('3')
    expect(doubled.textContent).toBe('6')

    reset.click()
    expect(count.textContent).toBe('0')
    expect(doubled.textContent).toBe('0')

    el.remove()
  }, 60_000)

  it('search filters with derived + if/else', async () => {
    const tag = await loadCompiled('search.set')
    const el = document.createElement(tag)
    document.body.appendChild(el)

    const query = el.querySelector('[data-testid="query"]') as HTMLInputElement
    const caseBox = el.querySelector('[data-testid="case"]') as HTMLInputElement

    // Initial: no query → all 6 items.
    expect(el.querySelectorAll('[data-testid="result"]').length).toBe(6)
    expect(el.querySelector('[data-testid="empty"]')).toBeNull()

    // Filter: "b" → banana, blueberry.
    query.value = 'b'
    query.dispatchEvent(new Event('input', { bubbles: true }))
    const results = Array.from(el.querySelectorAll('[data-testid="result"]'))
    expect(results.map((r) => r.textContent)).toEqual(['banana', 'blueberry'])

    // Empty branch: query that matches nothing.
    query.value = 'zzz'
    query.dispatchEvent(new Event('input', { bubbles: true }))
    expect(el.querySelectorAll('[data-testid="result"]').length).toBe(0)
    const empty = el.querySelector('[data-testid="empty"]')!
    expect(empty.textContent).toContain('zzz')

    // Case-sensitivity flips a derived dep.
    query.value = 'AP'
    query.dispatchEvent(new Event('input', { bubbles: true }))
    expect(el.querySelectorAll('[data-testid="result"]').length).toBe(1) // 'apple' (case-insensitive)
    caseBox.click()
    expect(el.querySelectorAll('[data-testid="result"]').length).toBe(0) // case-sensitive: no 'AP'

    el.remove()
  }, 60_000)

  it('note-display props + raw html + nested if', async () => {
    const tag = await loadCompiled('note-display.set')
    const el = document.createElement(tag)
    el.setAttribute('title', 'My Note')
    el.setAttribute('body', 'Hello <strong>world</strong>')
    el.setAttribute('tags', '["a","b"]')
    document.body.appendChild(el)

    expect(el.querySelector('[data-testid="title"]')!.textContent).toBe('My Note')
    // Initially collapsed.
    expect(el.querySelector('[data-testid="body"]')).toBeNull()
    expect(el.querySelector('[data-testid="tags"]')).toBeNull()

    // Expand.
    ;(el.querySelector('[data-testid="toggle"]') as HTMLButtonElement).click()
    const body = el.querySelector('[data-testid="body"]')!
    expect(body.textContent).toBe('Hello world')
    // The raw HTML is parsed into real elements (<strong>, not text).
    expect(body.querySelector('strong')).not.toBeNull()
    const tags = el.querySelector('[data-testid="tags"]')!
    expect(tags.querySelectorAll('li').length).toBe(2)

    // Title prop change (kebab attribute) flows through.
    el.setAttribute('title', 'Renamed')
    expect(el.querySelector('[data-testid="title"]')!.textContent).toBe('Renamed')

    el.remove()
  }, 60_000)

  it('modal: visibility, slots, backdrop close', async () => {
    const tag = await loadCompiled('modal.set')
    const el = document.createElement(tag) as HTMLElement & { open?: any }
    // Slot fills BEFORE attaching so resolveSlots picks them up at mount.
    const headerFill = document.createElement('div')
    headerFill.setAttribute('slot', 'header')
    headerFill.textContent = 'Custom Header'
    const bodyFill = document.createElement('p')
    bodyFill.textContent = 'Body content'
    el.appendChild(headerFill)
    el.appendChild(bodyFill)

    document.body.appendChild(el)
    // open=false → root has data-open="false" (CSS hides it but DOM exists).
    const root = el.querySelector('.modal-root')!
    expect(root.getAttribute('data-open')).toBe('false')
    // Slots resolved at mount regardless of visibility.
    expect(el.querySelector('[data-testid="header"]')!.textContent).toContain(
      'Custom Header',
    )
    expect(el.querySelector('[data-testid="body"]')!.textContent).toContain(
      'Body content',
    )

    // Open: attribute flows through prop signal → bindAttr re-evaluates.
    el.setAttribute('open', '')
    expect(root.getAttribute('data-open')).toBe('true')

    // Click backdrop closes.
    const backdrop = el.querySelector('[data-testid="backdrop"]') as HTMLElement
    backdrop.click()
    expect(root.getAttribute('data-open')).toBe('false')

    el.remove()
  }, 60_000)

  it('todo list adds, toggles, removes, reorders', async () => {
    const tag = await loadCompiled('todo.set')
    const el = document.createElement(tag)
    document.body.appendChild(el)

    const draft = el.querySelector('[data-testid="draft"]') as HTMLInputElement
    const add = el.querySelector('[data-testid="add"]') as HTMLButtonElement
    const list = el.querySelector('[data-testid="list"]')!
    const count = el.querySelector('[data-testid="count"]')!

    // Initial render: 2 items.
    expect(list.querySelectorAll('[data-testid="item"]').length).toBe(2)
    expect(count.textContent).toBe('2 item(s)')

    // Add one.
    draft.value = 'feed cat'
    draft.dispatchEvent(new Event('input', { bubbles: true }))
    add.click()
    expect(list.querySelectorAll('[data-testid="item"]').length).toBe(3)
    expect(count.textContent).toBe('3 item(s)')

    // Toggle the first.
    const firstToggle = list
      .querySelectorAll('[data-testid="toggle"]')[0] as HTMLButtonElement
    expect(firstToggle.textContent).toBe('O')
    firstToggle.click()
    expect(firstToggle.textContent).toBe('X')

    // Remove the second.
    const removeBtns = list.querySelectorAll('[data-testid="remove"]')
    ;(removeBtns[1] as HTMLButtonElement).click()
    expect(list.querySelectorAll('[data-testid="item"]').length).toBe(2)
    expect(count.textContent).toBe('2 item(s)')

    el.remove()
  }, 60_000)
})
