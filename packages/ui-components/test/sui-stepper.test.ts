import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-stepper/sui-stepper.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const dots = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('.dot'))

describe('<sui-stepper>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-stepper')).toBeDefined()
  })

  it('renders a shadow root with a dots container', () => {
    const { el } = mount(reg, 'sui-stepper')
    expect(shadow(el).querySelector('.dots')).not.toBeNull()
  })

  it('renders `total` dot spans', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '5', current: '1' })
    await flush()
    expect(dots(el).length).toBe(5)
  })

  it('marks dots with correct data-state', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '4', current: '3' })
    await flush()
    const states = dots(el).map(d => d.getAttribute('data-state'))
    expect(states).toEqual(['done', 'done', 'active', 'pending'])
  })

  it('sets role=progressbar on host', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '4', current: '1' })
    await flush()
    expect(el.getAttribute('role')).toBe('progressbar')
  })

  it('sets aria-valuemin, aria-valuemax, aria-valuenow', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '4', current: '2' })
    await flush()
    expect(el.getAttribute('aria-valuemin')).toBe('1')
    expect(el.getAttribute('aria-valuemax')).toBe('4')
    expect(el.getAttribute('aria-valuenow')).toBe('2')
  })

  it('derives default aria-label from current/total', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '4', current: '2' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Step 2 of 4')
  })

  it('uses label prop for aria-label when provided', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '4', current: '2', label: 'Onboarding' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Onboarding')
  })

  it('clamps current above total to total', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '3', current: '99' })
    await flush()
    const states = dots(el).map(d => d.getAttribute('data-state'))
    expect(states).toEqual(['done', 'done', 'active'])
    expect(el.getAttribute('aria-valuenow')).toBe('3')
  })

  it('clamps current below 1 to 1', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '3', current: '0' })
    await flush()
    const states = dots(el).map(d => d.getAttribute('data-state'))
    expect(states).toEqual(['active', 'pending', 'pending'])
    expect(el.getAttribute('aria-valuenow')).toBe('1')
  })

  it('re-renders dots when current changes', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '4', current: '1' })
    await flush()
    el.setAttribute('current', '4')
    await flush()
    const states = dots(el).map(d => d.getAttribute('data-state'))
    expect(states).toEqual(['done', 'done', 'done', 'active'])
  })

  it('each dot has part=dot and aria-hidden=true', async () => {
    const { el } = mount(reg, 'sui-stepper', { total: '3', current: '1' })
    await flush()
    for (const dot of dots(el)) {
      expect(dot.getAttribute('part')).toBe('dot')
      expect(dot.getAttribute('aria-hidden')).toBe('true')
    }
  })

  it('tone attribute reflects on host', () => {
    const { el } = mount(reg, 'sui-stepper', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('size attribute reflects on host', () => {
    const { el } = mount(reg, 'sui-stepper', { size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })
})
