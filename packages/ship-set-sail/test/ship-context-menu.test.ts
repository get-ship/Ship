import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-context-menu (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-context-menu.set')
  }, 120_000)

  function mount(markup: string) {
    const wrap = document.createElement('div')
    wrap.innerHTML = markup
    const el = wrap.firstElementChild as any
    document.body.appendChild(el)
    return el
  }

  it('renders the menu container with role=menu and aria-label', () => {
    const el = mount(`
      <ship-context-menu>
        <span slot="trigger">Right-click me</span>
        <button data-value="cut">Cut</button>
      </ship-context-menu>
    `)
    const menu = el.querySelector('.menu')!
    expect(menu).not.toBeNull()
    expect(menu.getAttribute('role')).toBe('menu')
    expect(menu.getAttribute('aria-label')).toBe('Context menu')
    el.remove()
  })

  it('starts closed and reflects data-state', () => {
    const el = mount(`
      <ship-context-menu>
        <span slot="trigger">t</span>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
    el.remove()
  })

  it('show(x, y) opens, sets data-state, and positions the menu', () => {
    const el = mount(`
      <ship-context-menu>
        <span slot="trigger">t</span>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    el.show(123, 456)
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
    const menu = el.querySelector('.menu') as HTMLElement
    expect(menu.style.left).toBe('123px')
    expect(menu.style.top).toBe('456px')
    el.remove()
  })

  it('contextmenu on the trigger wrap opens the menu and prevents default', () => {
    const el = mount(`
      <ship-context-menu>
        <span slot="trigger">t</span>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    const wrap = el.querySelector('.trigger-wrap') as HTMLElement
    const ev = new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 50, clientY: 75 })
    wrap.dispatchEvent(ev)
    expect(ev.defaultPrevented).toBe(true)
    expect(el.hasAttribute('open')).toBe(true)
    const menu = el.querySelector('.menu') as HTMLElement
    expect(menu.style.left).toBe('50px')
    expect(menu.style.top).toBe('75px')
    el.remove()
  })

  it('emits ship-open / ship-close on show / close', () => {
    const el = mount(`
      <ship-context-menu>
        <span slot="trigger">t</span>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    let opened = 0
    let closed = 0
    el.addEventListener('ship-open', () => opened++)
    el.addEventListener('ship-close', () => closed++)
    el.show(0, 0)
    el.close()
    expect(opened).toBe(1)
    expect(closed).toBe(1)
    el.remove()
  })

  it('clicking an item dispatches ship-select with its data-value and closes', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="cut">Cut</button>
        <button data-value="copy">Copy</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    let detail: { value: string } | null = null
    el.addEventListener('ship-select', (e: any) => { detail = e.detail })
    ;(el.querySelectorAll('button')[1] as HTMLButtonElement).click()
    expect(detail).toEqual({ value: 'copy' })
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('Escape closes the open menu', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="x">X</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('Tab closes the open menu (focus releases to the page)', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="x">X</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('keydown on a closed menu is a no-op', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="x">X</button>
      </ship-context-menu>
    `)
    expect(() =>
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })),
    ).not.toThrow()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('assigns role="menuitem" to items at mount', async () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
      </ship-context-menu>
    `)
    // setupItems is queued in a microtask at mount.
    await Promise.resolve()
    const items = el.querySelectorAll('button')
    expect(items[0].getAttribute('role')).toBe('menuitem')
    expect(items[1].getAttribute('role')).toBe('menuitem')
    el.remove()
  })

  it('ArrowDown / ArrowUp move focus through items with wrap', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">Apple</button>
        <button data-value="b">Banana</button>
        <button data-value="c">Cherry</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const items = el.querySelectorAll('button') as NodeListOf<HTMLButtonElement>
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(items[1])
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(items[2])
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(items[0]) // wrap
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    expect(document.activeElement).toBe(items[2]) // wrap up
    el.remove()
  })

  it('Home / End jump to first / last item', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
        <button data-value="c">C</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const items = el.querySelectorAll('button') as NodeListOf<HTMLButtonElement>
    items[1].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    expect(document.activeElement).toBe(items[2])
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }))
    expect(document.activeElement).toBe(items[0])
    el.remove()
  })

  it('Enter on the focused item activates it (fires ship-select, closes)', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const items = el.querySelectorAll('button') as NodeListOf<HTMLButtonElement>
    items[1].focus()
    let detail: any = null
    el.addEventListener('ship-select', (e: any) => { detail = e.detail })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(detail).toEqual({ value: 'b' })
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('Space activates the focused item the same as Enter', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="x">X</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const item = el.querySelector('button') as HTMLButtonElement
    item.focus()
    let detail: any = null
    el.addEventListener('ship-select', (e: any) => { detail = e.detail })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(detail).toEqual({ value: 'x' })
    el.remove()
  })

  it('typeahead jumps to the next item starting with the typed letter', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">Apple</button>
        <button data-value="b">Banana</button>
        <button data-value="c">Cherry</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const items = el.querySelectorAll('button') as NodeListOf<HTMLButtonElement>
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'b', bubbles: true }))
    expect(document.activeElement).toBe(items[1])
    el.remove()
  })

  it('typeahead cycles through matches when the same letter is repeated', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a1">Apple</button>
        <button data-value="a2">Apricot</button>
        <button data-value="b">Banana</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const items = el.querySelectorAll('button') as NodeListOf<HTMLButtonElement>
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', bubbles: true }))
    expect(document.activeElement).toBe(items[1]) // Apricot
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', bubbles: true }))
    expect(document.activeElement).toBe(items[0]) // wraps to Apple
    el.remove()
  })

  it('disabled items are skipped', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">A</button>
        <button data-value="b" disabled>B</button>
        <button data-value="c">C</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const items = el.querySelectorAll('button') as NodeListOf<HTMLButtonElement>
    items[0].focus()
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(items[2])
    el.remove()
  })

  it('clicking outside the host closes an open menu', () => {
    const outside = document.createElement('div')
    outside.textContent = 'outside'
    document.body.appendChild(outside)
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    expect(el.hasAttribute('open')).toBe(true)
    outside.click()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
    outside.remove()
  })

  it('clicking inside the host does not close the menu (unless on a value)', () => {
    const el = mount(`
      <ship-context-menu>
        <span slot="trigger">t</span>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    const trigger = el.querySelector('.trigger-wrap') as HTMLElement
    trigger.click()
    expect(el.hasAttribute('open')).toBe(true)
    el.remove()
  })

  it('cleanup removes document-level click listener on disconnect', () => {
    const el = mount(`
      <ship-context-menu>
        <button data-value="a">A</button>
      </ship-context-menu>
    `)
    el.show(0, 0)
    el.remove()
    expect(() => document.body.click()).not.toThrow()
  })
})
