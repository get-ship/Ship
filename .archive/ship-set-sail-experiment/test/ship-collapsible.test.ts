import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-collapsible (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-collapsible.set')
  }, 120_000)

  function mount(attrs = '') {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-collapsible ${attrs}>
        <button slot="trigger">Toggle</button>
        <div>Content here</div>
      </ship-collapsible>
    `
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }

  it('renders content-wrap region with role=region', () => {
    const el = mount()
    const wrap = el.querySelector('.content-wrap') as HTMLElement
    expect(wrap).not.toBeNull()
    expect(wrap.getAttribute('role')).toBe('region')
    expect(wrap.id).toMatch(/^ship-collapsible-/)
    el.remove()
  })

  it('renders closed by default (no [open] attribute)', () => {
    const el = mount()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('open attribute reflects on the host', () => {
    const el = mount('open')
    expect(el.hasAttribute('open')).toBe(true)
    el.remove()
  })

  it('toggles open + emits ship-toggle on trigger click', () => {
    const el = mount()
    const events: any[] = []
    el.addEventListener('ship-toggle', (e: any) => events.push(e.detail))
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    trigger.click()
    expect(el.hasAttribute('open')).toBe(true)
    expect(events).toEqual([{ open: true }])
    trigger.click()
    expect(el.hasAttribute('open')).toBe(false)
    expect(events).toEqual([{ open: true }, { open: false }])
    el.remove()
  })

  it('sets aria-expanded + aria-controls on the trigger', () => {
    const el = mount()
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    expect(trigger.getAttribute('aria-expanded')).toBe('false')
    expect(trigger.getAttribute('aria-controls')).toMatch(/^ship-collapsible-/)
    el.remove()
  })

  it('updates aria-expanded after toggle', () => {
    const el = mount()
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    trigger.click()
    expect(trigger.getAttribute('aria-expanded')).toBe('true')
    el.remove()
  })

  it('does not toggle when [disabled] is set', () => {
    const el = mount('disabled')
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    trigger.click()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('Enter and Space on trigger toggle (synthesized via keydown)', () => {
    const el = mount()
    const trigger = el.querySelector('.trigger-wrap > button') as HTMLElement
    trigger.focus()
    // The wrapper div listens for keydown — dispatch from the trigger,
    // letting the event bubble up to the wrapper.
    trigger.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    expect(el.hasAttribute('open')).toBe(true)
    trigger.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true, cancelable: true }))
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })
})
