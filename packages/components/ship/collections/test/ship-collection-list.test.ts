import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-collection-list/ship-collection-list.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

// ship-collection-list subscribes to ship.collections.list() via live().
// A mutable impl lets each test pick the resolved value.
let listImpl: () => unknown
beforeEach(() => {
  listImpl = () => ({ items: [] })
  setShipClient({ collections: { list: () => listImpl() } } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLElement | null

const COLLECTIONS = {
  items: [
    { id: 'notes',    name: 'Notes',    field_count: 4, fields: [], builtin: false, item_count: 3 },
    { id: 'tasks',    name: 'Tasks',    field_count: 5, fields: [], builtin: false, item_count: 5 },
    { id: 'feedback', name: 'Feedback', field_count: 1, fields: [], builtin: true,  item_count: 8 },
  ],
}

describe('<ship-collection-list>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-collection-list')).toBeDefined()
  })

  it('renders a row per collection with the name and count', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-list')
    await flush()
    const rows = shadow(el).querySelectorAll('[data-test="row"]')
    expect(rows.length).toBe(3)
    expect(rows[0].textContent).toContain('Notes')
    expect(rows[1].textContent).toContain('Tasks')
    expect(rows[2].textContent).toContain('Feedback')
  })

  it('flags system collections with the system pill', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-list')
    await flush()
    const systemPills = shadow(el).querySelectorAll('[data-test="system"]')
    expect(systemPills.length).toBe(1)
    expect(systemPills[0].textContent).toBe('system')
  })

  it('emits ship-collection-select { id, system } on row click', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-list')
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-collection-select', onSelect)
    const row = shadow(el).querySelector('[data-id="feedback"]') as HTMLElement
    row.click()
    expect(onSelect).toHaveBeenCalledTimes(1)
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ id: 'feedback', system: true })
  })

  it('marks the active row with aria-selected', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-list', { active: 'tasks' })
    await flush()
    const tasks = shadow(el).querySelector('[data-id="tasks"]') as HTMLElement
    expect(tasks.getAttribute('aria-selected')).toBe('true')
  })

  it('shows the skeleton loading boundary while pending', async () => {
    listImpl = () => new Promise(() => {})
    const { el } = mount(reg, 'ship-collection-list')
    await flush()
    expect(q(el, 'skeleton')).not.toBeNull()
  })

  it('shows the error boundary when list rejects', async () => {
    listImpl = () => Promise.reject(new Error('boom'))
    const { el } = mount(reg, 'ship-collection-list')
    await flush()
    await flush()
    const errState = shadow(el).querySelector('sui-error-state')
    expect(errState).not.toBeNull()
    expect(errState?.textContent).toContain('boom')
  })
})
