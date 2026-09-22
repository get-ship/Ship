import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-query-editor/ship-query-editor.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// ship-query-editor subscribes to ship.collections.state for field-name
// suggestions. Inputs dispatch ship-query-change.
let stateImpl: () => unknown
beforeEach(() => {
  stateImpl = () => ({
    schema: { id: 'notes', name: 'Notes', fields: [
      { name: 'title', type: 'text' },
      { name: 'pinned', type: 'bool' },
      { name: 'priority', type: 'integer' },
    ], builtin: false },
    total_items: 0,
  })
  setShipClient({
    collections: {
      state: () => stateImpl(),
    },
  } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLElement | null

describe('<ship-query-editor>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-query-editor')).toBeDefined()
  })

  it('renders the collection label in the header', async () => {
    const { el } = mount(reg, 'ship-query-editor', { collection: 'notes' })
    await flush()
    expect(q(el, 'collection')?.textContent).toBe('notes')
  })

  it('renders the seeded value in the textarea', async () => {
    const { el } = mount(reg, 'ship-query-editor', { collection: 'notes', value: 'SELECT title FROM items' })
    await flush()
    const ta = q(el, 'textarea') as HTMLTextAreaElement
    expect(ta.value).toBe('SELECT title FROM items')
  })

  it('emits ship-query-change { ast, sql } on input', async () => {
    const { el } = mount(reg, 'ship-query-editor', { collection: 'notes', value: '' })
    await flush()
    const onChange = vi.fn()
    el.addEventListener('ship-query-change', onChange)
    const ta = q(el, 'textarea') as HTMLTextAreaElement
    ta.value = 'SELECT pinned'
    ta.dispatchEvent(new Event('input', { bubbles: true }))
    expect(onChange).toHaveBeenCalledTimes(1)
    expect((onChange.mock.calls[0][0] as CustomEvent).detail).toEqual({
      ast: { collection: 'notes', source: 'SELECT pinned' },
      sql: 'SELECT pinned',
    })
  })

  it('shows field-name suggestions when the trailing token matches a field', async () => {
    const { el } = mount(reg, 'ship-query-editor', { collection: 'notes', value: 'SELECT pin' })
    await flush()
    const items = shadow(el).querySelectorAll('[data-test="suggestion"]')
    expect(items.length).toBeGreaterThan(0)
    expect(items[0].textContent).toContain('pinned')
  })

  it('hides suggestions when the trailing token is empty', async () => {
    const { el } = mount(reg, 'ship-query-editor', { collection: 'notes', value: 'SELECT 1 FROM items' })
    await flush()
    expect(q(el, 'suggestions')).toBeNull()
  })
})
