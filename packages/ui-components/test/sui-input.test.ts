import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-input/sui-input.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const input = (el: HTMLElement) => shadow(el).querySelector('input') as HTMLInputElement

describe('<sui-input>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-input')).toBeDefined()
  })

  it('renders a native input in shadow DOM', () => {
    const { el } = mount(reg, 'sui-input')
    expect(input(el)).not.toBeNull()
  })

  it('is form-associated', () => {
    const ctor = customElements.get('sui-input') as unknown as {
      formAssociated?: boolean
    }
    expect(ctor.formAssociated).toBe(true)
  })

  it('participates in FormData when named inside a <form>', async () => {
    const container = reg.mk()
    container.innerHTML = '<form><sui-input name="q" value="hello"></sui-input></form>'
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const proto = (window as unknown as { ElementInternals?: { prototype: ElementInternals } })
      .ElementInternals?.prototype
    if (!proto || typeof proto.setFormValue !== 'function') return
    const data = new FormData(form)
    expect(data.get('q')).toBe('hello')
  })

  it('formResetCallback restores value to the prop default (empty)', async () => {
    const { el } = mount(reg, 'sui-input', { value: 'typed' })
    await flush()
    ;(el as unknown as { formResetCallback?: () => void }).formResetCallback!()
    await flush()
    expect(input(el).value).toBe('')
  })

  it('formDisabledCallback drives the disabled prop', async () => {
    const { el } = mount(reg, 'sui-input')
    await flush()
    expect(el.hasAttribute('disabled')).toBe(false)
    ;(el as unknown as { formDisabledCallback?: (d: boolean) => void })
      .formDisabledCallback!(true)
    await flush()
    expect(el.hasAttribute('disabled')).toBe(true)
  })
})
