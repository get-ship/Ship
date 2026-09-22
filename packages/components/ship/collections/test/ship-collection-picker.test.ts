import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-collection-picker/ship-collection-picker.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// ship-collection-picker composes a trigger + listbox over ship.collections.list().
// The trigger is closed by default; opening renders the menu; picking a row
// emits ship-collection-pick { id } and closes.
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
    { id: 'notes', name: 'Notes', field_count: 4, fields: [], builtin: false, item_count: 3 },
    { id: 'tasks', name: 'Tasks', field_count: 5, fields: [], builtin: false, item_count: 5 },
  ],
}

describe('<ship-collection-picker>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-collection-picker')).toBeDefined()
  })

  it('shows the placeholder when no collection is active', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-picker', { placeholder: 'Pick one' })
    await flush()
    expect(q(el, 'label')?.textContent).toBe('Pick one')
  })

  it('shows the active collection name in the trigger', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-picker', { active: 'notes' })
    await flush()
    expect(q(el, 'label')?.textContent).toBe('Notes')
  })

  it('renders no menu while closed and a list of rows once opened', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-picker', { active: 'notes' })
    await flush()
    expect(q(el, 'menu')).toBeNull()
    const trigger = q(el, 'trigger') as HTMLElement
    trigger.click()
    await flush()
    const rows = shadow(el).querySelectorAll('[data-test="row"]')
    expect(rows.length).toBe(2)
  })

  it('emits ship-collection-pick { id } when a row is chosen', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-picker', { active: 'notes' })
    await flush()
    const onPick = vi.fn()
    el.addEventListener('ship-collection-pick', onPick)
    const trigger = q(el, 'trigger') as HTMLElement
    trigger.click()
    await flush()
    const row = shadow(el).querySelector('[data-id="tasks"]') as HTMLElement
    row.click()
    expect(onPick).toHaveBeenCalledTimes(1)
    expect((onPick.mock.calls[0][0] as CustomEvent).detail).toEqual({ id: 'tasks' })
  })

  it('marks the active row with class=active in the menu', async () => {
    listImpl = () => COLLECTIONS
    const { el } = mount(reg, 'ship-collection-picker', { active: 'notes' })
    await flush()
    ;(q(el, 'trigger') as HTMLElement).click()
    await flush()
    const notes = shadow(el).querySelector('[data-id="notes"]') as HTMLElement
    expect(notes.classList.contains('active')).toBe(true)
  })
})
