import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-schema-editor/ship-schema-editor.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// ship-schema-editor reads ship.collections.detail for the existing
// schema; on Save it calls create or update_schema. The reproject
// warning fires on the first save when a name is removed.
let detailImpl: () => unknown
let createImpl: (args: unknown) => unknown
let updateSchemaImpl: (args: unknown) => unknown
beforeEach(() => {
  detailImpl = () => ({
    schema: { id: 'notes', name: 'Notes', fields: [
      { name: 'title', type: 'text' },
      { name: 'pinned', type: 'bool' },
    ], builtin: false },
    items: [],
    total_items: 0,
  })
  createImpl = vi.fn().mockResolvedValue({ ok: true, id: 'new-collection' })
  updateSchemaImpl = vi.fn().mockResolvedValue({ ok: true, id: 'notes' })
  setShipClient({
    collections: {
      detail: () => detailImpl(),
      create: (a: unknown) => createImpl(a),
      update_schema: (a: unknown) => updateSchemaImpl(a),
    },
  } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLElement | null

describe('<ship-schema-editor>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-schema-editor')).toBeDefined()
  })

  it('hydrates the guided view with one row per existing field', async () => {
    const { el } = mount(reg, 'ship-schema-editor', { id: 'notes' })
    await flush()
    expect(q(el, 'editor-title')?.textContent).toBe('notes')
    expect(shadow(el).querySelectorAll('[data-test="field"]').length).toBe(2)
  })

  it('toggles view between guided and raw, reflecting the attribute', async () => {
    const { el } = mount(reg, 'ship-schema-editor', { id: 'notes' })
    await flush()
    expect(el.getAttribute('view')).toBe('guided')
    ;(q(el, 'view-raw') as HTMLElement).click()
    expect(el.getAttribute('view')).toBe('raw')
    expect(q(el, 'raw')).not.toBeNull()
  })

  it('adds a new field when + Add field is clicked', async () => {
    const { el } = mount(reg, 'ship-schema-editor', { id: 'notes' })
    await flush()
    const before = shadow(el).querySelectorAll('[data-test="field"]').length
    ;(q(el, 'add') as HTMLElement).click()
    await flush()
    expect(shadow(el).querySelectorAll('[data-test="field"]').length).toBe(before + 1)
  })

  it('removes a field when × is clicked', async () => {
    const { el } = mount(reg, 'ship-schema-editor', { id: 'notes' })
    await flush()
    const firstRemove = shadow(el).querySelector('[data-test="field"] [data-test="remove"]') as HTMLElement
    firstRemove.click()
    await flush()
    expect(shadow(el).querySelectorAll('[data-test="field"]').length).toBe(1)
  })

  it('fires update_schema on save and emits ship-schema-save { id, fields }', async () => {
    const { el } = mount(reg, 'ship-schema-editor', { id: 'notes' })
    await flush()
    const onSave = vi.fn()
    el.addEventListener('ship-schema-save', onSave)
    ;(q(el, 'save') as HTMLElement).click()
    await flush()
    expect(updateSchemaImpl).toHaveBeenCalled()
    expect(onSave).toHaveBeenCalledTimes(1)
    const detail = (onSave.mock.calls[0][0] as CustomEvent).detail
    expect(detail.id).toBe('notes')
    expect(Array.isArray(detail.fields)).toBe(true)
  })

  it('surfaces the reproject warning when a field is removed before save', async () => {
    const { el } = mount(reg, 'ship-schema-editor', { id: 'notes' })
    await flush()
    const firstRemove = shadow(el).querySelector('[data-test="field"] [data-test="remove"]') as HTMLElement
    firstRemove.click()
    await flush()
    ;(q(el, 'save') as HTMLElement).click()
    await flush()
    expect(q(el, 'reproject-warning')).not.toBeNull()
    expect(updateSchemaImpl).not.toHaveBeenCalled()
  })
})
