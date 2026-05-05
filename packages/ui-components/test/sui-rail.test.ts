import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-rail/dist/sui-rail.compiled.js'
import '../src/sui-rail/dist/sui-rail-item.compiled.js'
import '../src/sui-rail/dist/sui-rail-section.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-rail>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-rail')).toBeDefined()
  })

  it('renders nav + toggle button in shadow DOM', () => {
    const { el } = mount(reg, 'sui-rail')
    expect(shadow(el).querySelector('.rail-nav')).not.toBeNull()
    expect(shadow(el).querySelector('.toggle-btn')).not.toBeNull()
  })

  it('has part=nav and part=toggle', () => {
    const { el } = mount(reg, 'sui-rail')
    expect(shadow(el).querySelector('[part~="nav"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="toggle"]')).not.toBeNull()
  })

  it('collapsed=false by default — attribute absent', () => {
    const { el } = mount(reg, 'sui-rail')
    expect(el.hasAttribute('collapsed')).toBe(false)
  })

  it('reflects [collapsed] attribute when set', () => {
    const { el } = mount(reg, 'sui-rail', { collapsed: true })
    expect(el.hasAttribute('collapsed')).toBe(true)
  })

  it('toggle button click flips collapsed and dispatches sui-rail:toggle', async () => {
    const { el } = mount(reg, 'sui-rail')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui-rail:toggle', (e) => events.push(e as CustomEvent))

    const btn = shadow(el).querySelector<HTMLButtonElement>('.toggle-btn')!
    btn.click()
    await flush()

    expect(el.hasAttribute('collapsed')).toBe(true)
    expect(events).toHaveLength(1)
    expect(events[0].detail.collapsed).toBe(true)
  })

  it('second toggle click restores uncollapsed state', async () => {
    const { el } = mount(reg, 'sui-rail')
    await flush()
    const btn = shadow(el).querySelector<HTMLButtonElement>('.toggle-btn')!
    btn.click()
    await flush()
    btn.click()
    await flush()
    expect(el.hasAttribute('collapsed')).toBe(false)
  })

  it('toggle button aria-label changes with collapsed state', async () => {
    const { el } = mount(reg, 'sui-rail')
    await flush()
    const btn = shadow(el).querySelector<HTMLButtonElement>('.toggle-btn')!
    expect(btn.getAttribute('aria-label')).toBe('Hide rail')
    btn.click()
    await flush()
    expect(btn.getAttribute('aria-label')).toBe('Show rail')
  })

  it('toggle button has no sui-icon — uses CSS content for arrow indicator', async () => {
    const { el } = mount(reg, 'sui-rail')
    await flush()
    const btn = shadow(el).querySelector<HTMLButtonElement>('.toggle-btn')!
    expect(btn.querySelector('sui-icon')).toBeNull()
  })

  it('has a default slot for child items', () => {
    const { el } = mount(reg, 'sui-rail', {}, '<sui-rail-item label="Test" icon="zap"></sui-rail-item>')
    const slot = shadow(el).querySelector<HTMLSlotElement>('slot:not([name])')!
    expect(slot).not.toBeNull()
    expect(slot.assignedElements().length).toBe(1)
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-rail-item>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-rail-item')).toBeDefined()
  })

  it('renders an anchor with part=item in shadow DOM', () => {
    const { el } = mount(reg, 'sui-rail-item', { label: 'Dashboard', icon: 'layout-dashboard', href: '/' })
    const a = shadow(el).querySelector('a.item')
    expect(a).not.toBeNull()
    expect(a!.getAttribute('part')).toBe('item')
  })

  it('reflects href on the anchor', () => {
    const { el } = mount(reg, 'sui-rail-item', { href: '/pipelines', label: 'Pipelines', icon: 'zap' })
    const a = shadow(el).querySelector<HTMLAnchorElement>('a.item')!
    expect(a.getAttribute('href')).toBe('/pipelines')
  })

  it('renders label text in .label span', async () => {
    const { el } = mount(reg, 'sui-rail-item', { label: 'Agents', icon: 'bot', href: '/agents' })
    await flush()
    const labelEl = shadow(el).querySelector('.label')!
    expect(labelEl.textContent?.trim()).toBe('Agents')
  })

  it('passes icon name to sui-icon', () => {
    const { el } = mount(reg, 'sui-rail-item', { icon: 'settings', label: 'Settings', href: '/settings' })
    const icon = shadow(el).querySelector<HTMLElement>('sui-icon')!
    expect(icon.getAttribute('name')).toBe('settings')
  })

  it('active=false by default — no aria-current', () => {
    const { el } = mount(reg, 'sui-rail-item', { href: '/', label: 'Home', icon: 'home' })
    const a = shadow(el).querySelector('a.item')!
    expect(a.hasAttribute('aria-current')).toBe(false)
    expect(el.hasAttribute('active')).toBe(false)
  })

  it('reflects [active] and sets aria-current=page on anchor', async () => {
    const { el } = mount(reg, 'sui-rail-item', { href: '/', label: 'Home', icon: 'home', active: true })
    await flush()
    expect(el.hasAttribute('active')).toBe(true)
    const a = shadow(el).querySelector('a.item')!
    expect(a.getAttribute('aria-current')).toBe('page')
  })

  it('badge shows badge text when non-empty', async () => {
    const { el } = mount(reg, 'sui-rail-item', { label: 'Chat', icon: 'message-square', href: '/chat', badge: '3' })
    await flush()
    const badge = shadow(el).querySelector('.badge')!
    expect(badge.textContent?.trim()).toBe('3')
  })

  it('disabled reflects [disabled] and sets aria-disabled on anchor', () => {
    const { el } = mount(reg, 'sui-rail-item', { href: '/', label: 'Home', icon: 'home', disabled: true })
    expect(el.hasAttribute('disabled')).toBe(true)
    const a = shadow(el).querySelector('a.item')!
    expect(a.getAttribute('aria-disabled')).toBe('true')
  })

  it('has part=icon-wrap, part=label, part=badge', () => {
    const { el } = mount(reg, 'sui-rail-item', { label: 'X', icon: 'x', href: '/' })
    expect(shadow(el).querySelector('[part~="icon-wrap"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="label"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="badge"]')).not.toBeNull()
  })
})
