import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-nav-top/sui-nav-top.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-nav-top>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-nav-top')).toBeDefined()
  })

  it('renders a brand link in shadow DOM', () => {
    const { el } = mount(reg, 'sui-nav-top')
    expect(shadow(el).querySelector('.brand')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="brand"]')).not.toBeNull()
  })

  it('brand link has correct href', () => {
    const { el } = mount(reg, 'sui-nav-top')
    const brand = shadow(el).querySelector<HTMLAnchorElement>('.brand')!
    expect(brand.getAttribute('href')).toBe('/')
  })

  it('renders breadcrumbs nav', () => {
    const { el } = mount(reg, 'sui-nav-top')
    expect(shadow(el).querySelector('[part~="crumbs"]')).not.toBeNull()
  })

  it('renders all action buttons', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    const shadow_ = shadow(el)
    expect(shadow_.querySelector('[part~="rail-toggle"]')).not.toBeNull()
    expect(shadow_.querySelector('[part~="studio"]')).not.toBeNull()
    expect(shadow_.querySelector('[part~="dock-toggle"]')).not.toBeNull()
    expect(shadow_.querySelector('[part~="fullscreen"]')).not.toBeNull()
    expect(shadow_.querySelector('[part~="chat-toggle"]')).not.toBeNull()
  })

  it('renders <sui-theme-picker> in shadow DOM', () => {
    const { el } = mount(reg, 'sui-nav-top')
    expect(shadow(el).querySelector('sui-theme-picker')).not.toBeNull()
  })

  it('sui-theme-picker has placement="bottom-end"', () => {
    const { el } = mount(reg, 'sui-nav-top')
    const picker = shadow(el).querySelector('sui-theme-picker')!
    expect(picker.getAttribute('placement')).toBe('bottom-end')
  })

  it('path="" renders "dashboard" crumb as fallback', async () => {
    const { el } = mount(reg, 'sui-nav-top', { path: '' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    expect(crumbs.textContent).toContain('Dashboard')
  })

  it('path prop drives breadcrumb content', async () => {
    const { el } = mount(reg, 'sui-nav-top', { path: '/pipelines' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    expect(crumbs.textContent).toContain('Pipelines')
  })

  it('multi-segment path renders ancestor links + leaf span', async () => {
    const { el } = mount(reg, 'sui-nav-top', { path: '/settings/team' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    const link = crumbs.querySelector('a')!
    expect(link.getAttribute('href')).toBe('/settings')
    expect(crumbs.querySelector('.current')).not.toBeNull()
  })

  it('#breadcrumbs-title targets the leaf crumb span', async () => {
    const { el } = mount(reg, 'sui-nav-top', { path: '/agents' })
    await flush()
    const title = shadow(el).getElementById('breadcrumbs-title')
    expect(title).not.toBeNull()
    expect(title!.textContent?.trim()).not.toBe('')
  })

  it('sui:navigate event on document updates breadcrumbs', async () => {
    const { el } = mount(reg, 'sui-nav-top', { path: '/dashboard' })
    await flush()
    document.dispatchEvent(
      new CustomEvent('sui:navigate', { detail: { to: '/git' } }),
    )
    await flush()
    expect(shadow(el).querySelector('.crumbs')!.textContent).toContain('Git')
  })

  it('rail-toggle button dispatches sui:rail:toggle when clicked', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    const events: Event[] = []
    el.addEventListener('sui:rail:toggle', (e) => events.push(e))
    const btn = shadow(el).querySelector<HTMLElement>('[part~="rail-toggle"]')!
    btn.click()
    await flush()
    expect(events).toHaveLength(1)
  })

  it('chat-toggle button dispatches sui:chat:toggle when clicked', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    const events: Event[] = []
    el.addEventListener('sui:chat:toggle', (e) => events.push(e))
    const btn = shadow(el).querySelector<HTMLElement>('[part~="chat-toggle"]')!
    btn.click()
    await flush()
    expect(events).toHaveLength(1)
  })

  it('dock-toggle button dispatches sui:dock:toggle when clicked', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    const events: Event[] = []
    el.addEventListener('sui:dock:toggle', (e) => events.push(e))
    const btn = shadow(el).querySelector<HTMLElement>('[part~="dock-toggle"]')!
    btn.click()
    await flush()
    expect(events).toHaveLength(1)
  })

  it('dispatches sui:nav-top:ready on connect', async () => {
    const events: Event[] = []
    const container = reg.mk()
    const listen = (e: Event) => events.push(e)
    container.addEventListener('sui:nav-top:ready', listen, { once: true })
    container.innerHTML = '<sui-nav-top></sui-nav-top>'
    await flush()
    expect(events.length).toBeGreaterThanOrEqual(1)
  })

  it('has slot name="actions" for view-level buttons', () => {
    const { el } = mount(reg, 'sui-nav-top', {}, '<button slot="actions">A</button>')
    const slot = shadow(el).querySelector<HTMLSlotElement>('slot[name="actions"]')
    expect(slot).not.toBeNull()
  })

  it('has slot name="command-bar" for command palette', () => {
    const { el } = mount(reg, 'sui-nav-top')
    const slot = shadow(el).querySelector<HTMLSlotElement>('slot[name="command-bar"]')
    expect(slot).not.toBeNull()
  })

  it('dock-toggle has aria-pressed attribute', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    const btn = shadow(el).querySelector('[part~="dock-toggle"]')!
    expect(btn.hasAttribute('aria-pressed')).toBe(true)
  })
})
