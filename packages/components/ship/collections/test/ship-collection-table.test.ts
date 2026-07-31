import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-collection-table/ship-collection-table.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// ship-collection-table reads ship.collections.state for the schema (columns)
// and ship.collections.items for rows. Mutable impls let each test pick
// the resolved values.
let stateImpl: () => unknown
let itemsImpl: () => unknown
beforeEach(() => {
  stateImpl = () => ({
    schema: { id: 'notes', name: 'Notes', fields: [
      { name: 'title', type: 'text', field_type: 'text', required: true, values: null, default: null },
      { name: 'pinned', type: 'bool', field_type: 'bool', required: false, values: null, default: false },
    ], builtin: false },
    total_items: 3,
  })
  itemsImpl = () => ({
    items: [
      { id: 'note-1', data: { title: 'A', pinned: true } },
      { id: 'note-2', data: { title: 'B', pinned: false } },
    ],
    total_items: 3,
  })
  setShipClient({
    collections: {
      state: () => stateImpl(),
      items: () => itemsImpl(),
    },
  } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const table = (el: HTMLElement) =>
  shadow(el).querySelector('sui-table') as (HTMLElement & {
    rows?: Array<{ id: string; [k: string]: unknown }>
    columns?: Array<{ field: string; label: string }>
  }) | null

describe('<ship-collection-table>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-collection-table')).toBeDefined()
  })

  it('renders a sui-table once state + items resolve', async () => {
    const { el } = mount(reg, 'ship-collection-table', { collection: 'notes' })
    await flush()
    expect(table(el)).not.toBeNull()
  })

  it('derives columns from the schema (id + each field)', async () => {
    const { el } = mount(reg, 'ship-collection-table', { collection: 'notes' })
    await flush()
    const cols = table(el)?.columns
    expect(cols?.map((c) => c.field)).toEqual(['id', 'title', 'pinned'])
  })

  it('flattens item envelopes into table rows with the field values', async () => {
    const { el } = mount(reg, 'ship-collection-table', { collection: 'notes' })
    await flush()
    const rows = table(el)?.rows
    expect(rows?.length).toBe(2)
    expect((rows?.[0] as { id: string }).id).toBe('note-1')
    expect((rows?.[0] as { title: string }).title).toBe('A')
    expect((rows?.[0] as { pinned: boolean }).pinned).toBe(true)
  })

  it('shows the row count in the footer', async () => {
    const { el } = mount(reg, 'ship-collection-table', { collection: 'notes' })
    await flush()
    const footer = shadow(el).querySelector('[data-test="footer"]')
    expect(footer?.textContent).toContain('2 / 3')
  })

  it('shows the empty state when the collection has no items', async () => {
    itemsImpl = () => ({ items: [], total_items: 0 })
    const { el } = mount(reg, 'ship-collection-table', { collection: 'empty' })
    await flush()
    expect(shadow(el).querySelector('[data-test="empty"]')).not.toBeNull()
    expect(table(el)).toBeNull()
  })

  it('emits ship-collection-row-select { collection, id } on row click', async () => {
    const { el } = mount(reg, 'ship-collection-table', { collection: 'notes' })
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-collection-row-select', onSelect)
    const tr = shadow(el).querySelector('tr[data-row-id="note-2"]') as HTMLElement | null
    expect(tr).not.toBeNull()
    tr?.dispatchEvent(new Event('click', { bubbles: true, composed: true }))
    expect(onSelect).toHaveBeenCalledTimes(1)
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ collection: 'notes', id: 'note-2' })
  })
})
