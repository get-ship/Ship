import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-error-state/ship-error-state.compiled.js'
// ship-error-state composes <sui-error-state> from @ship/sui-primitives AND
// <sui-icon> — both must register before the host mounts, otherwise the
// shadow children render as unknown elements. The shared pack documents this
// dependency; tests must mirror the runtime contract.
import '../../../sui/primitives/src/sui-icon/sui-icon.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<ship-error-state>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-error-state')).toBeDefined()
  })

  it('renders the card with role=alert', () => {
    const { el } = mount(reg, 'ship-error-state')
    const c = el.shadowRoot!.querySelector('.card')
    expect(c).not.toBeNull()
    expect(c!.getAttribute('role')).toBe('alert')
  })

  it('defaults to variant=not-found and shows a "Not found" title', async () => {
    const { el } = mount(reg, 'ship-error-state')
    await flush()
    expect((el as HTMLElement & { variant: string }).variant).toBe('not-found')
    expect(el.shadowRoot!.querySelector('.title')?.textContent?.trim()).toBe('Not found')
  })

  it('variant=error shows "Something went wrong"', async () => {
    const { el } = mount(reg, 'ship-error-state', { variant: 'error' })
    await flush()
    expect(el.shadowRoot!.querySelector('.title')?.textContent?.trim()).toBe('Something went wrong')
  })

  it('uses the title prop when set', async () => {
    const { el } = mount(reg, 'ship-error-state', { title: 'Custom heading' })
    await flush()
    expect(el.shadowRoot!.querySelector('.title')?.textContent?.trim()).toBe('Custom heading')
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
    expect(el.shadowRoot!.querySelector('.body')?.textContent?.trim().length).toBeGreaterThan(0)
  })

  it('renders a primary-action button by default', () => {
    const { el } = mount(reg, 'ship-error-state')
    const p = el.shadowRoot!.querySelector('.primary-action')
    expect(p).not.toBeNull()
    expect(p!.tagName).toBe('BUTTON')
    expect(p!.textContent?.trim()).toBe('Go back')
  })

  it('renders an anchor when href is set', () => {
    const { el } = mount(reg, 'ship-error-state', { href: '/somewhere' })
    const p = el.shadowRoot!.querySelector('.primary-action')
    expect(p).not.toBeNull()
    expect(p!.tagName).toBe('A')
    expect((p as HTMLAnchorElement).getAttribute('href')).toBe('/somewhere')
  })

  it('uses action-label to override the primary action copy', () => {
    const { el } = mount(reg, 'ship-error-state', {
      'action-label': 'Try again',
    })
    expect(el.shadowRoot!.querySelector('.primary-action')?.textContent?.trim()).toBe('Try again')
  })

  it('clicking the primary action emits ship-error-state:retry', async () => {
    const { el } = mount(reg, 'ship-error-state')
    const handler = vi.fn()
    el.addEventListener('ship-error-state:retry', handler)
    const backSpy = vi
      .spyOn(history, 'back')
      .mockImplementation(() => undefined)
    el.shadowRoot!.querySelector('.primary-action')?.dispatchEvent(
      new MouseEvent('click', { bubbles: true }),
    )
    await flush()
    expect(handler).toHaveBeenCalledTimes(1)
    expect(backSpy).toHaveBeenCalled()
    backSpy.mockRestore()
  })

  it('falls back to location.assign("/") when history.back throws', async () => {
    const { el } = mount(reg, 'ship-error-state')
    const assignSpy = vi
      .spyOn(location, 'assign')
      .mockImplementation(() => undefined)
    const backSpy = vi
      .spyOn(history, 'back')
      .mockImplementation(() => {
        throw new Error('no history')
      })
    el.shadowRoot!.querySelector('.primary-action')?.dispatchEvent(
      new MouseEvent('click', { bubbles: true }),
    )
    await flush()
    expect(backSpy).toHaveBeenCalled()
    expect(assignSpy).toHaveBeenCalledWith('/')
    backSpy.mockRestore()
    assignSpy.mockRestore()
  })

  it('renders a sui-icon in the tile', () => {
    const { el } = mount(reg, 'ship-error-state')
    const tile = el.shadowRoot!.querySelector('.tile')
    expect(tile).not.toBeNull()
    expect(tile!.querySelector('sui-icon')).not.toBeNull()
  })

  it('error variant swaps the icon name to alert-octagon', () => {
    const { el } = mount(reg, 'ship-error-state', { variant: 'error' })
    expect(el.shadowRoot!.querySelector('sui-icon')?.getAttribute('name')).toBe('alert-octagon')
  })

  it('not-found variant keeps the map-pin-off icon', () => {
    const { el } = mount(reg, 'ship-error-state', { variant: 'not-found' })
    expect(el.shadowRoot!.querySelector('sui-icon')?.getAttribute('name')).toBe('map-pin-off')
  })

  it('passes a title slot through', () => {
    const { el } = mount(
      reg,
      'ship-error-state',
      {},
      '<span slot="title">Slotted title</span>',
    )
    const slot = el.shadowRoot!.querySelector('slot[name="title"]')
    expect(slot).not.toBeNull()
    expect(el.querySelector('[slot="title"]')?.textContent).toBe('Slotted title')
  })

  it('passes a default body slot through', () => {
    const { el } = mount(reg, 'ship-error-state', {}, '<p>Custom body</p>')
    expect(el.querySelector('p')?.textContent).toBe('Custom body')
  })

  it('marks data-action-from-slot when the action slot has content', async () => {
    const { el } = mount(
      reg,
      'ship-error-state',
      {},
      '<button slot="action">Custom action</button>',
    )
    await flush()
    expect(el.hasAttribute('data-action-from-slot')).toBe(true)
  })

  it('does not mark data-action-from-slot when the action slot is empty', async () => {
    const { el } = mount(reg, 'ship-error-state')
    await flush()
    expect(el.hasAttribute('data-action-from-slot')).toBe(false)
  })

  it('emits ship-error-state:retry with bubbles + composed for host listeners', async () => {
    const { el } = mount(reg, 'ship-error-state')
    let captured: CustomEvent | null = null
    el.addEventListener('ship-error-state:retry', (e) => {
      captured = e as CustomEvent
    })
    const backSpy = vi
      .spyOn(history, 'back')
      .mockImplementation(() => undefined)
    el.shadowRoot!.querySelector('.primary-action')?.dispatchEvent(
      new MouseEvent('click', { bubbles: true }),
    )
    await flush()
    expect(captured).not.toBeNull()
    expect(captured!.bubbles).toBe(true)
    expect(captured!.composed).toBe(true)
    backSpy.mockRestore()
  })
})
