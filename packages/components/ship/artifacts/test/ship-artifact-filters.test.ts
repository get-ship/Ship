import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-artifact-filters/ship-artifact-filters.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot ?? el

const findByTest = (root: ParentNode, sel: string): Element | null =>
  root.querySelector(`[data-test="${sel}"]`)

function fireSui(el: HTMLElement, testAttr: string, detail: Record<string, unknown>, eventName: string): void {
  const el2 = findByTest(shadow(el), testAttr) as HTMLElement | null
  expect(el2).not.toBeNull()
  el2!.dispatchEvent(new CustomEvent(eventName, { detail, bubbles: true, composed: true }))
}

// Returns a promise that resolves with the next matching event. Attach
// the listener FIRST, then drive the action — no real-clock timer.
function nextEvent(target: EventTarget, name: string): Promise<CustomEvent> {
  const { promise, resolve } = Promise.withResolvers<CustomEvent>()
  target.addEventListener(name, (e) => resolve(e as CustomEvent), { once: true })
  return promise
}

describe('<ship-artifact-filters>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-artifact-filters')).toBeDefined()
  })

  it('renders a search input plus four selects', async () => {
    const { el } = mount(reg, 'ship-artifact-filters')
    await flush()
    const root = shadow(el)
    expect(root.querySelector('sail-input[type="search"]')).not.toBeNull()
    expect(root.querySelectorAll('sail-select').length).toBe(4)
  })

  it('emits ship-artifact-filter { kind: "search", value } on search input', async () => {
    const { el } = mount(reg, 'ship-artifact-filters')
    await flush()
    const eventPromise = nextEvent(el, 'ship-artifact-filter')
    fireSui(el, 'filter-search', { value: 'arch' }, 'sail-input')
    const evt = await eventPromise
    expect((evt as CustomEvent<{ kind: string; value: string }>).detail).toEqual({ kind: 'search', value: 'arch' })
  })

  it('emits ship-artifact-filter { kind: "mime", value } on mime change', async () => {
    const { el } = mount(reg, 'ship-artifact-filters')
    await flush()
    const eventPromise = nextEvent(el, 'ship-artifact-filter')
    fireSui(el, 'filter-mime', { value: 'image' }, 'sail-change')
    const evt = await eventPromise
    expect((evt as CustomEvent<{ kind: string; value: string }>).detail).toEqual({ kind: 'mime', value: 'image' })
  })

  it('emits ship-artifact-filter { kind: "age", value } on age change', async () => {
    const { el } = mount(reg, 'ship-artifact-filters')
    await flush()
    const eventPromise = nextEvent(el, 'ship-artifact-filter')
    fireSui(el, 'filter-age', { value: '24h' }, 'sail-change')
    const evt = await eventPromise
    expect((evt as CustomEvent<{ kind: string; value: string }>).detail).toEqual({ kind: 'age', value: '24h' })
  })

  it('renders option rows from the prop:collections JSON', async () => {
    const { el } = mount(reg, 'ship-artifact-filters', {
      collections: JSON.stringify([{ id: 'notes', name: 'Notes' }, { id: 'screens', name: null }]),
    })
    await flush()
    const select = findByTest(shadow(el), 'filter-collection') as HTMLElement
    const opts = select?.querySelectorAll('option')
    // 1 default + 2 from JSON
    expect(opts?.length).toBe(3)
    expect(opts?.[1]?.getAttribute('value')).toBe('notes')
    expect(opts?.[1]?.textContent).toContain('Notes')
    expect(opts?.[2]?.getAttribute('value')).toBe('screens')
    expect(opts?.[2]?.textContent).toContain('screens') // falls back to id when name is null
  })
})
