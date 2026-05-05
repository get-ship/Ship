import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-status-bar/dist/sui-status-bar.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-status-bar>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-status-bar')).toBeDefined()
  })

  it('renders a connection dot with part=dot', () => {
    const { el } = mount(reg, 'sui-status-bar')
    expect(shadow(el).querySelector('.dot')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="dot"]')).not.toBeNull()
  })

  it('renders branch, agents, and theme spans', () => {
    const { el } = mount(reg, 'sui-status-bar')
    expect(shadow(el).querySelector('[part~="branch"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="agents"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="theme"]')).not.toBeNull()
  })

  it('online=false by default — attribute absent', () => {
    const { el } = mount(reg, 'sui-status-bar')
    expect(el.hasAttribute('online')).toBe(false)
  })

  it('reflects [online] attribute when set', () => {
    const { el } = mount(reg, 'sui-status-bar', { online: true })
    expect(el.hasAttribute('online')).toBe(true)
  })

  it('branch prop sets branch text content', async () => {
    const { el } = mount(reg, 'sui-status-bar', { branch: 'main' })
    await flush()
    const branchEl = shadow(el).querySelector('[part~="branch"]')!
    expect(branchEl.textContent?.trim()).toBe('main')
  })

  it('agents prop sets agents text content', async () => {
    const { el } = mount(reg, 'sui-status-bar', { agents: 'agent: build-01' })
    await flush()
    const agentsEl = shadow(el).querySelector('[part~="agents"]')!
    expect(agentsEl.textContent?.trim()).toBe('agent: build-01')
  })

  it('theme prop sets theme text content', async () => {
    const { el } = mount(reg, 'sui-status-bar', { theme: 'ship-dark' })
    await flush()
    const themeEl = shadow(el).querySelector('[part~="theme"]')!
    expect(themeEl.textContent?.trim()).toBe('ship-dark')
  })

  it('has start and end named slots', () => {
    const { el } = mount(reg, 'sui-status-bar')
    const slots = shadow(el).querySelectorAll('slot[name]')
    const names = Array.from(slots).map((s) => s.getAttribute('name'))
    expect(names).toContain('start')
    expect(names).toContain('end')
  })

  it('branch updates reactively when attribute changes', async () => {
    const { el } = mount(reg, 'sui-status-bar', { branch: 'main' })
    await flush()
    el.setAttribute('branch', 'feature/my-work')
    await flush()
    const branchEl = shadow(el).querySelector('[part~="branch"]')!
    expect(branchEl.textContent?.trim()).toBe('feature/my-work')
  })

  it('online attribute change updates [online] reflect', async () => {
    const { el } = mount(reg, 'sui-status-bar')
    await flush()
    expect(el.hasAttribute('online')).toBe(false)
    el.setAttribute('online', '')
    await flush()
    expect(el.hasAttribute('online')).toBe(true)
  })

  it('dot has aria-hidden=true', () => {
    const { el } = mount(reg, 'sui-status-bar')
    const dot = shadow(el).querySelector('.dot')!
    expect(dot.getAttribute('aria-hidden')).toBe('true')
  })
})
