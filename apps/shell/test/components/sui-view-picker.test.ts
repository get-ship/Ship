import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
// sui-popover / sui-portal / sui-icon / sui-app-icon live in @ship/sui-primitives.
import '@ship/sui-primitives'
import '../../src/components/sui-view-picker/sui-view-picker.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()
afterEach(() => {
  reg.cleanup()
  vi.restoreAllMocks()
})

const shadow = (el: HTMLElement) => el.shadowRoot!

// Generic stub for the ConnectRPC ops wire (SettingsService/SpaceService)
// the picker's data layer calls on open — empty-success everything so
// load() settles without touching the network.
function mockFetch(): void {
  vi.spyOn(globalThis, 'fetch').mockImplementation(async () =>
    new Response('{}', { status: 200, headers: { 'Content-Type': 'application/json' } }))
}

describe('<sui-view-picker>', () => {
  beforeEach(() => {
    mockFetch()
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

  // The row-painting / filter / row-click / drag-pin tests were removed:
  // they asserted rows fed by the /api/v1/views catalog, and the daemon
  // view-discovery op was quarantined (06153c1a5) — `fetchViews()` now
  // intentionally returns []. Re-add behavioral coverage when the
  // router→view-list integration lands and rows have a real data source.

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
