import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-error-state/ship-error-state.compiled.js'
// ship-error-state renders <sui-icon> from @ship/sail-primitives in its tile —
// it must register before the host mounts, otherwise the shadow child renders
// as an unknown element. The shared pack documents this dependency; tests must
// mirror the runtime contract.
import '../../../sail/primitives/src/sui-icon/sui-icon.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// ship-error-state is RUNTIME-FREE and purely presentational: it has no
// controller, dispatches no events, and owns no navigation. Recovery is the
// host's job — supplied declaratively via `href` (an SPA link) or via the
// `action` slot. These tests assert the rendered surface only.

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<ship-error-state>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-error-state')).toBeDefined()
  })

  it('renders the card with role=alert', () => {
    const { el } = mount(reg, 'ship-error-state')
    const c = (el.shadowRoot ?? el).querySelector('.card')
    expect(c).not.toBeNull()
    expect(c!.getAttribute('role')).toBe('alert')
  })

  it('defaults to variant=not-found and shows a "Not found" title', async () => {
    const { el } = mount(reg, 'ship-error-state')
    await flush()
    expect((el as HTMLElement & { variant: string }).variant).toBe('not-found')
    expect((el.shadowRoot ?? el).querySelector('.title')?.textContent?.trim()).toBe('Not found')
  })

  it('variant=error shows "Something went wrong"', async () => {
    const { el } = mount(reg, 'ship-error-state', { variant: 'error' })
    await flush()
    expect((el.shadowRoot ?? el).querySelector('.title')?.textContent?.trim()).toBe('Something went wrong')
  })

  it('uses the title prop when set', async () => {
    const { el } = mount(reg, 'ship-error-state', { title: 'Custom heading' })
    await flush()
    expect((el.shadowRoot ?? el).querySelector('.title')?.textContent?.trim()).toBe('Custom heading')
  })

  it('reflects variant and title attributes back to the host', () => {
    const { el } = mount(reg, 'ship-error-state', {
      title: 'X',
      variant: 'error',
    }) as { el: HTMLElement & { title: string; variant: string } }
    expect(el.getAttribute('variant')).toBe('error')
    expect(el.variant).toBe('error')
    expect(el.title).toBe('X')
  })

  it('renders a default body when none is provided', async () => {
    const { el } = mount(reg, 'ship-error-state')
    await flush()
    expect((el.shadowRoot ?? el).querySelector('.body')?.textContent?.trim().length).toBeGreaterThan(0)
  })

  it('renders no built-in primary action when href is empty', () => {
    const { el } = mount(reg, 'ship-error-state')
    expect((el.shadowRoot ?? el).querySelector('.primary-action')).toBeNull()
  })

  it('renders an anchor when href is set', () => {
    const { el } = mount(reg, 'ship-error-state', { href: '/somewhere' })
    const p = (el.shadowRoot ?? el).querySelector('.primary-action')
    expect(p).not.toBeNull()
    expect(p!.tagName).toBe('A')
    expect((p as HTMLAnchorElement).getAttribute('href')).toBe('/somewhere')
  })

  it('uses action-label to override the primary link copy', () => {
    const { el } = mount(reg, 'ship-error-state', {
      href: '/',
      'action-label': 'Go home',
    })
    expect((el.shadowRoot ?? el).querySelector('.primary-action')?.textContent?.trim()).toBe('Go home')
  })

  it('renders a sui-icon in the tile', () => {
    const { el } = mount(reg, 'ship-error-state')
    const tile = (el.shadowRoot ?? el).querySelector('.tile')
    expect(tile).not.toBeNull()
    expect(tile!.querySelector('sui-icon')).not.toBeNull()
  })

  it('error variant swaps the icon name to alert-octagon', () => {
    const { el } = mount(reg, 'ship-error-state', { variant: 'error' })
    expect((el.shadowRoot ?? el).querySelector('sui-icon')?.getAttribute('name')).toBe('alert-octagon')
  })

  it('not-found variant keeps the map-pin-off icon', () => {
    const { el } = mount(reg, 'ship-error-state', { variant: 'not-found' })
    expect((el.shadowRoot ?? el).querySelector('sui-icon')?.getAttribute('name')).toBe('map-pin-off')
  })

  it('passes a title slot through', () => {
    const { el } = mount(
      reg,
      'ship-error-state',
      {},
      '<span slot="title">Slotted title</span>',
    )
    const slot = (el.shadowRoot ?? el).querySelector('slot[name="title"]')
    expect(slot).not.toBeNull()
    expect(el.querySelector('[slot="title"]')?.textContent).toBe('Slotted title')
  })

  it('passes a default body slot through', () => {
    const { el } = mount(reg, 'ship-error-state', {}, '<p>Custom body</p>')
    expect(el.querySelector('p')?.textContent).toBe('Custom body')
  })

  it('projects a host-supplied recovery action via the action slot', () => {
    const { el } = mount(
      reg,
      'ship-error-state',
      {},
      '<button slot="action">Custom action</button>',
    )
    const slot = (el.shadowRoot ?? el).querySelector('slot[name="action"]')
    expect(slot).not.toBeNull()
    expect(el.querySelector('[slot="action"]')?.textContent).toBe('Custom action')
  })
})
