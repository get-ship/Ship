import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-grid/sui-grid.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()

afterEach(() => {
  reg.cleanup()
})

describe('<sui-grid>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-grid')).toBeDefined()
  })

  it('renders a shadow root with a default slot', async () => {
    const { el } = mount(reg, 'sui-grid')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('slot')).not.toBeNull()
  })

  it('reflects gap + min attributes', async () => {
    const { el } = mount(reg, 'sui-grid', { gap: '6', min: '220px' })
    await flush()
    expect(el.getAttribute('gap')).toBe('6')
    expect(el.getAttribute('min')).toBe('220px')
  })

  it('wraps the slot in a single .grid container', async () => {
    const { el } = mount(reg, 'sui-grid')
    await flush()
    const inner = el.shadowRoot!.querySelector('.grid') as HTMLElement
    expect(inner).not.toBeNull()
    expect(inner.querySelector('slot')).not.toBeNull()
  })
})
