// @vitest-environment happy-dom
// TESTS-FIRST — Slice 2: the <ship-code-editor> VIEW + controller.
//
// The component wraps the headless CM6 engine (Slice 1, GREEN). The engine owns
// its DOM inside the bind:this div — these specs assert the wiring: a .cm-editor
// mounts in the shadow root, the `value` prop seeds the doc, edits emit a
// debounced ship-code-change, readonly blocks edits, a `language` attr change
// reconfigures the language compartment, and Mod-s emits ship-code-save.
//
// Harness: renderComponent compiles ship-code-editor.sui under the tag and
// registers the custom element (mirrors the agents pack). happy-dom tolerates
// CM6's modern scoped CSS (jsdom 25 throws on it — same reason sui-calendar
// pins happy-dom).
//
// Authority: build-spec §"Slice 2", §"Tests".

import { readFileSync } from 'node:fs'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import editorController from '../src/ship-code-editor/ship-code-editor.controller'
import type { CodeEditor } from '../src/engine/types.js'

const SOURCE = readFileSync(
  new URL('../src/ship-code-editor/ship-code-editor.sui', import.meta.url),
  'utf8',
)

const shadow = (el: HTMLElement): ShadowRoot => el.shadowRoot as ShadowRoot
const engineOf = (el: HTMLElement): CodeEditor =>
  (el as HTMLElement & { editor: CodeEditor }).editor

const live: Array<{ unmount(): void }> = []
afterEach(() => {
  vi.useRealTimers()
  for (const v of live.splice(0)) {
    try {
      v.unmount()
    } catch {
      /* best-effort */
    }
  }
})

function mount(attrs?: Record<string, string | boolean>) {
  const v = renderComponent(SOURCE, 'ship-code-editor', {
    controller: editorController,
    ...(attrs ? { attrs } : {}),
  })
  live.push(v)
  return v
}

describe('<ship-code-editor> — registration + mount', () => {
  it('registers the custom element', () => {
    mount()
    expect(customElements.get('ship-code-editor')).toBeDefined()
  })

  it('mounts a .cm-editor into the shadow root', async () => {
    const v = mount()
    await v.flush()
    expect(shadow(v.el).querySelector('.cm-editor')).not.toBeNull()
  })

  it('the editor lives inside the bound .editor div (engine-owned DOM)', async () => {
    const v = mount()
    await v.flush()
    const host = shadow(v.el).querySelector('.editor')!
    expect(host.querySelector('.cm-editor')).not.toBeNull()
  })
})

describe('<ship-code-editor> — value seeding', () => {
  it('seeds the document from the value prop', async () => {
    const v = mount({ value: 'const a = 1' })
    await v.flush()
    expect(engineOf(v.el).value()).toBe('const a = 1')
  })
})

describe('<ship-code-editor> — change events', () => {
  it('emits a debounced ship-code-change on a doc edit', async () => {
    vi.useFakeTimers()
    const v = mount({ value: 'a' })
    await Promise.resolve()
    const emitted: string[] = []
    v.el.addEventListener('ship-code-change', (e) => {
      emitted.push((e as CustomEvent<{ value: string }>).detail.value)
    })
    // Drive a real document edit through the engine.
    engineOf(v.el).setValue('a = 2')
    // Debounced 200ms — nothing yet, then flush the timer.
    expect(emitted).toEqual([])
    vi.advanceTimersByTime(250)
    expect(emitted).toEqual(['a = 2'])
  })

  it('does NOT emit on construction (only on real edits)', async () => {
    vi.useFakeTimers()
    const v = mount({ value: 'seed' })
    const emitted: unknown[] = []
    v.el.addEventListener('ship-code-change', () => emitted.push(1))
    vi.advanceTimersByTime(250)
    expect(emitted).toEqual([])
  })
})

describe('<ship-code-editor> — readonly', () => {
  it('blocks edits when the readonly attr is set', async () => {
    const v = mount({ value: 'frozen', readonly: true })
    await v.flush()
    expect(engineOf(v.el).isReadonly()).toBe(true)
  })

  it('toggles editability when the readonly attr changes (MutationObserver)', async () => {
    const v = mount({ value: 'x' })
    await v.flush()
    expect(engineOf(v.el).isReadonly()).toBe(false)
    v.el.setAttribute('readonly', '')
    await v.flush()
    expect(engineOf(v.el).isReadonly()).toBe(true)
  })
})

describe('<ship-code-editor> — language switch', () => {
  it('reconfigures on a language attr change (same engine, no rebuild)', async () => {
    const v = mount({ value: 'x = 1' })
    await v.flush()
    const before = engineOf(v.el)
    v.el.setAttribute('language', 'python')
    await v.flush()
    // Same engine instance — the language compartment reconfigured in place.
    expect(engineOf(v.el)).toBe(before)
    expect(shadow(v.el).querySelector('.cm-editor')).not.toBeNull()
  })
})

describe('<ship-code-editor> — save', () => {
  it('emits ship-code-save on Mod-s', async () => {
    const v = mount({ value: 'save me' })
    await v.flush()
    const saved: string[] = []
    v.el.addEventListener('ship-code-save', (e) => {
      saved.push((e as CustomEvent<{ value: string }>).detail.value)
    })
    const content = shadow(v.el).querySelector('.cm-content') as HTMLElement
    content.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 's',
        ctrlKey: true,
        bubbles: true,
      }),
    )
    expect(saved).toEqual(['save me'])
  })
})

describe('<ship-code-editor> — CSS token discipline (REVIEW gate, asserted)', () => {
  it('the component CSS contains no color-mix/oklch/hex/var-fallback literals', () => {
    const styleMatch = /<style>([\s\S]*?)<\/style>/.exec(SOURCE)
    const css = styleMatch ? styleMatch[1]! : ''
    expect(css).not.toMatch(/color-mix/)
    expect(css).not.toMatch(/oklch/)
    expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    // var(--token, fallback) — a comma inside a var() is a fallback smell.
    expect(css).not.toMatch(/var\(\s*--[\w-]+\s*,/)
  })
})
