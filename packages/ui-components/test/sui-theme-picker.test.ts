import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import '../src/sui-theme-picker/dist/sui-theme-picker.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
beforeEach(() => {
  try { localStorage.clear() } catch { /* jsdom */ }
})
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const trigger = (el: HTMLElement) =>
  shadow(el).querySelector<HTMLButtonElement>('.trigger')!
const panel = (el: HTMLElement) =>
  shadow(el).querySelector('.panel') as HTMLElement & {
    showPopover?: () => void
    hidePopover?: () => void
  }
const cards = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll<HTMLButtonElement>('.card'))

describe('<sui-theme-picker>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-theme-picker')).toBeDefined()
  })

  it('renders trigger + panel in shadow DOM', () => {
    const { el } = mount(reg, 'sui-theme-picker')
    expect(trigger(el)).not.toBeNull()
    expect(panel(el)).not.toBeNull()
  })

  it('renders one card per theme via {#for}', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    expect(cards(el).length).toBeGreaterThanOrEqual(7)
  })

  it('cards live inside the panel grid with role=listbox', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    const grid = panel(el).querySelector('.grid')
    expect(grid).not.toBeNull()
    expect(grid!.getAttribute('role')).toBe('listbox')
    expect(grid!.querySelectorAll('.card').length).toBeGreaterThan(0)
  })

  it('trigger has aria-expanded=false when closed', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    expect(trigger(el).getAttribute('aria-expanded')).toBe('false')
  })

  it('clicking the trigger calls showPopover', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    const p = panel(el)
    let shows = 0
    const orig = p.showPopover
    p.showPopover = function () { shows++; orig?.call(p) }
    trigger(el).click()
    await flush()
    expect(shows).toBe(1)
  })

  it('clicking a card applies the theme and updates aria-selected', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    const all = cards(el)
    const dark = all.find((c) => c.dataset.themeCard === 'ship-dark')!
    dark.click()
    await flush()
    expect(dark.getAttribute('aria-selected')).toBe('true')
    expect(document.documentElement.dataset.theme).toBe('ship-dark')
  })

  it('clicking a card dispatches sui-theme-change with id + dark', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    let detail: { id?: string; dark?: boolean } | null = null
    el.addEventListener('sui-theme-change', (e) => {
      detail = (e as CustomEvent).detail
    })
    const ck = cards(el).find((c) => c.dataset.themeCard === 'console')!
    ck.click()
    await flush()
    expect(detail).not.toBeNull()
    expect(detail!.id).toBe('console')
    expect(typeof detail!.dark).toBe('boolean')
  })

  it('clicking a card calls hidePopover to dismiss the panel', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    const p = panel(el)
    let hides = 0
    const orig = p.hidePopover
    p.hidePopover = function () { hides++; orig?.call(p) }
    cards(el)[0].click()
    await flush()
    expect(hides).toBe(1)
  })

  it('external ship:theme:change syncs the active card', async () => {
    const { el } = mount(reg, 'sui-theme-picker')
    await flush()
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', { detail: { id: 'paper', dark: false } }),
    )
    await flush()
    const paperCard = cards(el).find((c) => c.dataset.themeCard === 'paper')!
    expect(paperCard.getAttribute('aria-selected')).toBe('true')
  })

  it('placement prop reflects on host', () => {
    const { el } = mount(reg, 'sui-theme-picker', { placement: 'bottom-end' })
    expect(el.getAttribute('placement')).toBe('bottom-end')
  })

  it('label-style=compact reflects on host', () => {
    const { el } = mount(reg, 'sui-theme-picker', { 'label-style': 'compact' })
    expect(el.getAttribute('label-style')).toBe('compact')
  })

  it('disconnect — no error', () => {
    const { container } = mount(reg, 'sui-theme-picker')
    container.remove()
  })
})
