import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-context-menu/sui-context-menu.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const panelEl = (el: HTMLElement) => shadow(el).querySelector('.panel') as HTMLElement | null

function makeMenu(attrs: Record<string, string | boolean> = {}, items = '') {
  return mount(
    reg,
    'sui-context-menu',
    attrs,
    `<span slot="trigger">Right-click here</span>${items}`,
  )
}

function makeMenuWithItems(attrs: Record<string, string | boolean> = {}) {
  return makeMenu(attrs, `
    <button data-value="cut">Cut</button>
    <button data-value="copy">Copy</button>
    <button data-value="paste">Paste</button>
  `)
}

type SuiContextMenu = HTMLElement & { show(x: number, y: number): void; close(): void }

describe('<sui-context-menu>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-context-menu')).toBeDefined()
  })

  it('closed by default', () => {
    const { el } = makeMenu()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('renders a panel with role=menu', () => {
    const { el } = makeMenu()
    const panel = panelEl(el)
    expect(panel).not.toBeNull()
    expect(panel!.getAttribute('role')).toBe('menu')
  })

  it('has trigger and default slots', () => {
    const { el } = makeMenu()
    const slots = Array.from(shadow(el).querySelectorAll('slot'))
    const names = slots.map(s => s.getAttribute('name'))
    expect(names).toContain('trigger')
    expect(names.some(n => n === null)).toBe(true)
  })

  it('show(x,y) calls showPopover and positions panel', async () => {
    const { el } = makeMenuWithItems()
    const p = panelEl(el)!
    let showed = 0
    const orig = p.showPopover
    p.showPopover = () => { showed++; orig?.call(p) }
    ;(el as SuiContextMenu).show(150, 250)
    await flush()
    expect(showed).toBe(1)
    expect(p.style.left).toBe('150px')
    expect(p.style.top).toBe('250px')
  })

  it('close() calls hidePopover', async () => {
    const { el } = makeMenuWithItems()
    const p = panelEl(el)!
    let hid = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hid++; orig?.call(p) }
    ;(el as SuiContextMenu).close()
    await flush()
    expect(hid).toBe(1)
  })

  it('items get role=menuitem', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const items = el.querySelectorAll('button[data-value]')
    for (const item of items) {
      expect(item.getAttribute('role')).toBe('menuitem')
    }
  })

  it('item click fires sui-select with value', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    let detail: { value: string } | null = null
    el.addEventListener('sui-select', (e) => { detail = (e as CustomEvent).detail })
    el.querySelector<HTMLButtonElement>('button[data-value="copy"]')!.click()
    await flush()
    expect(detail).toEqual({ value: 'copy' })
  })

  it('panel closes after item click', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const p = panelEl(el)!
    let hid = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hid++; orig?.call(p) }
    el.querySelector<HTMLButtonElement>('button[data-value="cut"]')!.click()
    await flush()
    expect(hid).toBe(1)
  })

  it('Escape key closes the menu', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const p = panelEl(el)!
    let hid = 0
    const orig = p.hidePopover
    p.hidePopover = () => { hid++; orig?.call(p) }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(hid).toBe(1)
  })

  it('ArrowDown moves focus to next item', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(items[1])
  })

  it('ArrowUp wraps to last item', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    expect(document.activeElement).toBe(items[items.length - 1])
  })

  it('Home/End key navigation', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[1].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }))
    expect(document.activeElement).toBe(items[0])
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    expect(document.activeElement).toBe(items[items.length - 1])
  })

  it('Enter activates focused item', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    items[0].setAttribute('tabindex', '0')
    items[0].focus()
    let detail: { value: string } | null = null
    el.addEventListener('sui-select', (e) => { detail = (e as CustomEvent).detail })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await flush()
    expect(detail?.value).toBe('cut')
  })

  it('items have roving tabindex after open', async () => {
    const { el } = makeMenuWithItems()
    ;(el as SuiContextMenu).show(0, 0)
    await flush()
    const items = Array.from(el.querySelectorAll<HTMLElement>('button[data-value]'))
    expect(items[0].getAttribute('tabindex')).toBe('0')
    expect(items[1].getAttribute('tabindex')).toBe('-1')
  })

  it('disconnect — no error', () => {
    const { container } = makeMenuWithItems()
    container.remove()
  })
})
