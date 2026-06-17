import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-item-editor/ship-item-editor.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

// ship-item-editor reads ship.collections.state for the schema, then
// create_item / update_item on save. Per the spec the create path is
// the default; update happens when mode='edit' + an id is passed.
let stateImpl: () => unknown
let createItemImpl: (args: unknown) => unknown
let updateItemImpl: (args: unknown) => unknown
beforeEach(() => {
  stateImpl = () => ({
    schema: { id: 'notes', name: 'Notes', fields: [
      { name: 'title', type: 'text', field_type: 'text', required: true, values: null, default: null },
      { name: 'pinned', type: 'bool', field_type: 'bool', required: false, values: null, default: false },
    ], builtin: false },
    total_items: 0,
  })
  createItemImpl = vi.fn().mockResolvedValue({ ok: true, id: 'new-id' })
  updateItemImpl = vi.fn().mockResolvedValue({ ok: true, id: 'note-1' })
  setShipClient({
    collections: {
      state: () => stateImpl(),
      create_item: (a: unknown) => createItemImpl(a),
      update_item: (a: unknown) => updateItemImpl(a),
    },
  } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLElement | null

describe('<ship-item-editor>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-item-editor')).toBeDefined()
  })

  it('renders a form with one field row per schema field', async () => {
    const { el } = mount(reg, 'ship-item-editor', { collection: 'notes', mode: 'create' })
    await flush()
    expect(q(el, 'editor-title')?.textContent).toBe('New item')
    expect(shadow(el).querySelectorAll('[data-test^="field-"]').length).toBe(2)
  })

  it('reflects the mode attribute (create|edit) and renders the id in edit', async () => {
    const { el } = mount(reg, 'ship-item-editor', { collection: 'notes', mode: 'edit', id: 'note-1' })
    await flush()
    expect(el.getAttribute('mode')).toBe('edit')
    expect(q(el, 'editor-title')?.textContent).toBe('Edit item')
    expect(q(el, 'editor-id')?.textContent).toBe('note-1')
  })

  it('calls create_item on submit and emits ship-item-save { collection, data }', async () => {
    const { el } = mount(reg, 'ship-item-editor', { collection: 'notes', mode: 'create' })
    await flush()
    // Type into the title field via a synthetic input event so the
    // bind:value signal picks it up.
    const titleInput = q(el, 'input-title') as HTMLElement & { value?: string } | null
    expect(titleInput).not.toBeNull()
    const innerInput = shadow(titleInput as HTMLElement).querySelector('input') as HTMLInputElement | null
    expect(innerInput).not.toBeNull()
    if (innerInput) {
      innerInput.value = 'Hello'
      innerInput.dispatchEvent(new Event('input', { bubbles: true }))
      innerInput.dispatchEvent(new Event('change', { bubbles: true }))
    }
    await flush()
    const onSave = vi.fn()
    el.addEventListener('ship-item-save', onSave)
    const form = q(el, 'form') as HTMLFormElement
    form.requestSubmit()
    await flush()
    expect(createItemImpl).toHaveBeenCalled()
    expect(onSave).toHaveBeenCalledTimes(1)
    const detail = (onSave.mock.calls[0][0] as CustomEvent).detail
    expect(detail.collection).toBe('notes')
    expect(detail.data.title).toBe('Hello')
  })

  it('emits ship-item-cancel { collection } on cancel click', async () => {
    const { el } = mount(reg, 'ship-item-editor', { collection: 'notes', mode: 'create' })
    await flush()
    const onCancel = vi.fn()
    el.addEventListener('ship-item-cancel', onCancel)
    ;(q(el, 'cancel') as HTMLElement).click()
    expect(onCancel).toHaveBeenCalledTimes(1)
    expect((onCancel.mock.calls[0][0] as CustomEvent).detail).toEqual({ collection: 'notes' })
  })

  it('surfaces a required-field error before calling create_item', async () => {
    const { el } = mount(reg, 'ship-item-editor', { collection: 'notes', mode: 'create' })
    await flush()
    const onSave = vi.fn()
    el.addEventListener('ship-item-save', onSave)
    const form = q(el, 'form') as HTMLFormElement
    form.requestSubmit()
    await flush()
    expect(createItemImpl).not.toHaveBeenCalled()
    expect(onSave).not.toHaveBeenCalled()
    expect(q(el, 'errors')?.textContent).toContain('title')
  })
})
