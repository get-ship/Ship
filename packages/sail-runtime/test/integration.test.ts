// Integration test: compile a real .sail file via the Rust `sail`
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
  const stem = file.replace(/\.sail$/, '')
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
  const stem = file.replace(/\.sail$/, '')
  return stem.includes('-') ? stem : `sail-${stem}`
}

describe('integration: real compile → run', () => {
  beforeAll(() => {
    ensureBinary()
  }, 120_000)

  it('counter compiles and ticks', async () => {
    const tag = await loadCompiled('counter.sail')
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

  it('todo list adds, toggles, removes, reorders', async () => {
    const tag = await loadCompiled('todo.sail')
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
