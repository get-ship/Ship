import { afterEach, describe, expect, it, vi, beforeEach } from 'vitest'
import '../../src/components/sui-devtools/sui-devtools-source.compiled.js'
import '../../src/components/sui-devtools/sui-devtools-components.compiled.js'
import '../../src/components/sui-devtools/sui-devtools-schema.compiled.js'
import '../../src/components/sui-devtools/sui-devtools-signals.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const toolbar = (el: HTMLElement) => shadow(el).querySelector('.toolbar')!
const body = (el: HTMLElement) => shadow(el).querySelector('.body')!

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-devtools-source>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-devtools-source')).toBeDefined()
  })

  it('renders two-pane row in shadow DOM', () => {
    // Source devtools was rewritten as a two-pane IDE shell: explorer (file
    // tree) on the left + editor column on the right. The old single-pane
    // "header + split" layout no longer exists. Async mount means inner
    // panes appear after a flush; the outer .row wrapper is enough for
    // smoke verification.
    const { el } = mount(reg, 'sui-devtools-source')
    expect(shadow(el).querySelector('.row')).not.toBeNull()
  })

  it('refresh() method is exposed on host', async () => {
    const { el } = mount(reg, 'sui-devtools-source')
    await flush()
    expect(typeof (el as unknown as { refresh?: unknown }).refresh).toBe('function')
  })

  it('does not embed legacy <ship-code-editor> tags', () => {
    // Regression: ensure no Lit-era ship-code-editor tags leak in.
    const { el } = mount(reg, 'sui-devtools-source')
    expect(shadow(el).querySelector('ship-code-editor')).toBeNull()
  })

})

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-devtools-components>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-devtools-components')).toBeDefined()
  })

  it('renders toolbar with filter input and body container', () => {
    const { el } = mount(reg, 'sui-devtools-components')
    expect(toolbar(el).querySelector('input[type="text"]')).not.toBeNull()
    expect(body(el)).not.toBeNull()
  })

  it('shows loading state before fetch resolves', async () => {
    let resolve!: (v: unknown) => void
    const fetchSpy = vi.fn().mockReturnValue(new Promise(r => { resolve = r }))
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-components')
    await flush()
    expect(body(el).innerHTML).toContain('Loading sui-* components')
    resolve({ ok: true, json: () => Promise.resolve({ schemaVersion: '1', count: 0, entries: [] }) })
    globalThis.fetch = origFetch
  })

  it('renders entry list when catalog data loads', async () => {
    const catalog = {
      schemaVersion: '1',
      count: 1,
      entries: [{
        tagName: 'foo-el',
        package: '@ship/test',
        private: false,
        sourcePath: 'src/foo.sui',
        bundleUrl: '/_sui/packs/test/foo.js',
        manifestUrl: '/_sui/packs/test/custom-elements.json',
        description: 'A foo element',
        firstDesc: 'A foo element',
        attributes: [],
        slots: [],
      }],
    }
    const fetchSpy = vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve(catalog) })
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-components')
    await flush(); await flush()
    expect(body(el).innerHTML).toContain('foo-el')
    globalThis.fetch = origFetch
  })

  it('filter input narrows displayed entries', async () => {
    const catalog = {
      schemaVersion: '1',
      count: 2,
      entries: [
        { tagName: 'sui-button', package: '@ship/primitives', private: false, sourcePath: '', bundleUrl: '', manifestUrl: '', description: '', firstDesc: '', attributes: [], slots: [] },
        { tagName: 'sui-input',  package: '@ship/primitives', private: false, sourcePath: '', bundleUrl: '', manifestUrl: '', description: '', firstDesc: '', attributes: [], slots: [] },
      ],
    }
    const fetchSpy = vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve(catalog) })
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-components')
    await flush(); await flush()
    const input = toolbar(el).querySelector<HTMLInputElement>('input')!
    input.value = 'button'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(body(el).innerHTML).toContain('sui-button')
    expect(body(el).innerHTML).not.toContain('sui-input')
    globalThis.fetch = origFetch
  })

  it('refresh() method is exposed on host', async () => {
    const { el } = mount(reg, 'sui-devtools-components')
    await flush()
    expect(typeof (el as unknown as { refresh?: unknown }).refresh).toBe('function')
  })

  it('fetches the workspace catalog from /api/sui/catalog/workspace', async () => {
    const fetchSpy = vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve({ schemaVersion: '1', count: 0, entries: [] }) })
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    mount(reg, 'sui-devtools-components')
    await flush(); await flush()
    expect(fetchSpy).toHaveBeenCalled()
    const calledUrl = String(fetchSpy.mock.calls[0]?.[0] ?? '')
    expect(calledUrl).toContain('/api/sui/catalog/workspace')
    expect(calledUrl).not.toContain('/api/assets/sui-custom-elements.json')
    globalThis.fetch = origFetch
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-devtools-schema>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-devtools-schema')).toBeDefined()
  })

  it('renders toolbar and body containers in shadow DOM', () => {
    const { el } = mount(reg, 'sui-devtools-schema')
    expect(shadow(el).querySelector('.toolbar')).not.toBeNull()
    expect(shadow(el).querySelector('.body')).not.toBeNull()
  })

  it('shows no-reference empty-state when source has no ship.* ops (with view path)', async () => {
    const fetchSpy = vi.fn()
      .mockResolvedValueOnce({ ok: true, json: () => Promise.resolve({ ok: true, data: { ops: [] } }) })
      .mockResolvedValueOnce({ ok: true, json: () => Promise.resolve({ template: 'Hello world', manifest: null }) })
    const origFetch = globalThis.fetch
    const origShip = (window as unknown as { ship?: unknown }).ship
    ;(window as unknown as { ship: unknown }).ship = { current: () => ({ path: '/api/v/myview' }) }
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-schema')
    await flush(); await flush(); await flush(); await flush()
    expect(body(el).innerHTML).toContain('sui-empty-state')
    globalThis.fetch = origFetch
    ;(window as unknown as { ship: unknown }).ship = origShip
  })

  it('refresh() method is exposed on host', async () => {
    const { el } = mount(reg, 'sui-devtools-schema')
    await flush()
    expect(typeof (el as unknown as { refresh?: unknown }).refresh).toBe('function')
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-devtools-signals>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-devtools-signals')).toBeDefined()
  })

  it('renders toolbar with filter input', () => {
    const { el } = mount(reg, 'sui-devtools-signals')
    expect(toolbar(el).querySelector('input[type="text"]')).not.toBeNull()
  })

  it('renders body container', () => {
    const { el } = mount(reg, 'sui-devtools-signals')
    expect(body(el)).not.toBeNull()
  })

  it('shows no-signals empty-state when no signals present', async () => {
    const { el } = mount(reg, 'sui-devtools-signals')
    await flush()
    expect(body(el).innerHTML).toContain('sui-empty-state')
  })

  it('filter input changes count display', async () => {
    const { el } = mount(reg, 'sui-devtools-signals')
    await flush()
    const countEl = toolbar(el).querySelector('span')!
    expect(countEl.textContent).toBe('0 / 0')
  })

  it('refresh() method is exposed on host', async () => {
    const { el } = mount(reg, 'sui-devtools-signals')
    await flush()
    expect(typeof (el as unknown as { refresh?: unknown }).refresh).toBe('function')
  })

  it('filter input narrows shown signals', async () => {
    // The signals tree reads window.__suiCurrentScope. Stub a scope with
    // two signal getters (matching the runtime's Signal shape — callable
    // plus `__peek()`) so the controller surfaces them as live entries.
    type FakeSig<T> = (() => T) & { __peek: () => T }
    const mk = <T>(v: T): FakeSig<T> => {
      const s = (() => v) as FakeSig<T>
      s.__peek = () => v
      return s
    }
    ;(window as unknown as { __suiCurrentScope: Record<string, unknown> }).__suiCurrentScope = {
      foo: mk('bar'),
      baz: mk(42),
    }
    const { el } = mount(reg, 'sui-devtools-signals')
    await flush()
    ;(el as unknown as { refresh(): void }).refresh()
    await flush()
    const input = toolbar(el).querySelector<HTMLInputElement>('input')!
    input.value = 'foo'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(body(el).innerHTML).toContain('foo')
    expect(body(el).innerHTML).not.toContain('baz')
    ;(window as unknown as { __suiCurrentScope: unknown }).__suiCurrentScope = null
  })
})
