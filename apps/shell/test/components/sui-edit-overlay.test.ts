import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import '../../src/components/sui-edit-overlay/sui-edit-overlay.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()
afterEach(() => {
  reg.cleanup()
  document.body.classList.remove('edit-mode')
})

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-edit-overlay>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-edit-overlay')).toBeDefined()
  })

  it('renders box, label, and hint parts in shadow DOM', () => {
    const { el } = mount(reg, 'sui-edit-overlay')
    expect(shadow(el).querySelector('[part~="box"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="label"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="hint"]')).not.toBeNull()
  })

  it('hint contains edit-mode text', () => {
    const { el } = mount(reg, 'sui-edit-overlay')
    const hint = shadow(el).querySelector('[part~="hint"]')!
    expect(hint.textContent).toContain('Edit mode')
  })

  it('is inactive by default — no .active class on host', async () => {
    const { el } = mount(reg, 'sui-edit-overlay')
    await flush()
    expect(el.classList.contains('active')).toBe(false)
  })

  it('adds .active class when body gains annotation-mode', async () => {
    // sui-edit-overlay reacts to body.annotation-mode (the sub-toggle
    // inside edit mode). The earlier coupling to body.edit-mode predated
    // the annotation/edit split — the overlay only paints when the user
    // is actively annotating.
    const { el } = mount(reg, 'sui-edit-overlay')
    await flush()
    document.body.classList.add('annotation-mode')
    await flush()
    expect(el.classList.contains('active')).toBe(true)
    document.body.classList.remove('annotation-mode')
  })

  it('removes .active class when body loses annotation-mode', async () => {
    document.body.classList.add('annotation-mode')
    const { el } = mount(reg, 'sui-edit-overlay')
    await flush()
    expect(el.classList.contains('active')).toBe(true)
    document.body.classList.remove('annotation-mode')
    await flush()
    expect(el.classList.contains('active')).toBe(false)
  })

  it('box starts hidden (no .visible class)', async () => {
    const { el } = mount(reg, 'sui-edit-overlay')
    await flush()
    const box = shadow(el).querySelector('[part~="box"]')!
    expect(box.classList.contains('visible')).toBe(false)
  })

  it('disconnects observer on removal', async () => {
    const { el, container } = mount(reg, 'sui-edit-overlay')
    await flush()
    // Should not throw when removed
    expect(() => container.removeChild(el)).not.toThrow()
  })
})
