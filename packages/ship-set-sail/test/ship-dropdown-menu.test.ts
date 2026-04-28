import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-dropdown-menu (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-dropdown-menu.set')
  }, 120_000)

  function mount(markup: string) {
    const wrap = document.createElement('div')
    wrap.innerHTML = markup
    const el = wrap.firstElementChild as any
    document.body.appendChild(el)
    return el
  }

  it('renders the menu with role=menu and a trigger wrap', () => {
    const el = mount(
      '<ship-dropdown-menu><button slot="trigger">More</button></ship-dropdown-menu>',
    )
    const menu = el.querySelector('.menu')!
    const triggerWrap = el.querySelector('.trigger-wrap')!
    expect(menu).not.toBeNull()
    expect(menu.getAttribute('role')).toBe('menu')
    expect(triggerWrap).not.toBeNull()
    el.remove()
  })

  it('starts closed and reflects data-state', () => {
    const el = mount(
      '<ship-dropdown-menu><button slot="trigger">More</button></ship-dropdown-menu>',
    )
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
    el.remove()
  })

  it('clicking the trigger toggles open and data-state', () => {
    const el = mount(`
      <ship-dropdown-menu>
        <button slot="trigger">More</button>
        <button data-value="rename">Rename</button>
      </ship-dropdown-menu>
    `)
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    trigger.click()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
    trigger.click()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
    el.remove()
  })

  it('wires aria-haspopup and aria-expanded on the trigger', () => {
    const el = mount(
      '<ship-dropdown-menu><button slot="trigger">More</button></ship-dropdown-menu>',
    )
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    expect(trigger.getAttribute('aria-haspopup')).toBe('menu')
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    trigger.click()
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    trigger.click()
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    el.remove()
  })

  it('fires ship-select with data-value on item click and closes', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="rename">Rename</button>
        <button data-value="delete">Delete</button>
      </ship-dropdown-menu>
    `)
    let detail: { value: string } | null = null
    el.addEventListener('ship-select', (e: any) => {
      detail = e.detail
    })
    const items = Array.from(
      el.querySelectorAll('button[data-value]'),
    ) as HTMLButtonElement[]
    items[1].click()
    expect(detail).toEqual({ value: 'delete' })
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('clicking an inner element bubbles up to the [data-value] ancestor', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="copy"><span class="inner">Copy</span></button>
      </ship-dropdown-menu>
    `)
    let detail: { value: string } | null = null
    el.addEventListener('ship-select', (e: any) => {
      detail = e.detail
    })
    ;(el.querySelector('.inner') as HTMLElement).click()
    expect(detail).toEqual({ value: 'copy' })
    el.remove()
  })

  it('disabled items are skipped by item discovery', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
        <button data-value="b" disabled>B</button>
        <button data-value="c">C</button>
      </ship-dropdown-menu>
    `)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('a')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('c')
    el.remove()
  })

  it('ArrowDown / ArrowUp navigate items with wrap', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
        <button data-value="c">C</button>
      </ship-dropdown-menu>
    `)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('a')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('b')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('a')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('c')
    el.remove()
  })

  it('Home / End jump to the first / last item', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
        <button data-value="c">C</button>
      </ship-dropdown-menu>
    `)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('c')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('a')
    el.remove()
  })

  it('Escape closes the menu and returns focus to the trigger', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
      </ship-dropdown-menu>
    `)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(el.hasAttribute('open')).toBe(false)
    expect((document.activeElement as HTMLElement).textContent).toBe('More')
    el.remove()
  })

  it('Tab closes the menu without preventing default focus traversal', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
      </ship-dropdown-menu>
    `)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('Enter / Space activate the focused item', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
      </ship-dropdown-menu>
    `)
    let detail: { value: string } | null = null
    el.addEventListener('ship-select', (e: any) => {
      detail = e.detail
    })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(detail).toEqual({ value: 'b' })
    el.remove()
  })

  it('typeahead jumps to the first item starting with the typed letter', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="apple">Apple</button>
        <button data-value="banana">Banana</button>
        <button data-value="cherry">Cherry</button>
      </ship-dropdown-menu>
    `)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'b', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('banana')
    el.remove()
  })

  it('typeahead cycles through items sharing the typed letter', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="ant">Ant</button>
        <button data-value="ape">Ape</button>
        <button data-value="bear">Bear</button>
      </ship-dropdown-menu>
    `)
    // Repeated single-char keys cycle (Listbox APG pattern). The
    // buffer stays length=1 because typeaheadPush short-circuits when
    // the same char repeats, so each press advances by one match.
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('ant')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', bubbles: true }))
    expect((document.activeElement as HTMLElement).getAttribute('data-value')).toBe('ape')
    el.remove()
  })

  it('items get role=menuitem and the first item gets tabindex=0', () => {
    const el = mount(`
      <ship-dropdown-menu>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
        <button data-value="b">B</button>
      </ship-dropdown-menu>
    `)
    const items = Array.from(
      el.querySelectorAll('button[data-value]'),
    ) as HTMLElement[]
    for (const it of items) expect(it.getAttribute('role')).toBe('menuitem')
    expect(items[0].getAttribute('tabindex')).toBe('0')
    expect(items[1].getAttribute('tabindex')).toBe('-1')
    el.remove()
  })

  it('outside click closes an open menu', () => {
    const outside = document.createElement('div')
    outside.textContent = 'outside'
    document.body.appendChild(outside)
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
      </ship-dropdown-menu>
    `)
    expect(el.hasAttribute('open')).toBe(true)
    outside.click()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
    outside.remove()
  })

  it('clicking inside the menu on a non-item area does not emit ship-select', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
      </ship-dropdown-menu>
    `)
    let fired = 0
    el.addEventListener('ship-select', () => {
      fired++
    })
    const menu = el.querySelector('.menu') as HTMLElement
    menu.click()
    expect(fired).toBe(0)
    expect(el.hasAttribute('open')).toBe(true)
    el.remove()
  })

  it('cleanup removes the document-level click listener on disconnect', () => {
    const el = mount(`
      <ship-dropdown-menu open>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
      </ship-dropdown-menu>
    `)
    el.remove()
    expect(() => document.body.click()).not.toThrow()
  })

  it('setting the open attribute externally syncs ARIA + data-state', () => {
    const el = mount(`
      <ship-dropdown-menu>
        <button slot="trigger">More</button>
        <button data-value="a">A</button>
      </ship-dropdown-menu>
    `)
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    el.setAttribute('open', '')
    expect(el.getAttribute('data-state')).toBe('open')
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    el.removeAttribute('open')
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    el.remove()
  })
})
