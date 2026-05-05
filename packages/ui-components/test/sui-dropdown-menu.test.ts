import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-dropdown-menu/dist/sui-dropdown-menu.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const panelEl = (el: HTMLElement) => shadow(el).querySelector('.panel') as HTMLElement | null
const triggerWrap = (el: HTMLElement) => shadow(el).querySelector('.trigger-wrap') as HTMLElement | null

function makeMenu(attrs: Record<string, string | boolean> = {}, items = '') {
  return mount(
    reg,
    'sui-dropdown-menu',
    attrs,
    `<button slot="trigger">Menu</button>${items}`,
  )
}

function makeMenuWithItems(attrs: Record<string, string | boolean> = {}) {
  return makeMenu(attrs, `
    <button data-value="rename">Rename</button>
    <button data-value="delete">Delete</button>
    <button data-value="copy">Copy</button>
  `)
}

describe('<sui-dropdown-menu>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-dropdown-menu')).toBeDefined()
  })

  it('closed by default', () => {
    const { el } = makeMenu()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('renders a trigger slot', () => {
    const { el } = makeMenu()
    const slot = shadow(el).querySelector('slot[name="trigger"]')
    expect(slot).not.toBeNull()
  })

  it('renders a panel element with role=menu', () => {
    const { el } = makeMenu()
    const panel = panelEl(el)
    expect(panel).not.toBeNull()
    expect(panel!.getAttribute('role')).toBe('menu')
  })

  it('trigger has aria-haspopup=menu and aria-controls after flush', async () => {
    const { el } = makeMenu()
    await flush()
    const trigger = el.querySelector('button[slot="trigger"]') as HTMLElement
    expect(trigger.getAttribute('aria-haspopup')).toBe('menu')
    expect(trigger.getAttribute('aria-controls')).toBeTruthy()
  })

  it('trigger aria-expanded=false when closed', async () => {
    const { el } = makeMenu()
    await flush()
    const trigger = el.querySelector('button[slot="trigger"]') as HTMLElement
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
  })

  it('items get role=menuitem after setup', async () => {
    const { el } = makeMenuWithItems()
    await flush()
    const items = el.querySelectorAll('button[data-value]')
    expect(items[0].getAttribute('role')).toBe('menuitem')
    expect(items[1].getAttribute('role')).toBe('menuitem')
  })

  it('items get roving tabindex — first=0, rest=-1', async () => {
    const { el } = makeMenuWithItems()
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    expect(items[0].getAttribute('tabindex')).toBe('0')
    expect(items[1].getAttribute('tabindex')).toBe('-1')
    expect(items[2].getAttribute('tabindex')).toBe('-1')
  })

  it('trigger click opens panel (showPopover called)', async () => {
    const { el } = makeMenu()
    const tw = triggerWrap(el)!
    let popovers = 0
    const p = panelEl(el)!
    const orig = p.showPopover
    p.showPopover = () => { popovers++; orig?.call(p) }
    tw.click()
    await flush()
    expect(popovers).toBe(1)
  })

  it('fires sui-select with data-value on item click', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    let detail: { value: string } | null = null
    el.addEventListener('sui-select', (e) => { detail = (e as CustomEvent).detail })
    const deleteBtn = el.querySelector<HTMLButtonElement>('button[data-value="delete"]')!
    deleteBtn.click()
    await flush()
    expect(detail).toEqual({ value: 'delete' })
  })

  it('panel closes after item click', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const p = panelEl(el)!
    let hideCount = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hideCount++; orig?.call(p) }
    el.querySelector<HTMLButtonElement>('button[data-value="rename"]')!.click()
    await flush()
    expect(hideCount).toBe(1)
  })

  it('ArrowDown key moves focus to next item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(items[1])
  })

  it('ArrowUp key wraps to last item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    expect(document.activeElement).toBe(items[items.length - 1])
  })

  it('Home key focuses first item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[2].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }))
    expect(document.activeElement).toBe(items[0])
  })

  it('End key focuses last item', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    expect(document.activeElement).toBe(items[items.length - 1])
  })

  it('Escape key hides panel', async () => {
    const { el } = makeMenuWithItems({ open: true })
    await flush()
    const p = panelEl(el)!
    let hideCount = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hideCount++; orig?.call(p) }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(hideCount).toBe(1)
  })

  it('disconnect — no error', () => {
    const { container } = makeMenuWithItems()
    container.remove()
  })
})
