import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-loading/sui-loading.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-loading>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-loading')).toBeDefined()
  })

  it('renders stage and message parts in shadow DOM', () => {
    const { el } = mount(reg, 'sui-loading')
    expect(shadow(el).querySelector('[part~="stage"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="message"]')).not.toBeNull()
  })

  it('renders rocket SVG inside stage', () => {
    const { el } = mount(reg, 'sui-loading')
    const svg = shadow(el).querySelector('[part~="stage"] svg')
    expect(svg).not.toBeNull()
  })

  it('compact=false by default — attribute absent', () => {
    const { el } = mount(reg, 'sui-loading')
    expect(el.hasAttribute('compact')).toBe(false)
  })

  it('reflects [compact] attribute when set', () => {
    const { el } = mount(reg, 'sui-loading', { compact: true })
    expect(el.hasAttribute('compact')).toBe(true)
  })

  it('shows a default message on mount', async () => {
    const { el } = mount(reg, 'sui-loading')
    await flush()
    const msgEl = shadow(el).querySelector('[part~="message"]')!
    expect(msgEl.textContent?.trim().length).toBeGreaterThan(0)
  })

  it('shows first message from messages prop', async () => {
    const { el } = mount(reg, 'sui-loading', { messages: 'Hello|World' })
    await flush()
    const msgEl = shadow(el).querySelector('[part~="message"]')!
    expect(msgEl.textContent?.trim()).toBe('Hello')
  })

  it('has .flame inside .stage', () => {
    const { el } = mount(reg, 'sui-loading')
    expect(shadow(el).querySelector('.stage .flame')).not.toBeNull()
  })

  it('has exhaust particles inside .stage', () => {
    const { el } = mount(reg, 'sui-loading')
    const particles = shadow(el).querySelectorAll('.stage .particle')
    expect(particles.length).toBe(3)
  })

  it('message element starts with opacity 1 (visible)', async () => {
    const { el } = mount(reg, 'sui-loading', { messages: 'A|B|C' })
    await flush()
    const msgEl = shadow(el).querySelector('[part~="message"]')! as HTMLElement
    // After mount the controller sets opacity to '1'
    expect(msgEl.style.opacity).toBe('1')
  })

  it('disconnects without throwing', async () => {
    const { el, container } = mount(reg, 'sui-loading')
    await flush()
    expect(() => container.removeChild(el)).not.toThrow()
  })
})
