import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../../src/components/sui-rail/sui-rail.compiled.js'
import '../../src/components/sui-rail/sui-rail-item.compiled.js'
import '../../src/components/sui-rail/sui-rail-section.compiled.js'
// sui-portal / sui-icon live in @ship/primitives.
import '@ship/primitives'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()
afterEach(() => {
  reg.cleanup()
  // Strip any popovers we teleported to body during a test so the
  // next test starts from a clean document.
  for (const el of document.body.querySelectorAll('.sui-rail-project-popover')) {
    el.remove()
  }
  vi.restoreAllMocks()
})

const shadow = (el: HTMLElement) => el.shadowRoot!

function mockFetch(routes: Record<string, unknown>): void {
  vi.spyOn(globalThis, 'fetch').mockImplementation(async (input: RequestInfo | URL) => {
    const url = typeof input === 'string' ? input : input.toString()
    const key = Object.keys(routes).find((k) => url.includes(k))
    const body = key ? routes[key] : null
    return new Response(JSON.stringify(body ?? {}), { status: 200, headers: { 'Content-Type': 'application/json' } })
  })
}

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-rail>', () => {
  beforeEach(() => {
    mockFetch({
      '/api/v1/projects': { projects: [], active: null },
    })
  })

  it('registers the custom element', () => {
    expect(customElements.get('sui-rail')).toBeDefined()
  })

  it('renders nav, footer, picker, create parts', async () => {
    // Project switcher + launcher moved off the rail's footer when the
    // rail was condensed; the picker-btn now opens a <sui-view-picker>
    // that owns both. Tests for the legacy `.project-badge` and
    // `[part~="launcher-btn"]` are no longer applicable — see the
    // sui-view-picker tests for picker coverage.
    const { el } = mount(reg, 'sui-rail')
    await flush()
    const s = shadow(el)
    expect(s.querySelector('[part~="nav"]')).not.toBeNull()
    expect(s.querySelector('[part~="footer"]')).not.toBeNull()
    expect(s.querySelector('[part~="picker-btn"]')).not.toBeNull()
    expect(s.querySelector('[part~="create-btn"]')).not.toBeNull()
  })

  it('host exposes role=navigation + aria-label for assistive tech', async () => {
    const { el } = mount(reg, 'sui-rail')
    await flush()
    expect(el.getAttribute('role')).toBe('navigation')
    expect(el.getAttribute('aria-label')).toBe('Primary navigation')
  })

  it('active slotted item exposes aria-current=page on its anchor', async () => {
    history.pushState({}, '', '/agents')
    const html = '<sui-rail-item data-view="agents" label="Agents" icon="bot" href="/agents"></sui-rail-item>'
    const { el } = mount(reg, 'sui-rail', {}, html)
    await flush()
    await flush()
    const item = el.querySelector('sui-rail-item[data-view="agents"]') as HTMLElement
    const anchor = item.shadowRoot!.querySelector('a.item')!
    expect(anchor.getAttribute('aria-current')).toBe('page')
  })

  it('has no arrow toggle button — toggle is owned by the top-nav menu', async () => {
    const { el } = mount(reg, 'sui-rail')
    await flush()
    expect(shadow(el).querySelector('[part~="toggle"]')).toBeNull()
    expect(shadow(el).querySelector('.toggle-btn')).toBeNull()
  })

  it('collapsed=false by default — attribute absent', () => {
    const { el } = mount(reg, 'sui-rail')
    expect(el.hasAttribute('collapsed')).toBe(false)
  })

  it('reflects [collapsed] attribute when set', () => {
    const { el } = mount(reg, 'sui-rail', { collapsed: true })
    expect(el.hasAttribute('collapsed')).toBe(true)
  })

  it('has a default slot for child items', () => {
    const { el } = mount(reg, 'sui-rail', {}, '<sui-rail-item label="Test" icon="zap"></sui-rail-item>')
    const slot = shadow(el).querySelector<HTMLSlotElement>('slot:not([name])')!
    expect(slot).not.toBeNull()
    expect(slot.assignedElements().length).toBe(1)
  })
})

