import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-metachip/dist/sui-metachip.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-metachip>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-metachip')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-metachip')
    expect(shadow(el)).not.toBeNull()
  })

  it('renders label text from the label attribute', async () => {
    const { el } = mount(reg, 'sui-metachip', { label: 'Status' })
    await flush()
    const k = shadow(el).querySelector('.k')!
    expect(k).not.toBeNull()
    const slot = k.querySelector('slot[name="label"]') as HTMLSlotElement
    expect(slot).not.toBeNull()
  })

  it('renders value text from the value attribute as fallback', async () => {
    const { el } = mount(reg, 'sui-metachip', { value: 'Draft' })
    await flush()
    const v = shadow(el).querySelector('.v')!
    expect(v).not.toBeNull()
    const slot = v.querySelector('slot:not([name])') as HTMLSlotElement
    expect(slot).not.toBeNull()
  })

  it('label reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-metachip', { label: 'Agent' })
    expect(el.getAttribute('label')).toBe('Agent')
  })

  it('value reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-metachip', { value: 'test-writer' })
    expect(el.getAttribute('value')).toBe('test-writer')
  })

  it('.k part is "label"', () => {
    const { el } = mount(reg, 'sui-metachip', { label: 'X' })
    expect(shadow(el).querySelector('.k')!.getAttribute('part')).toBe('label')
  })

  it('.v part is "value"', () => {
    const { el } = mount(reg, 'sui-metachip', { value: 'Y' })
    expect(shadow(el).querySelector('.v')!.getAttribute('part')).toBe('value')
  })

  it('default slot overrides value attr for display', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-metachip label="Pipeline">tdd</sui-metachip>'
    const el = container.querySelector('sui-metachip')!
    expect(el.textContent?.trim()).toBe('tdd')
  })

  it('named label slot overrides label attr', () => {
    const container = reg.mk()
    container.innerHTML =
      '<sui-metachip label="raw"><span slot="label">formatted</span></sui-metachip>'
    const el = container.querySelector('sui-metachip')!
    const slotted = el.querySelector('[slot="label"]')!
    expect(slotted.textContent).toBe('formatted')
  })

  it('both label and value attrs can be set together', () => {
    const { el } = mount(reg, 'sui-metachip', { label: 'Status', value: 'ok' })
    expect(el.getAttribute('label')).toBe('Status')
    expect(el.getAttribute('value')).toBe('ok')
  })

  it('shadow has .k and .v spans', () => {
    const { el } = mount(reg, 'sui-metachip', { label: 'k', value: 'v' })
    expect(shadow(el).querySelector('.k')).not.toBeNull()
    expect(shadow(el).querySelector('.v')).not.toBeNull()
  })
})
