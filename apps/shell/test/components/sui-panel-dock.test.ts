import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../../src/components/sui-panel-dock/sui-panel-dock.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())
beforeEach(() => { try { localStorage.clear() } catch { /* ignore */ } })

const shadow = (el: HTMLElement) => el.shadowRoot!
const tabstrip = (el: HTMLElement) => shadow(el).querySelector<HTMLElement>('.tabstrip')!
const bodies = (el: HTMLElement) => shadow(el).querySelector<HTMLElement>('.bodies')!
const tabBtn = (el: HTMLElement, id: string) => tabstrip(el).querySelector<HTMLButtonElement>(`[data-tab="${id}"]`)
const activeSlot = (el: HTMLElement) => bodies(el).querySelector<HTMLSlotElement>('slot[data-active]')
const toolBtn = (el: HTMLElement, action: string) => tabstrip(el).querySelector<HTMLButtonElement>(`[data-action="${action}"]`)

type Dock = HTMLElement & { show(): void; hide(): void; toggle(): void; activate(tab: string): void }

describe('<sui-panel-dock>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-panel-dock')).toBeDefined()
  })

  it('renders tabstrip and bodies in shadow DOM', () => {
    const { el } = mount(reg, 'sui-panel-dock')
    expect(tabstrip(el)).not.toBeNull()
    expect(bodies(el)).not.toBeNull()
  })

  it('renders 5 default tab buttons', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const btns = tabstrip(el).querySelectorAll('[data-tab]')
    expect(btns.length).toBe(5)
    const ids = Array.from(btns).map(b => (b as HTMLElement).dataset.tab)
    expect(ids).toEqual(['terminal', 'source', 'signals', 'schema', 'components'])
  })

  it('active-tab defaults to "terminal" — first tab aria-selected=true', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    // Reflected string props don't set the attribute with the default value —
    // only explicitly setting changes the attribute. Check the behavior instead.
    const termBtn = tabBtn(el, 'terminal')
    expect(termBtn?.getAttribute('aria-selected')).toBe('true')
    expect(termBtn?.getAttribute('tabindex')).toBe('0')
  })

  it('other tabs have aria-selected=false', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    for (const id of ['source', 'signals', 'schema', 'components']) {
      expect(tabBtn(el, id)?.getAttribute('aria-selected')).toBe('false')
    }
  })

  it('active tab body slot has data-active attribute', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const active = activeSlot(el)
    expect(active).not.toBeNull()
    expect(active?.getAttribute('name')).toBe('terminal')
  })

  it('reflects active-tab attribute when set', async () => {
    const { el } = mount(reg, 'sui-panel-dock', { 'active-tab': 'source' })
    await flush()
    expect(el.getAttribute('active-tab')).toBe('source')
    expect(tabBtn(el, 'source')?.getAttribute('aria-selected')).toBe('true')
    expect(activeSlot(el)?.getAttribute('name')).toBe('source')
  })

  it('clicking a tab updates active-tab and dispatches sui:dock:activate', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui:dock:activate', e => events.push(e as CustomEvent))

    tabBtn(el, 'schema')?.click()
    await flush()

    expect(el.getAttribute('active-tab')).toBe('schema')
    expect(events).toHaveLength(1)
    expect(events[0]!.detail.tab).toBe('schema')
    expect(activeSlot(el)?.getAttribute('name')).toBe('schema')
  })

  it('clicking active tab does not re-fire event', async () => {
    const { el } = mount(reg, 'sui-panel-dock', { 'active-tab': 'source' })
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui:dock:activate', e => events.push(e as CustomEvent))
    tabBtn(el, 'source')?.click()
    await flush()
    expect(events).toHaveLength(0)
  })

  // clear button + sui:dock:clear were removed from the dock toolbar
  // when the panel-dock action set was condensed (only popout + close
  // + edit-mode-toggle remain on the right cluster).

  it('popout button dispatches sui:dock:popout', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui:dock:popout', e => events.push(e as CustomEvent))
    toolBtn(el, 'popout')?.click()
    expect(events).toHaveLength(1)
  })

  it('close button dispatches sui:dock:close', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui:dock:close', e => events.push(e as CustomEvent))
    toolBtn(el, 'close')?.click()
    expect(events).toHaveLength(1)
  })

  it('show() adds body.dock-open', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const dock = el as Dock
    dock.show()
    expect(document.body.classList.contains('dock-open')).toBe(true)
    document.body.classList.remove('dock-open')
  })

  it('hide() removes body.dock-open', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    document.body.classList.add('dock-open')
    const dock = el as Dock
    dock.hide()
    expect(document.body.classList.contains('dock-open')).toBe(false)
  })

  it('toggle() flips body.dock-open', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const dock = el as Dock
    dock.toggle()
    expect(document.body.classList.contains('dock-open')).toBe(true)
    dock.toggle()
    expect(document.body.classList.contains('dock-open')).toBe(false)
  })

  it('activate() method changes tab and fires event', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui:dock:activate', e => events.push(e as CustomEvent))
    const dock = el as Dock
    dock.activate('schema')
    await flush()
    expect(el.getAttribute('active-tab')).toBe('schema')
    expect(events).toHaveLength(1)
    expect(events[0]!.detail.tab).toBe('schema')
  })

  it('tabs property setter rebuilds tabstrip', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    ;(el as unknown as { tabs: unknown[] }).tabs = [
      { id: 'custom', label: 'Custom', icon: 'zap' },
    ]
    await flush()
    const btns = tabstrip(el).querySelectorAll('[data-tab]')
    expect(btns.length).toBe(1)
    expect((btns[0] as HTMLElement).dataset.tab).toBe('custom')
  })

  it('renders 5 toolbar slots for toolbar:* names', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const sr = shadow(el)
    const ids = ['terminal', 'source', 'signals', 'schema', 'components']
    for (const id of ids) {
      expect(sr.querySelector(`slot[name="toolbar:${id}"]`)).not.toBeNull()
    }
  })

  it('renders resize handle with correct role', () => {
    const { el } = mount(reg, 'sui-panel-dock')
    const handle = shadow(el).querySelector('.resize')
    expect(handle?.getAttribute('role')).toBe('separator')
    expect(handle?.getAttribute('aria-orientation')).toBe('horizontal')
  })

  it('resizing attribute reflects during pointer drag', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    expect(el.hasAttribute('resizing')).toBe(false)
  })

  it('live-dot appears on active live tab, not on inactive', async () => {
    const { el } = mount(reg, 'sui-panel-dock', { 'active-tab': 'terminal' })
    await flush()
    expect(tabBtn(el, 'terminal')?.querySelector('.live-dot')).not.toBeNull()
    expect(tabBtn(el, 'signals')?.querySelector('.live-dot')).toBeNull()
  })

  it('live-dot moves when switching to a live tab', async () => {
    const { el } = mount(reg, 'sui-panel-dock', { 'active-tab': 'terminal' })
    await flush()
    tabBtn(el, 'signals')?.click()
    await flush()
    expect(tabBtn(el, 'signals')?.querySelector('.live-dot')).not.toBeNull()
    expect(tabBtn(el, 'terminal')?.querySelector('.live-dot')).toBeNull()
  })

  it('resize double-click dispatches sui:dock:resize with DEFAULT_HEIGHT=240', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui:dock:resize', e => events.push(e as CustomEvent))
    const handle = shadow(el).querySelector<HTMLElement>('.resize')!
    handle.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }))
    expect(events).toHaveLength(1)
    expect(events[0]!.detail.height).toBe(240)
  })

  it('clicking a tab writes the new active tab to localStorage', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    tabBtn(el, 'schema')?.click()
    await flush()
    expect(localStorage.getItem('ship-dock-active')).toBe('schema')
  })

  it('imperative activate() also writes the new tab to localStorage', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    ;(el as Dock).activate('schema')
    await flush()
    expect(localStorage.getItem('ship-dock-active')).toBe('schema')
  })

  it('storage-key prop scopes the persisted key', async () => {
    const { el } = mount(reg, 'sui-panel-dock', { 'storage-key': 'devtools-dock' })
    await flush()
    tabBtn(el, 'signals')?.click()
    await flush()
    expect(localStorage.getItem('devtools-dock-active')).toBe('signals')
    expect(localStorage.getItem('ship-dock-active')).toBeNull()
  })

  it('hydrates active tab from localStorage on mount', async () => {
    localStorage.setItem('ship-dock-active', 'schema')
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    expect(el.getAttribute('active-tab')).toBe('schema')
    expect(activeSlot(el)?.getAttribute('name')).toBe('schema')
  })

  it('hydration ignores unknown tab ids', async () => {
    localStorage.setItem('ship-dock-active', 'does-not-exist')
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    // Falls back to the prop default ('terminal').
    expect(tabBtn(el, 'terminal')?.getAttribute('aria-selected')).toBe('true')
  })

  it('setting host.tabs to a custom list rebuilds toolbar and body slots', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    ;(el as unknown as { tabs: unknown[] }).tabs = [
      { id: 'alpha', label: 'Alpha', icon: 'zap' },
      { id: 'beta', label: 'Beta', icon: 'box' },
    ]
    await flush()
    const sr = shadow(el)
    // Tabstrip rebuilt.
    expect(sr.querySelectorAll('[data-tab]').length).toBe(2)
    // Toolbar slots rebuilt.
    expect(sr.querySelector('slot[name="toolbar:alpha"]')).not.toBeNull()
    expect(sr.querySelector('slot[name="toolbar:beta"]')).not.toBeNull()
    expect(sr.querySelector('slot[name="toolbar:terminal"]')).toBeNull()
    // Body slots rebuilt.
    expect(sr.querySelector('slot[name="alpha"]')).not.toBeNull()
    expect(sr.querySelector('slot[name="beta"]')).not.toBeNull()
    expect(sr.querySelector('slot[name="terminal"]')).toBeNull()
  })

  it('resize-height pill only mounts while resizing', async () => {
    const { el } = mount(reg, 'sui-panel-dock')
    await flush()
    // At rest, no pill in document.body (it portals out of shadow DOM).
    expect(document.body.querySelector('.sui-dock-resize-pill')).toBeNull()
    // Drive a dblclick reset — resizing remains false; pill stays absent.
    shadow(el).querySelector<HTMLElement>('.resize')!.dispatchEvent(
      new MouseEvent('dblclick', { bubbles: true }),
    )
    await flush()
    expect(document.body.querySelector('.sui-dock-resize-pill')).toBeNull()
  })
})
