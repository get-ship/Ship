import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-switch (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-switch.set')
  }, 120_000)

  it('renders track + thumb + native checkbox with role=switch', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('.track')).not.toBeNull()
    expect(el.querySelector('.thumb')).not.toBeNull()
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.type).toBe('checkbox')
    expect(input.getAttribute('role')).toBe('switch')
    el.remove()
  })

  it('checked attribute reflects to native input + aria-checked', () => {
    const el = document.createElement(tag)
    el.setAttribute('checked', '')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.checked).toBe(true)
    expect(input.getAttribute('aria-checked')).toBe('true')
    el.remove()
  })

  it('toggling fires ship-change with the new checked state', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))

    const input = el.querySelector('input') as HTMLInputElement
    input.checked = true
    input.dispatchEvent(new Event('change', { bubbles: true }))
    expect(events).toEqual([{ checked: true }])
    expect(el.hasAttribute('checked')).toBe(true)

    input.checked = false
    input.dispatchEvent(new Event('change', { bubbles: true }))
    expect(events).toEqual([{ checked: true }, { checked: false }])
    expect(el.hasAttribute('checked')).toBe(false)

    el.remove()
  })

  it('disabled prop disables the native input', () => {
    const el = document.createElement(tag)
    el.setAttribute('disabled', '')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.disabled).toBe(true)
    el.remove()
  })

  it('aria-label falls back through label → name → "toggle"', () => {
    const e1 = document.createElement(tag)
    e1.setAttribute('label', 'Notifications')
    document.body.appendChild(e1)
    expect(e1.querySelector('input')!.getAttribute('aria-label')).toBe('Notifications')
    e1.remove()

    const e2 = document.createElement(tag)
    e2.setAttribute('name', 'notify')
    document.body.appendChild(e2)
    expect(e2.querySelector('input')!.getAttribute('aria-label')).toBe('notify')
    e2.remove()

    const e3 = document.createElement(tag)
    document.body.appendChild(e3)
    expect(e3.querySelector('input')!.getAttribute('aria-label')).toBe('toggle')
    e3.remove()
  })

  it('component declares static formAssociated = true', () => {
    const ctor = customElements.get(tag) as any
    expect(ctor.formAssociated).toBe(true)
  })

  it('default slot fills with label content', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('Receive emails'))
    document.body.appendChild(el)
    expect(el.querySelector('label')!.textContent).toContain('Receive emails')
    el.remove()
  })

  it('tone × size matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const size of sizes) {
        const el = document.createElement(tag)
        el.setAttribute('tone', tone)
        el.setAttribute('size', size)
        document.body.appendChild(el)
        expect(el.querySelector('.track')).not.toBeNull()
        el.remove()
        n++
      }
    }
    expect(n).toBe(7 * 3)
  })
})
