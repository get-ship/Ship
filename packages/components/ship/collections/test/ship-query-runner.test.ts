import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-query-runner/ship-query-runner.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// ship-query-runner reads ship.collections.state for the schema and
// ship.collections.items for the result rows. The runner does not
// dispatch a real query in v1 (see KNOWN_GAPS.md §2) — it surfaces
// the items() rows.
let stateImpl: () => unknown
let itemsImpl: () => unknown
beforeEach(() => {
  stateImpl = () => ({
    schema: { id: 'notes', name: 'Notes', fields: [
      { name: 'title', type: 'text' },
      { name: 'pinned', type: 'bool' },
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
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLElement | null
const table = (el: HTMLElement) =>
  shadow(el).querySelector('sui-table') as (HTMLElement & {
    rows?: Array<{ id: string; [k: string]: unknown }>
    columns?: Array<{ field: string; label: string }>
  }) | null

describe('<ship-query-runner>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-query-runner')).toBeDefined()
  })

  it('renders a sui-table once the results resolve', async () => {
    const { el } = mount(reg, 'ship-query-runner', { collection: 'notes' })
    await flush()
    expect(table(el)).not.toBeNull()
  })

  it('passes the items as the table rows', async () => {
    const { el } = mount(reg, 'ship-query-runner', { collection: 'notes' })
    await flush()
    const rows = table(el)?.rows
    expect(rows?.length).toBe(2)
    expect((rows?.[0] as { title: string }).title).toBe('A')
  })

  it('shows the row count in the meta pill', async () => {
    const { el } = mount(reg, 'ship-query-runner', { collection: 'notes' })
    await flush()
    expect(q(el, 'count')?.textContent).toBe('2 rows')
  })

  it('shows the empty state when items() returns no rows', async () => {
    itemsImpl = () => ({ items: [], total_items: 0 })
    const { el } = mount(reg, 'ship-query-runner', { collection: 'notes' })
    await flush()
    expect(q(el, 'empty')).not.toBeNull()
    expect(table(el)).toBeNull()
  })

  it('shows the error boundary when items() rejects', async () => {
    itemsImpl = () => Promise.reject(new Error('query failed'))
    const { el } = mount(reg, 'ship-query-runner', { collection: 'notes' })
    await flush()
    await flush()
    const errState = shadow(el).querySelector('[data-test="error"]') ?? shadow(el).querySelector('sui-error-state')
    expect(errState).not.toBeNull()
    expect(errState?.textContent).toContain('query failed')
  })
})
