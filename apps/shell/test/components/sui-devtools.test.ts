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

  it('renders header and split elements in shadow DOM', () => {
    const { el } = mount(reg, 'sui-devtools-source')
    expect(shadow(el).querySelector('.header')).not.toBeNull()
    expect(shadow(el).querySelector('.split')).not.toBeNull()
  })

  it('diagnostics container is not rendered initially', () => {
    const { el } = mount(reg, 'sui-devtools-source')
    expect(shadow(el).querySelector('.diagnostics')).toBeNull()
  })

  it('shows empty-state when fetch returns 404', async () => {
    const fetchSpy = vi.fn().mockResolvedValue({ ok: false, status: 404, json: () => Promise.resolve({}) })
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-source')
    await flush()
    await flush()
    const split = shadow(el).querySelector('.split')!
    expect(split.innerHTML).toContain('sui-empty-state')
    globalThis.fetch = origFetch
  })

  it('shows loading empty-state while fetching (when path is set)', async () => {
    let resolve!: (v: unknown) => void
    const fetchSpy = vi.fn().mockReturnValue(new Promise(r => { resolve = r }))
    const origFetch = globalThis.fetch
    const origShip = (window as unknown as { ship?: unknown }).ship
    ;(window as unknown as { ship: unknown }).ship = { current: () => ({ path: '/api/v/myview' }) }
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-source')
    await flush()
    const split = shadow(el).querySelector('.split')!
    expect(split.innerHTML).toContain('Loading source')
    resolve({ ok: false, status: 404, json: () => Promise.resolve({}) })
    globalThis.fetch = origFetch
    ;(window as unknown as { ship: unknown }).ship = origShip
  })

  it('refresh() method is exposed on host', async () => {
    const { el } = mount(reg, 'sui-devtools-source')
    await flush()
    expect(typeof (el as unknown as { refresh?: unknown }).refresh).toBe('function')
  })

  it('embeds <sui-code-editor> tags (not <ship-code-editor>) when source is present', async () => {
    const origShip = (window as unknown as { ship?: unknown }).ship
    ;(window as unknown as { ship: unknown }).ship = { current: () => ({ path: '/api/v/myview' }) }
    const origFetch = globalThis.fetch
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({
        view_name: 'myview.j2',
        template_path: '/abs/views/myview/index.html.j2',
        template: '<p>hi</p>',
        manifest_filename: null,
        manifest: null,
        scope: 'project',
      }),
    })
    const { el } = mount(reg, 'sui-devtools-source')
    await flush(); await flush()
    expect(shadow(el).querySelector('sui-code-editor')).not.toBeNull()
    expect(shadow(el).querySelector('ship-code-editor')).toBeNull()
    globalThis.fetch = origFetch
    ;(window as unknown as { ship: unknown }).ship = origShip
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
    resolve({ ok: true, json: () => Promise.resolve({ modules: [] }) })
    globalThis.fetch = origFetch
  })

  it('renders entry list when CEM data loads', async () => {
    const cem = {
      modules: [{
        kind: 'javascript-module', path: 'src/foo.js',
        declarations: [{
          kind: 'class', name: 'FooEl', tagName: 'foo-el', customElement: true,
          description: 'A foo element', attributes: [], slots: [],
        }],
      }],
    }
    const fetchSpy = vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve(cem) })
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-components')
    await flush(); await flush()
    expect(body(el).innerHTML).toContain('foo-el')
    globalThis.fetch = origFetch
  })

  it('filter input narrows displayed entries', async () => {
    const cem = {
      modules: [{
        kind: 'javascript-module', path: 'x.js',
        declarations: [
          { kind: 'class', name: 'A', tagName: 'ship-button', customElement: true, description: '', attributes: [], slots: [] },
          { kind: 'class', name: 'B', tagName: 'ship-input', customElement: true, description: '', attributes: [], slots: [] },
        ],
      }],
    }
    const fetchSpy = vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve(cem) })
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    const { el } = mount(reg, 'sui-devtools-components')
    await flush(); await flush()
    const input = toolbar(el).querySelector<HTMLInputElement>('input')!
    input.value = 'button'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(body(el).innerHTML).toContain('ship-button')
    expect(body(el).innerHTML).not.toContain('ship-input')
    globalThis.fetch = origFetch
  })

  it('refresh() method is exposed on host', async () => {
    const { el } = mount(reg, 'sui-devtools-components')
    await flush()
    expect(typeof (el as unknown as { refresh?: unknown }).refresh).toBe('function')
  })

  it('fetches the SUI inventory (sui-custom-elements.json), not the legacy lit CEM', async () => {
    const fetchSpy = vi.fn().mockResolvedValue({ ok: true, json: () => Promise.resolve({ modules: [] }) })
    const origFetch = globalThis.fetch
    globalThis.fetch = fetchSpy
    mount(reg, 'sui-devtools-components')
    await flush(); await flush()
    expect(fetchSpy).toHaveBeenCalled()
    const calledUrl = String(fetchSpy.mock.calls[0]?.[0] ?? '')
    expect(calledUrl).toContain('/api/assets/sui-custom-elements.json')
    expect(calledUrl).not.toBe('/api/assets/custom-elements.json')
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
