import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-session-metrics/ship-session-metrics.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot ?? el
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)

describe('<ship-session-metrics>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-session-metrics')).toBeDefined()
  })

  it('renders the rollup stats', async () => {
    const { el } = mount(reg, 'ship-session-metrics', {
      duration: '32s', turns: '5', tools: '12', errors: '1', cost: '$0.21',
    })
    await flush()
    expect(q(el, 'm-duration')?.textContent).toBe('32s')
    expect(q(el, 'm-turns')?.textContent).toBe('5')
    expect(q(el, 'm-tools')?.textContent).toBe('12')
    expect(q(el, 'm-errors')?.textContent).toBe('1')
    expect(q(el, 'm-cost')?.textContent).toBe('$0.21')
  })

  it('reflects outcome and tones the strip', async () => {
    const { el } = mount(reg, 'ship-session-metrics', { outcome: 'failed' })
    await flush()
    expect(el.getAttribute('outcome')).toBe('failed')
    expect(q(el, 'm-outcome')?.textContent).toBe('failed')
  })

  it('neutralises the errors stat when zero via data-zero', async () => {
    const { el } = mount(reg, 'ship-session-metrics', { errors: '0' })
    await flush()
    const errStat = shadow(el).querySelector('.stat.err')
    expect(errStat?.getAttribute('data-zero')).toBe('true')

    el.setAttribute('errors', '3')
    await flush()
    expect(shadow(el).querySelector('.stat.err')?.hasAttribute('data-zero')).toBe(false)
  })
})
