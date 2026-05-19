import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-center/sui-center.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()

afterEach(() => {
  reg.cleanup()
})

describe('<sui-center>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-center')).toBeDefined()
  })

  it('renders a shadow root with a default slot', async () => {
    const { el } = mount(reg, 'sui-center', {}, '<p>x</p>')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('slot')).not.toBeNull()
  })

  it('reflects max + padded attributes', async () => {
    const { el } = mount(reg, 'sui-center', { max: 'wide', padded: '' })
    await flush()
    expect(el.getAttribute('max')).toBe('wide')
    expect(el.hasAttribute('padded')).toBe(true)
  })

  it('wraps the slot in a single .center container', async () => {
    const { el } = mount(reg, 'sui-center')
    await flush()
    const inner = el.shadowRoot!.querySelector('.center') as HTMLElement
    expect(inner).not.toBeNull()
    expect(inner.querySelector('slot')).not.toBeNull()
  })
})
