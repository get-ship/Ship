import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
// sui-popover / sui-portal / sui-icon / sui-app-icon live in @ship/sui-primitives.
import '@ship/sui-primitives'
import '../../src/components/sui-view-picker/sui-view-picker.compiled.js'
import { createContainerRegistry, mount } from '../test-utils'

// flush() advances fake timers by 0 ms — equivalent to the real
// `setTimeout(r, 0)` flush from test-utils but compatible with the
// fake-timer setup in beforeEach/afterEach below.
const flush = () => vi.advanceTimersByTimeAsync(0)

const reg = createContainerRegistry()
// Fake timers: the view-picker's filter debounce (60 ms) is a real
// `setTimeout` call in the controller. Advancing fake timers makes it
// fire instantaneously and deterministically rather than relying on
// wall-clock waits that flake under load.
beforeEach(() => { vi.useFakeTimers() })
afterEach(() => {
  reg.cleanup()
  vi.restoreAllMocks()
  vi.useRealTimers()
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

describe('<sui-view-picker>', () => {
  beforeEach(() => {
    mockFetch({
      '/api/v1/settings': { data: { 'shell.rail.pinned': '["agents","git"]' } },
      '/api/v1/projects': {
        projects: [{ id: 'a', name: 'Alpha', path: '/work/alpha' }],
        active: 'a',
      },
      '/api/v1/views': {
        data: {
          views: [
            { name: 'agents', type: 'view', source: 'views/agents/page.sui' },
            { name: 'git', type: 'view', source: 'views/git/page.sui' },
            { name: 'skills', type: 'view', source: 'views/skills/page.sui' },
            { name: 'settings', type: 'view', source: 'views/settings/page.sui' },
          ],
        },
      },
    })
  })

  it('registers the custom element', () => {
    expect(customElements.get('sui-view-picker')).toBeDefined()
  })

  it('renders header, filter, body, footer parts in shadow DOM', () => {
    const { el } = mount(reg, 'sui-view-picker')
    expect(shadow(el).querySelector('[part~="header"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="filter"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="body"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="footer"]')).not.toBeNull()
  })

  it('filter input has correct placeholder and aria-label', () => {
    const { el } = mount(reg, 'sui-view-picker')
    const input = shadow(el).querySelector<HTMLInputElement>('.vp-filter input')!
    expect(input.getAttribute('placeholder')).toBe('Filter views…')
    expect(input.getAttribute('aria-label')).toBe('Filter views')
  })

  it('install-href prop defaults to /studio/components', () => {
    const { el } = mount(reg, 'sui-view-picker')
    const a = shadow(el).querySelector<HTMLAnchorElement>('[part~="install-link"]')!
    expect(a.getAttribute('href')).toBe('/studio/components')
  })

  it('install-href prop overrides default', async () => {
    const { el } = mount(reg, 'sui-view-picker', { 'install-href': '/install' })
    await flush()
    const a = shadow(el).querySelector<HTMLAnchorElement>('[part~="install-link"]')!
    expect(a.getAttribute('href')).toBe('/install')
  })

  it('opens on open prop set, loads data, paints pinned + others', async () => {
    const { el } = mount(reg, 'sui-view-picker', { open: true })
    // Wait for fetch microtasks + effects.
    await flush(); await flush(); await flush()
    const body = shadow(el).querySelector('[part~="body"]')!
    expect(body.querySelectorAll('.vp-row').length).toBeGreaterThan(0)
    // Pinned section should contain 'agents' + 'git' (per mock pins).
    const pinned = body.querySelector('[part~="section-pinned"]')!
    expect(pinned.querySelector('[data-view="agents"]')).not.toBeNull()
    expect(pinned.querySelector('[data-view="git"]')).not.toBeNull()
    // 'skills' should be in the all-views section.
    const all = body.querySelector('[part~="section-all"]')!
    expect(all.querySelector('[data-view="skills"]')).not.toBeNull()
  })

  it('filter narrows visible rows in real-time', async () => {
    const { el } = mount(reg, 'sui-view-picker', { open: true })
    await flush(); await flush(); await flush()
    const input = shadow(el).querySelector<HTMLInputElement>('.vp-filter input')!
    input.value = 'skill'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    // Advance past the 60 ms filter debounce + drain queued 0 ms timers.
    await vi.advanceTimersByTimeAsync(100)
    await flush(); await flush()
    const body = shadow(el).querySelector('[part~="body"]')!
    const rows = Array.from(body.querySelectorAll<HTMLElement>('.vp-row'))
    expect(rows.some((r) => r.dataset['view'] === 'skills')).toBe(true)
    expect(rows.some((r) => r.dataset['view'] === 'agents')).toBe(false)
  })

  it('clicking a row dispatches sui:view-picker:navigate and closes', async () => {
    const { el } = mount(reg, 'sui-view-picker', { open: true })
    await flush(); await flush(); await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui:view-picker:navigate', (e) => events.push(e as CustomEvent))
    const row = shadow(el).querySelector<HTMLElement>('[data-view="skills"]')!
    row.click()
    await flush()
    expect(events).toHaveLength(1)
    expect(events[0].detail.name).toBe('skills')
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('drag from all-views onto a pinned row pins it', async () => {
    const { el } = mount(reg, 'sui-view-picker', { open: true })
    await flush(); await flush(); await flush()
    const pinEvents: CustomEvent[] = []
    el.addEventListener('sui:view-picker:pin-changed', (e) => pinEvents.push(e as CustomEvent))
    const src = shadow(el).querySelector<HTMLElement>('[data-section="all"][data-view="skills"]')!
    const tgt = shadow(el).querySelector<HTMLElement>('[data-section="pinned"][data-view="agents"]')!
    // jsdom doesn't construct DragEvent/DataTransfer fully — fake the
    // dataTransfer surface our handlers actually read.
    const dt = { setData() {}, dropEffect: '', effectAllowed: '' } as unknown as DataTransfer
    const fire = (target: HTMLElement, type: string) => {
      const ev = new Event(type, { bubbles: true, cancelable: true }) as DragEvent
      Object.defineProperty(ev, 'dataTransfer', { value: dt, writable: false })
      target.dispatchEvent(ev)
    }
    fire(src, 'dragstart')
    fire(tgt, 'dragover')
    fire(tgt, 'drop')
    await flush(); await flush()
    expect(pinEvents.length).toBeGreaterThanOrEqual(1)
    const last = pinEvents[pinEvents.length - 1].detail.pins as string[]
    expect(last).toContain('skills')
  })

  it('refresh() reloads data', async () => {
    const { el } = mount(reg, 'sui-view-picker', { open: true })
    await flush(); await flush(); await flush()
    const fetchSpy = vi.spyOn(globalThis, 'fetch')
    const before = fetchSpy.mock.calls.length
    ;(el as unknown as { refresh: () => Promise<void> }).refresh()
    await flush(); await flush()
    expect(fetchSpy.mock.calls.length).toBeGreaterThan(before)
  })

  it('trigger slot is exposed (named slot=trigger inside composed popover)', () => {
    const { el } = mount(reg, 'sui-view-picker')
    const slot = shadow(el).querySelector<HTMLSlotElement>('sui-popover')
    expect(slot).not.toBeNull()
  })
})
