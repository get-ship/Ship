// Behavioral spec for <ship-rail> — the chrome pack's left nav rail.
// @ship/chrome pack port of the shell's <sui-rail>; the rail owns
// pinned-views + project switcher + launcher/create triggers. Project
// data comes from `fetch` against a configurable list URL.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sail-primitives'
import railController from '../src/ship-rail/ship-rail.controller'

const RAIL_SRC = readFileSync(
  resolve(__dirname, '../src/ship-rail/ship-rail.sail'),
  'utf8',
)
const ITEM_SRC = readFileSync(
  resolve(__dirname, '../src/ship-rail/ship-rail-item.sail'),
  'utf8',
)
const RAIL_TAG = 'ship-rail'
const ITEM_TAG = 'ship-rail-item'

function mockFetch(routes: Record<string, unknown>): void {
  vi.spyOn(globalThis, 'fetch').mockImplementation(async (input: RequestInfo | URL) => {
    const url = typeof input === 'string' ? input : input.toString()
    const key = Object.keys(routes).find((k) => url.includes(k))
    const body = key ? routes[key] : null
    return new Response(JSON.stringify(body ?? {}), { status: 200, headers: { 'Content-Type': 'application/json' } })
  })
}

function mountRail(attrs: Record<string, string | boolean> = {}, innerHTML = '') {
  const c = renderComponent(RAIL_SRC, RAIL_TAG, { controller: railController, attrs })
  if (innerHTML) c.el.innerHTML = innerHTML
  return c
}

const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

afterEach(() => {
  // Strip any popovers we teleported to body during a test so the
  // next test starts from a clean document.
  for (const el of document.body.querySelectorAll('.ship-rail-project-popover')) {
    el.remove()
  }
  vi.restoreAllMocks()
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<ship-rail>', () => {
  beforeEach(() => {
    mockFetch({ '/api/v1/spaces': { spaces: [], activeId: null } })
  })

  it('registers the custom element', () => {
    mountRail().unmount()
    expect(customElements.get(RAIL_TAG)).toBeDefined()
  })

  it('renders nav, footer, picker, create parts', async () => {
    const c = mountRail()
    await c.flush()
    const s = root(c)
    expect(s.querySelector('[part~="nav"]')).not.toBeNull()
    expect(s.querySelector('[part~="footer"]')).not.toBeNull()
    expect(s.querySelector('[part~="picker-btn"]')).not.toBeNull()
    expect(s.querySelector('[part~="create-btn"]')).not.toBeNull()
    c.unmount()
  })

  it('host exposes role=navigation + aria-label for assistive tech', async () => {
    const c = mountRail()
    await c.flush()
    expect(c.el.getAttribute('role')).toBe('navigation')
    expect(c.el.getAttribute('aria-label')).toBe('Primary navigation')
    c.unmount()
  })

  it('active slotted item exposes aria-current=page on its anchor', async () => {
    history.pushState({}, '', '/agents')
    const c = mountRail(
      {},
      '<ship-rail-item data-view="agents" label="Agents" icon="bot" href="/agents"></ship-rail-item>',
    )
    await c.flush()
    await c.flush()
    const item = c.el.querySelector(ITEM_TAG + '[data-view="agents"]') as HTMLElement
    const anchor = item.shadowRoot!.querySelector('a.item')!
    expect(anchor.getAttribute('aria-current')).toBe('page')
    c.unmount()
  })

  it('has no arrow toggle button — toggle is owned by the top-nav menu', async () => {
    const c = mountRail()
    await c.flush()
    expect(root(c).querySelector('[part~="toggle"]')).toBeNull()
    expect(root(c).querySelector('.toggle-btn')).toBeNull()
    c.unmount()
  })

  it('collapsed=false by default — attribute absent', () => {
    const c = mountRail()
    expect(c.el.hasAttribute('collapsed')).toBe(false)
    c.unmount()
  })

  it('reflects [collapsed] attribute when set', () => {
    const c = mountRail({ collapsed: true })
    expect(c.el.hasAttribute('collapsed')).toBe(true)
    c.unmount()
  })

  it('has a default slot for child items', async () => {
    const c = mountRail({}, `<${ITEM_TAG} label="Test" icon="zap"></${ITEM_TAG}>`)
    await c.flush()
    const slot = root(c).querySelector<HTMLSlotElement>('slot:not([name])')!
    expect(slot).not.toBeNull()
    expect(slot.assignedElements().length).toBe(1)
    c.unmount()
  })

  it('fetch failures leave projects signal at its initial value (no throw)', async () => {
    vi.restoreAllMocks()
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('network down'))
    const c = mountRail()
    await c.flush()
    // No popover mounted when projects is empty — sanity-check the
    // signal path didn't throw.
    expect(root(c).querySelector('.ship-rail-project-popover')).toBeNull()
    c.unmount()
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<ship-rail> create button', () => {
  beforeEach(() => {
    mockFetch({ '/api/v1/spaces': { spaces: [], activeId: null } })
  })

  it('create button dispatches ship:create:open', async () => {
    const c = mountRail()
    await c.flush()
    const events: CustomEvent[] = []
    c.el.addEventListener('ship:create:open', (e) => events.push(e as CustomEvent))
    const btn = root(c).querySelector<HTMLButtonElement>('[part~="create-btn"]')!
    btn.click()
    await c.flush()
    expect(events).toHaveLength(1)
    c.unmount()
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<ship-rail> active-view tracking', () => {
  beforeEach(() => {
    mockFetch({ '/api/v1/spaces': { spaces: [], activeId: null } })
  })

  it('marks the slotted child matching the current path segment as active', async () => {
    history.pushState({}, '', '/agents')
    const c = mountRail(
      {},
      `<${ITEM_TAG} data-view="agents" label="Agents" icon="bot"></${ITEM_TAG}>` +
      `<${ITEM_TAG} data-view="git" label="Git" icon="git-branch"></${ITEM_TAG}>`,
    )
    await c.flush()
    await c.flush()
    const agents = c.el.querySelector(ITEM_TAG + '[data-view="agents"]')!
    const git = c.el.querySelector(ITEM_TAG + '[data-view="git"]')!
    expect(agents.hasAttribute('active')).toBe(true)
    expect(git.hasAttribute('active')).toBe(false)
    c.unmount()
  })

  it('reacts to sui:navigate events', async () => {
    history.pushState({}, '', '/agents')
    const c = mountRail(
      {},
      `<${ITEM_TAG} data-view="agents"></${ITEM_TAG}>`,
    )
    await c.flush()
    await c.flush()
    history.pushState({}, '', '/git')
    document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: '/git' } }))
    await c.flush()
    const agents = c.el.querySelector(ITEM_TAG + '[data-view="agents"]')!
    expect(agents.hasAttribute('active')).toBe(false)
    c.unmount()
  })
})

