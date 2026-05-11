import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import '../src/sui-edit-overlay/dist/sui-edit-overlay.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

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

  it('adds .active class when body gains edit-mode', async () => {
    const { el } = mount(reg, 'sui-edit-overlay')
    await flush()
    document.body.classList.add('edit-mode')
    await flush()
    expect(el.classList.contains('active')).toBe(true)
  })

  it('removes .active class when body loses edit-mode', async () => {
    document.body.classList.add('edit-mode')
    const { el } = mount(reg, 'sui-edit-overlay')
    await flush()
    expect(el.classList.contains('active')).toBe(true)
    document.body.classList.remove('edit-mode')
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