// Project switcher tests removed: the project-switcher + launcher-button
// affordances were lifted off the rail's footer when the rail was
// condensed (the picker-btn now opens a <sui-view-picker> that owns
// project switching, and the launcher is opened via the command bar).
// Switcher coverage lives in sui-view-picker tests; launcher coverage
// lives in sui-launcher tests.

// Mission Control rail item was removed earlier; the session-count
// polling + badge wiring went with it.

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-rail> create button', () => {
  beforeEach(() => {
    mockFetch({
      '/api/v1/projects': { projects: [], active: null },
    })
  })

  it('create button dispatches ship:create:open', async () => {
    const { el } = mount(reg, 'sui-rail')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('ship:create:open', (e) => events.push(e as CustomEvent))
    const btn = shadow(el).querySelector<HTMLButtonElement>('[part~="create-btn"]')!
    btn.click()
    await flush()
    expect(events).toHaveLength(1)
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-rail> active-view tracking', () => {
  beforeEach(() => {
    mockFetch({
      '/api/v1/projects': { projects: [], active: null },
    })
  })

  it('marks the slotted child matching the current path segment as active', async () => {
    history.pushState({}, '', '/agents')
    const html =
      '<sui-rail-item data-view="agents" label="Agents" icon="bot"></sui-rail-item>' +
      '<sui-rail-item data-view="git" label="Git" icon="git-branch"></sui-rail-item>'
    const { el } = mount(reg, 'sui-rail', {}, html)
    await flush()
    await flush()
    const agents = el.querySelector('sui-rail-item[data-view="agents"]')!
    const git = el.querySelector('sui-rail-item[data-view="git"]')!
    expect(agents.hasAttribute('active')).toBe(true)
    expect(git.hasAttribute('active')).toBe(false)
  })

  it('reacts to sui:navigate events', async () => {
    history.pushState({}, '', '/agents')
    const html = '<sui-rail-item data-view="agents"></sui-rail-item>'
    const { el } = mount(reg, 'sui-rail', {}, html)
    await flush()
    await flush()
    history.pushState({}, '', '/git')
    document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: '/git' } }))
    await flush()
    const agents = el.querySelector('sui-rail-item[data-view="agents"]')!
    expect(agents.hasAttribute('active')).toBe(false)
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<sui-rail-item>', () => {
  it('registers and renders an anchor with part=item in shadow DOM', () => {
    expect(customElements.get('sui-rail-item')).toBeDefined()
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

  it('renders label text and mirrors it as native title tooltip', async () => {
    const { el } = mount(reg, 'sui-rail-item', { label: 'Agents', icon: 'bot', href: '/agents' })
    await flush()
    expect(shadow(el).querySelector('.label')!.textContent?.trim()).toBe('Agents')
    expect(shadow(el).querySelector<HTMLAnchorElement>('a.item')!.getAttribute('title')).toBe('Agents')
  })

  it('passes icon name to sui-icon', () => {
    const { el } = mount(reg, 'sui-rail-item', { icon: 'settings', label: 'Settings', href: '/settings' })
    const icon = shadow(el).querySelector<HTMLElement>('sui-icon')!
    expect(icon.getAttribute('name')).toBe('settings')
  })

  it('active toggles aria-current=page on the anchor', async () => {
    const off = mount(reg, 'sui-rail-item', { href: '/', label: 'Home', icon: 'home' })
    expect(shadow(off.el).querySelector('a.item')!.hasAttribute('aria-current')).toBe(false)
    const on = mount(reg, 'sui-rail-item', { href: '/', label: 'Home', icon: 'home', active: true })
    await flush()
    expect(on.el.hasAttribute('active')).toBe(true)
    expect(shadow(on.el).querySelector('a.item')!.getAttribute('aria-current')).toBe('page')
  })

  it('badge shows badge text when non-empty', async () => {
    const { el } = mount(reg, 'sui-rail-item', { label: 'Inbox', icon: 'inbox', href: '/inbox', badge: '3' })
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
