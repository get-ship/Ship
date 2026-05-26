import { afterEach, describe, expect, it, vi } from 'vitest'
import '../../src/components/sui-nav-top/sui-nav-top.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

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

  it('renders the action cluster (fullscreen, theme picker, account menu)', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    const shadow_ = shadow(el)
    expect(shadow_.querySelector('[part~="fullscreen"]')).not.toBeNull()
    expect(shadow_.querySelector('sui-theme-picker')).not.toBeNull()
    expect(shadow_.querySelector('sui-account-menu')).not.toBeNull()
  })

  it('has no rail-toggle or dock-toggle button (dropped — rail + dock own their own controls)', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    const shadow_ = shadow(el)
    expect(shadow_.querySelector('[part~="rail-toggle"]')).toBeNull()
    expect(shadow_.querySelector('[part~="dock-toggle"]')).toBeNull()
  })

  it('has no chat-toggle button (removed)', async () => {
    const { el } = mount(reg, 'sui-nav-top')
    await flush()
    expect(shadow(el).querySelector('[part~="chat-toggle"]')).toBeNull()
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

  it('path="" renders no view crumb (home — the brand logo is the home button)', async () => {
    const { el } = mount(reg, 'sui-nav-top', { path: '' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    expect(crumbs.querySelector('a')).toBeNull()
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

  it('project prop prepends a home-linked crumb', async () => {
    const { el } = mount(reg, 'sui-nav-top', { project: 'Ship', path: '/agents' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    const links = crumbs.querySelectorAll('a')
    expect(links.length).toBeGreaterThanOrEqual(1)
    expect(links[0]!.getAttribute('href')).toBe('/')
    expect(links[0]!.textContent).toBe('Ship')
  })

  it('view prop overrides path-derived root segment', async () => {
    const { el } = mount(reg, 'sui-nav-top', { view: 'home', path: '/pipelines' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    // 'home' should be the leaf, not 'pipelines'
    expect(crumbs.textContent).toContain('Home')
    expect(crumbs.textContent).not.toContain('Pipelines')
  })

  it('project + view together render "Project › View"', async () => {
    const { el } = mount(reg, 'sui-nav-top', { project: 'Ship', view: 'home' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    const projectLink = crumbs.querySelector('a')!
    expect(projectLink.getAttribute('href')).toBe('/')
    expect(projectLink.textContent).toBe('Ship')
    expect(crumbs.querySelector('.current')!.textContent).toBe('Home')
  })

  it('empty project + empty view falls back to path-only crumbs', async () => {
    const { el } = mount(reg, 'sui-nav-top', { path: '/git' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    // No leading anchor segment beyond the path-derived one (which would be the leaf)
    expect(crumbs.textContent).toContain('Git')
    expect(crumbs.textContent).not.toContain('Ship')
  })

  it('crumb anchors carry a data-sui-transition marker (transition:fade wired)', async () => {
    const { el } = mount(reg, 'sui-nav-top', { project: 'Ship', path: '/agents' })
    await flush()
    const crumbs = shadow(el).querySelector('.crumbs')!
    // transition: directive registers `data-sui-transition` on the node.
    // See packages/sui/runtime/src/transition.ts MARK constant.
    expect(crumbs.querySelector('a[data-sui-transition]')).not.toBeNull()
    expect(crumbs.querySelector('.current[data-sui-transition]')).not.toBeNull()
  })
})
