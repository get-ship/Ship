import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-input (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-input.set')
  }, 120_000)

  it('renders a native input wrapped in a .field container', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    const field = el.querySelector('.field')
    expect(input).not.toBeNull()
    expect(field).not.toBeNull()
    expect(input.type).toBe('text')
    el.remove()
  })

  it('value attribute flows to the native input', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', 'hello')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.value).toBe('hello')
    el.remove()
  })

  it('typing fires ship-input with the new value', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const events: any[] = []
    el.addEventListener('ship-input', (e: any) => events.push(e.detail))

    const input = el.querySelector('input') as HTMLInputElement
    input.value = 'typed'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    expect(events).toEqual([{ value: 'typed' }])
    el.remove()
  })

  it('type/placeholder/name/autocomplete propagate', () => {
    const el = document.createElement(tag)
    el.setAttribute('type', 'email')
    el.setAttribute('placeholder', 'you@ship.dev')
    el.setAttribute('name', 'email')
    el.setAttribute('autocomplete', 'email')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.type).toBe('email')
    expect(input.placeholder).toBe('you@ship.dev')
    expect(input.name).toBe('email')
    expect(input.autocomplete).toBe('email')
    el.remove()
  })

  it('autocomplete defaults to "off" when not set', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.autocomplete).toBe('off')
    el.remove()
  })

  it('disabled / readonly / required reflect to the native input', () => {
    const el = document.createElement(tag)
    el.setAttribute('disabled', '')
    el.setAttribute('readonly', '')
    el.setAttribute('required', '')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.disabled).toBe(true)
    expect(input.readOnly).toBe(true)
    expect(input.required).toBe(true)
    expect(input.getAttribute('aria-required')).toBe('true')
    el.remove()
  })

  it('invalid sets aria-invalid and renders the error message via aria-describedby', () => {
    const el = document.createElement(tag)
    el.setAttribute('invalid', '')
    el.setAttribute('error-message', 'Required')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.getAttribute('aria-invalid')).toBe('true')
    const describedBy = input.getAttribute('aria-describedby')
    expect(describedBy).toBeTruthy()
    expect(describedBy!.startsWith('ship-input-err-')).toBe(true)
    // Error message rendered as visually-hidden alert.
    const err = el.querySelector('[role="alert"]')
    expect(err).not.toBeNull()
    expect(err!.textContent).toBe('Required')
    expect(err!.id).toBe(describedBy)
    el.remove()
  })

  it('loading toggles aria-busy and the [loading] attribute drives spinner CSS', () => {
    const el = document.createElement(tag)
    el.setAttribute('loading', '')
    document.body.appendChild(el)
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.getAttribute('aria-busy')).toBe('true')
    expect(el.hasAttribute('loading')).toBe(true)
    // Spinner element is always in the DOM (CSS toggles visibility);
    // verify it's present and visibility is controlled by the host
    // attribute rather than conditional rendering.
    expect(el.querySelector('.spinner')).not.toBeNull()

    // Toggle off — aria-busy clears, host attribute clears, spinner
    // node remains but its CSS hides it.
    el.removeAttribute('loading')
    expect(input.getAttribute('aria-busy')).toBe('false')
    expect(el.hasAttribute('loading')).toBe(false)
    el.remove()
  })

  it('leading and trailing slots distribute icons', () => {
    const el = document.createElement(tag)
    const lead = document.createElement('span')
    lead.setAttribute('slot', 'leading')
    lead.textContent = '🔍'
    const trail = document.createElement('span')
    trail.setAttribute('slot', 'trailing')
    trail.textContent = '✕'
    el.appendChild(lead)
    el.appendChild(trail)
    document.body.appendChild(el)
    const field = el.querySelector('.field')!
    expect(field.textContent).toContain('🔍')
    expect(field.textContent).toContain('✕')
    el.remove()
  })

  it('component declares static formAssociated = true', () => {
    const ctor = customElements.get(tag) as any
    expect(ctor.formAssociated).toBe(true)
  })

  it('tone × size × variant matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    const variants = ['default', 'ghost']
    let n = 0
    for (const tone of tones) {
      for (const size of sizes) {
        for (const variant of variants) {
          const el = document.createElement(tag)
          el.setAttribute('tone', tone)
          el.setAttribute('size', size)
          el.setAttribute('variant', variant)
          document.body.appendChild(el)
          expect(el.querySelector('input')).not.toBeNull()
          el.remove()
          n++
        }
      }
    }
    expect(n).toBe(7 * 3 * 2)
  })

  it('label flows to aria-label', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Email address')
    document.body.appendChild(el)
    expect(el.querySelector('input')!.getAttribute('aria-label')).toBe('Email address')
    el.remove()
  })
})
