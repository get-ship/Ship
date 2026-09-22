import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-usage-chip/ship-usage-chip.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot ?? el
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)

describe('<ship-usage-chip>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-usage-chip')).toBeDefined()
  })

  it('renders input + output token counts', async () => {
    const { el } = mount(reg, 'ship-usage-chip', { input: '1.2k', output: '340' })
    await flush()
    expect(q(el, 'usage-input')?.textContent).toBe('1.2k')
    expect(q(el, 'usage-output')?.textContent).toBe('340')
  })

  it('minimal: hides reasoning, cache and cost when empty', async () => {
    const { el } = mount(reg, 'ship-usage-chip', { input: '10', output: '5' })
    await flush()
    expect(q(el, 'usage-reasoning')).toBeNull()
    expect(q(el, 'usage-cache')).toBeNull()
    expect(q(el, 'usage-cost')).toBeNull()
  })

  it('full: shows reasoning, cache and cost when provided', async () => {
    const { el } = mount(reg, 'ship-usage-chip', {
      input: '10', output: '5', reasoning: '88', cache: '120', cost: '$0.04',
    })
    await flush()
    expect(q(el, 'usage-reasoning')?.textContent).toContain('88')
    expect(q(el, 'usage-cache')?.textContent).toContain('120')
    expect(q(el, 'usage-cost')?.textContent).toBe('$0.04')
  })
})
