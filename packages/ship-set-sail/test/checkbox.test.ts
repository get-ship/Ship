import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-checkbox (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-checkbox.set')
  }, 120_000)

  it('renders a native checkbox input wrapped by a label', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const input = el.querySelector('input[type="checkbox"]') as HTMLInputElement
    const label = el.querySelector('label')
    expect(input).not.toBeNull()
    expect(label).not.toBeNull()
    el.remove()
  })

  it('checked attribute reflects to the native input', () => {
    const el = document.createElement(tag)
    el.setAttribute('checked', '')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.checked).toBe(true)
    expect(input.getAttribute('aria-checked')).toBe('true')
    el.remove()
  })

  it('disabled state is exposed via attributes', () => {
    const el = document.createElement(tag)
    el.setAttribute('disabled', '')
    document.body.appendChild(el)
    expect(el.hasAttribute('disabled')).toBe(true)
    el.remove()
  })

  it('clicking the input dispatches ship-change with the new state', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))

    const input = el.querySelector('input') as HTMLInputElement
    // Simulate user toggle: set checked on the native input + dispatch change.
    input.checked = true
    input.dispatchEvent(new Event('change', { bubbles: true }))
    expect(events).toEqual([{ checked: true }])
    expect(el.hasAttribute('checked')).toBe(true)
    el.remove()
  })

  it('indeterminate prop syncs to the native input.indeterminate', async () => {
    const el = document.createElement(tag)
    el.setAttribute('indeterminate', '')
    document.body.appendChild(el)
    // The mount block grabs the native input on the next microtask.
    await new Promise((r) => queueMicrotask(() => r(undefined)))
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.indeterminate).toBe(true)
    expect(input.getAttribute('aria-checked')).toBe('mixed')
    el.remove()
  })

  it('invalid prop sets aria-invalid', () => {
    const el = document.createElement(tag)
    el.setAttribute('invalid', '')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.getAttribute('aria-invalid')).toBe('true')
    el.remove()
  })

  it('tone × size matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    for (const tone of tones) {
      for (const size of sizes) {
        const el = document.createElement(tag)
        el.setAttribute('tone', tone)
        el.setAttribute('size', size)
        document.body.appendChild(el)
        expect(el.querySelector('.box')).not.toBeNull()
        el.remove()
      }
    }
  })

  it('default slot fills with the label text', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('Accept terms'))
    document.body.appendChild(el)
    expect(el.querySelector('label')!.textContent).toContain('Accept terms')
    el.remove()
  })

  it('aria-label falls back to the label prop, then name, then "checkbox"', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Newsletter')
    document.body.appendChild(el)
    expect(el.querySelector('input')!.getAttribute('aria-label')).toBe('Newsletter')
    el.remove()

    const el2 = document.createElement(tag)
    el2.setAttribute('name', 'tos')
    document.body.appendChild(el2)
    expect(el2.querySelector('input')!.getAttribute('aria-label')).toBe('tos')
    el2.remove()

    const el3 = document.createElement(tag)
    document.body.appendChild(el3)
    expect(el3.querySelector('input')!.getAttribute('aria-label')).toBe('checkbox')
    el3.remove()
  })

  it('component declares static formAssociated = true', () => {
    const ctor = customElements.get(tag) as any
    expect(ctor.formAssociated).toBe(true)
  })

  it('host can be attached inside a <form> without error', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'agree')
    el.setAttribute('value', 'yes')
    const form = document.createElement('form')
    form.appendChild(el)
    document.body.appendChild(form)
    expect(() => form.elements).not.toThrow()
    form.remove()
  })
})