// ─────────────────────────────────────────────────────────────────────────────
describe('<ship-rail-item>', () => {
  it('registers the custom element', () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG)
    c.unmount()
    expect(customElements.get(ITEM_TAG)).toBeDefined()
  })

  it('renders an anchor with part=item in shadow DOM', () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { label: 'Dashboard', icon: 'layout-dashboard', href: '/' } })
    const a = (c.el.shadowRoot ?? c.el).querySelector('a.item')
    expect(a).not.toBeNull()
    expect(a!.getAttribute('part')).toBe('item')
    c.unmount()
  })

  it('reflects href on the anchor', () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { href: '/pipelines', label: 'Pipelines', icon: 'zap' } })
    const a = (c.el.shadowRoot ?? c.el).querySelector<HTMLAnchorElement>('a.item')!
    expect(a.getAttribute('href')).toBe('/pipelines')
    c.unmount()
  })

  it('renders label text and mirrors it as native title tooltip', async () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { label: 'Agents', icon: 'bot', href: '/agents' } })
    await c.flush()
    const s = c.el.shadowRoot ?? c.el
    expect(s.querySelector('.label')!.textContent?.trim()).toBe('Agents')
    expect(s.querySelector<HTMLAnchorElement>('a.item')!.getAttribute('title')).toBe('Agents')
    c.unmount()
  })

  it('passes icon name to sui-icon', () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { icon: 'settings', label: 'Settings', href: '/settings' } })
    const icon = (c.el.shadowRoot ?? c.el).querySelector<HTMLElement>('sui-icon')!
    expect(icon.getAttribute('name')).toBe('settings')
    c.unmount()
  })

  it('active toggles aria-current=page on the anchor', async () => {
    const off = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { href: '/', label: 'Home', icon: 'home' } })
    expect((off.el.shadowRoot ?? off.el).querySelector('a.item')!.hasAttribute('aria-current')).toBe(false)
    off.unmount()
    const on = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { href: '/', label: 'Home', icon: 'home', active: true } })
    await on.flush()
    expect(on.el.hasAttribute('active')).toBe(true)
    expect((on.el.shadowRoot ?? on.el).querySelector('a.item')!.getAttribute('aria-current')).toBe('page')
    on.unmount()
  })

  it('badge shows badge text when non-empty', async () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { label: 'Inbox', icon: 'inbox', href: '/inbox', badge: '3' } })
    await c.flush()
    const badge = (c.el.shadowRoot ?? c.el).querySelector('.badge')!
    expect(badge.textContent?.trim()).toBe('3')
    c.unmount()
  })

  it('disabled reflects [disabled] and sets aria-disabled on anchor', () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { href: '/', label: 'Home', icon: 'home', disabled: true } })
    expect(c.el.hasAttribute('disabled')).toBe(true)
    const a = (c.el.shadowRoot ?? c.el).querySelector('a.item')!
    expect(a.getAttribute('aria-disabled')).toBe('true')
    c.unmount()
  })

  it('has part=icon-wrap, part=label, part=badge', () => {
    const c = renderComponent(ITEM_SRC, ITEM_TAG, { attrs: { label: 'X', icon: 'x', href: '/' } })
    const s = c.el.shadowRoot ?? c.el
    expect(s.querySelector('[part~="icon-wrap"]')).not.toBeNull()
    expect(s.querySelector('[part~="label"]')).not.toBeNull()
    expect(s.querySelector('[part~="badge"]')).not.toBeNull()
    c.unmount()
  })
})
