import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-item-detail/ship-item-detail.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// ship-item-detail calls ship.collections.get_item (Promise) and reads
// the schema via ship.collections.state to label typed rows.
let getItemImpl: () => unknown
let stateImpl: () => unknown
beforeEach(() => {
  getItemImpl = () => ({ ok: true, data: { id: 'note-1', data: { title: 'A', pinned: true } } })
  stateImpl = () => ({
    schema: { id: 'notes', name: 'Notes', fields: [
      { name: 'title', type: 'text', field_type: 'text', required: true, values: null, default: null },
      { name: 'pinned', type: 'bool', field_type: 'bool', required: false, values: null, default: false },
    ], builtin: false },
    total_items: 1,
  })
  setShipClient({
    collections: {
      get_item: () => getItemImpl(),
      state: () => stateImpl(),
    },
  } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLElement | null

describe('<ship-item-detail>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-item-detail')).toBeDefined()
  })

  it('renders the item id header once the get_item resolves', async () => {
    const { el } = mount(reg, 'ship-item-detail', { collection: 'notes', id: 'note-1' })
    await flush()
    expect(q(el, 'item-id')?.textContent).toBe('note-1')
  })

  it('renders one row per schema field with the typed value', async () => {
    const { el } = mount(reg, 'ship-item-detail', { collection: 'notes', id: 'note-1' })
    await flush()
    const rows = q(el, 'rows')
    expect(rows?.querySelectorAll('dt').length).toBe(2)
    expect(rows?.textContent).toContain('title')
    expect(rows?.textContent).toContain('A')
    expect(rows?.textContent).toContain('pinned')
    expect(rows?.textContent).toContain('true')
  })

  it('reflects the view attribute (typed/raw) and switches on click', async () => {
    const { el } = mount(reg, 'ship-item-detail', { collection: 'notes', id: 'note-1' })
    await flush()
    expect(el.getAttribute('view')).toBe('typed')
    ;(q(el, 'view-raw') as HTMLElement).click()
    expect(el.getAttribute('view')).toBe('raw')
    ;(q(el, 'view-typed') as HTMLElement).click()
    expect(el.getAttribute('view')).toBe('typed')
  })

  it('shows the raw JSON view as a sui-json-view when view=raw', async () => {
    const { el } = mount(reg, 'ship-item-detail', { collection: 'notes', id: 'note-1', view: 'raw' })
    await flush()
    expect(q(el, 'raw')?.querySelector('sui-json-view')).not.toBeNull()
    expect(q(el, 'rows')).toBeNull()
  })

  it('emits ship-item-edit { collection, id } on the Edit button', async () => {
    const { el } = mount(reg, 'ship-item-detail', { collection: 'notes', id: 'note-1' })
    await flush()
    const onEdit = vi.fn()
    el.addEventListener('ship-item-edit', onEdit)
    ;(q(el, 'edit') as HTMLElement).click()
    expect(onEdit).toHaveBeenCalledTimes(1)
    expect((onEdit.mock.calls[0][0] as CustomEvent).detail).toEqual({ collection: 'notes', id: 'note-1' })
  })

  it('shows the error boundary when get_item rejects', async () => {
    getItemImpl = () => Promise.reject(new Error('boom'))
    const { el } = mount(reg, 'ship-item-detail', { collection: 'notes', id: 'note-1' })
    await flush()
    await flush()
    const errState = shadow(el).querySelector('sui-error-state')
    expect(errState).not.toBeNull()
    expect(errState?.textContent).toContain('boom')
  })
})
