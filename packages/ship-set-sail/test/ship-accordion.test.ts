import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-accordion (sail)', () => {
  beforeAll(async () => {
    // Children before parent so the accordion sees them at mount.
    await loadSailComponent('ship-accordion-item.set')
    await loadSailComponent('ship-accordion.set')
  }, 120_000)

  function mount(attrs = ''): HTMLElement {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-accordion ${attrs}>
        <ship-accordion-item value="a">
          <span slot="trigger">Item A</span>
          <p>Content A</p>
        </ship-accordion-item>
        <ship-accordion-item value="b">
          <span slot="trigger">Item B</span>
          <p>Content B</p>
        </ship-accordion-item>
        <ship-accordion-item value="c" disabled>
          <span slot="trigger">Item C</span>
          <p>Content C</p>
        </ship-accordion-item>
      </ship-accordion>
    `
    document.body.appendChild(wrap)
    return wrap.querySelector('ship-accordion') as HTMLElement
  }

  it('renders all items closed by default', () => {
    const el = mount()
    const items = el.querySelectorAll('ship-accordion-item')
    expect(items.length).toBe(3)
    for (const item of items) expect(item.hasAttribute('open')).toBe(false)
    el.parentElement!.remove()
  })

  it('opens item via value attribute on mount', () => {
    const el = mount('value="a"')
    const items = el.querySelectorAll('ship-accordion-item')
    expect(items[0]!.hasAttribute('open')).toBe(true)
    expect(items[1]!.hasAttribute('open')).toBe(false)
    expect(items[2]!.hasAttribute('open')).toBe(false)
    el.parentElement!.remove()
  })

  it('single mode: clicking opens one item, closes previous', () => {
    const el = mount()
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))

    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const triggerA = itemA.querySelector('button.trigger') as HTMLButtonElement
    triggerA.click()

    expect(itemA.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('value')).toBe('a')
    expect(events).toEqual([{ value: 'a' }])

    const itemB = el.querySelector('ship-accordion-item[value="b"]') as HTMLElement
    const triggerB = itemB.querySelector('button.trigger') as HTMLButtonElement
    triggerB.click()

    expect(itemA.hasAttribute('open')).toBe(false)
    expect(itemB.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('value')).toBe('b')
    expect(events).toEqual([{ value: 'a' }, { value: 'b' }])

    el.parentElement!.remove()
  })

  it('single mode: clicking the same item again closes it', () => {
    const el = mount()
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const triggerA = itemA.querySelector('button.trigger') as HTMLButtonElement
    triggerA.click()
    expect(itemA.hasAttribute('open')).toBe(true)
    triggerA.click()
    expect(itemA.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('value')).toBeNull()
    el.parentElement!.remove()
  })

  it('multiple mode: allows several items open', () => {
    const el = mount('type="multiple"')
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const itemB = el.querySelector('ship-accordion-item[value="b"]') as HTMLElement
    ;(itemA.querySelector('button.trigger') as HTMLButtonElement).click()
    ;(itemB.querySelector('button.trigger') as HTMLButtonElement).click()
    expect(itemA.hasAttribute('open')).toBe(true)
    expect(itemB.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('value')!.split(',').sort()).toEqual(['a', 'b'])
    el.parentElement!.remove()
  })

  it('multiple mode: clicking an open item closes only that item', () => {
    const el = mount('type="multiple"')
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const itemB = el.querySelector('ship-accordion-item[value="b"]') as HTMLElement
    ;(itemA.querySelector('button.trigger') as HTMLButtonElement).click()
    ;(itemB.querySelector('button.trigger') as HTMLButtonElement).click()
    ;(itemA.querySelector('button.trigger') as HTMLButtonElement).click()
    expect(itemA.hasAttribute('open')).toBe(false)
    expect(itemB.hasAttribute('open')).toBe(true)
    el.parentElement!.remove()
  })

  it('disabled item does not toggle on click', () => {
    const el = mount()
    const itemC = el.querySelector('ship-accordion-item[value="c"]') as HTMLElement
    const triggerC = itemC.querySelector('button.trigger') as HTMLButtonElement
    // The native button is also disabled, so click() is a no-op even
    // if the handler runs.
    expect(triggerC.disabled).toBe(true)
    triggerC.click()
    expect(itemC.hasAttribute('open')).toBe(false)
    expect(el.hasAttribute('value')).toBe(false)
    el.parentElement!.remove()
  })

  it('item exposes correct ARIA wiring on its trigger and panel', () => {
    const el = mount('value="a"')
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const trigger = itemA.querySelector('button.trigger') as HTMLButtonElement
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    expect(trigger.getAttribute('aria-controls')).toMatch(/^ship-acc-content-/)
    expect(trigger.id).toMatch(/^ship-acc-trigger-/)

    const region = itemA.querySelector('[role="region"]') as HTMLElement
    expect(region).not.toBeNull()
    expect(region.getAttribute('aria-labelledby')).toBe(trigger.id)
    expect(region.id).toBe(trigger.getAttribute('aria-controls'))

    const closed = el.querySelector('ship-accordion-item[value="b"]')!
      .querySelector('button.trigger') as HTMLButtonElement
    expect(closed.getAttribute('aria-expanded')).toBe('false')

    el.parentElement!.remove()
  })

  it('aria-expanded updates when the item opens', () => {
    const el = mount()
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const trigger = itemA.querySelector('button.trigger') as HTMLButtonElement
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    trigger.click()
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    el.parentElement!.remove()
  })

  it('ArrowDown / ArrowUp move focus across enabled triggers, skipping disabled', () => {
    const el = mount()
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const itemB = el.querySelector('ship-accordion-item[value="b"]') as HTMLElement
    const triggerA = itemA.querySelector('button.trigger') as HTMLButtonElement
    const triggerB = itemB.querySelector('button.trigger') as HTMLButtonElement

    triggerA.focus()
    triggerA.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowDown', bubbles: true, cancelable: true,
    }))
    expect(document.activeElement).toBe(triggerB)

    // Next from B would be C (disabled) — wraps to A.
    triggerB.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowDown', bubbles: true, cancelable: true,
    }))
    expect(document.activeElement).toBe(triggerA)

    // ArrowUp from A wraps to last enabled (B).
    triggerA.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowUp', bubbles: true, cancelable: true,
    }))
    expect(document.activeElement).toBe(triggerB)

    el.parentElement!.remove()
  })

  it('Home / End jump to first / last enabled trigger', () => {
    const el = mount()
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const itemB = el.querySelector('ship-accordion-item[value="b"]') as HTMLElement
    const triggerA = itemA.querySelector('button.trigger') as HTMLButtonElement
    const triggerB = itemB.querySelector('button.trigger') as HTMLButtonElement

    triggerB.focus()
    triggerB.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Home', bubbles: true, cancelable: true,
    }))
    expect(document.activeElement).toBe(triggerA)

    triggerA.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'End', bubbles: true, cancelable: true,
    }))
    // End targets the last *enabled* trigger; C is disabled, so B.
    expect(document.activeElement).toBe(triggerB)

    el.parentElement!.remove()
  })

  it('chevron + content-wrap parts are present', () => {
    const el = mount()
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    expect(itemA.querySelector('[part="trigger"]')).not.toBeNull()
    expect(itemA.querySelector('[part="content"]')).not.toBeNull()
    expect(itemA.querySelector('[part="chevron"]')).not.toBeNull()
    el.parentElement!.remove()
  })

  it('value attribute change reflects to children imperatively', () => {
    const el = mount()
    el.setAttribute('value', 'b')
    const itemA = el.querySelector('ship-accordion-item[value="a"]') as HTMLElement
    const itemB = el.querySelector('ship-accordion-item[value="b"]') as HTMLElement
    expect(itemA.hasAttribute('open')).toBe(false)
    expect(itemB.hasAttribute('open')).toBe(true)
    el.parentElement!.remove()
  })
})
