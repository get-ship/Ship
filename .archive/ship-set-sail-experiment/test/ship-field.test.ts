import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-field (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-field.set')
  }, 120_000)

  it('renders label, default-control, hint and error regions', () => {
    const el = document.createElement(tag)
    const label = document.createElement('span')
    label.setAttribute('slot', 'label')
    label.textContent = 'L'
    const control = document.createElement('input')
    const hint = document.createElement('span')
    hint.setAttribute('slot', 'hint')
    hint.textContent = 'H'
    el.appendChild(label)
    el.appendChild(control)
    el.appendChild(hint)
    document.body.appendChild(el)

    expect(el.querySelector('.label')).not.toBeNull()
    expect(el.querySelector('.hint')).not.toBeNull()
    expect(el.querySelector('.error')).not.toBeNull()
    expect(el.querySelector('input')).not.toBeNull()
    expect(el.querySelector('.label')!.textContent).toContain('L')
    expect(el.querySelector('.hint')!.textContent).toContain('H')
    el.remove()
  })

  it('part names label, hint, error are exposed', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('[part="label"]')).not.toBeNull()
    expect(el.querySelector('[part="hint"]')).not.toBeNull()
    expect(el.querySelector('[part="error"]')).not.toBeNull()
    el.remove()
  })

  it('error region declares role=alert and aria-live=polite', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const err = el.querySelector('.error')!
    expect(err.getAttribute('role')).toBe('alert')
    expect(err.getAttribute('aria-live')).toBe('polite')
    el.remove()
  })

  it('invalid attribute is reflected on the host', () => {
    const el = document.createElement(tag)
    el.setAttribute('invalid', '')
    document.body.appendChild(el)
    expect(el.hasAttribute('invalid')).toBe(true)
    el.remove()
  })

  it('wires aria-describedby on the slotted control to the hint id when not invalid', async () => {
    const el = document.createElement(tag)
    const input = document.createElement('input')
    el.appendChild(input)
    document.body.appendChild(el)
    await Promise.resolve()
    const described = input.getAttribute('aria-describedby')
    expect(described).toBeTruthy()
    expect(described!.startsWith('ship-field-hint-')).toBe(true)
    expect(input.getAttribute('aria-invalid')).toBe('false')
    el.remove()
  })

  it('wires aria-describedby to the error id when invalid is set', async () => {
    const el = document.createElement(tag)
    el.setAttribute('invalid', '')
    const input = document.createElement('input')
    el.appendChild(input)
    document.body.appendChild(el)
    await Promise.resolve()
    const described = input.getAttribute('aria-describedby')
    expect(described).toBeTruthy()
    expect(described!.startsWith('ship-field-error-')).toBe(true)
    expect(input.getAttribute('aria-invalid')).toBe('true')
    el.remove()
  })

  it('hint id matches the .hint container id (and same for error)', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const hint = el.querySelector('.hint')!
    const error = el.querySelector('.error')!
    expect(hint.id).toMatch(/^ship-field-hint-/)
    expect(error.id).toMatch(/^ship-field-error-/)
    el.remove()
  })

  it('preserves prior aria-describedby tokens when wiring', async () => {
    const el = document.createElement(tag)
    const input = document.createElement('input')
    input.setAttribute('aria-describedby', 'caller-token')
    el.appendChild(input)
    document.body.appendChild(el)
    await Promise.resolve()
    const tokens = (input.getAttribute('aria-describedby') || '').split(/\s+/)
    expect(tokens).toContain('caller-token')
    expect(tokens.some(t => t.startsWith('ship-field-hint-'))).toBe(true)
    el.remove()
  })

  it('flips the wired control between hint and error on invalid toggle', async () => {
    const el = document.createElement(tag)
    const input = document.createElement('input')
    el.appendChild(input)
    document.body.appendChild(el)
    await Promise.resolve()
    expect(input.getAttribute('aria-describedby')!.startsWith('ship-field-hint-')).toBe(true)
    el.setAttribute('invalid', '')
    await Promise.resolve()
    expect(input.getAttribute('aria-describedby')!.startsWith('ship-field-error-')).toBe(true)
    expect(input.getAttribute('aria-invalid')).toBe('true')
    el.removeAttribute('invalid')
    await Promise.resolve()
    expect(input.getAttribute('aria-describedby')!.startsWith('ship-field-hint-')).toBe(true)
    expect(input.getAttribute('aria-invalid')).toBe('false')
    el.remove()
  })

  it('sets the invalid property when the slotted control exposes one', async () => {
    const el = document.createElement(tag)
    // Object pretending to be an element with an `invalid` property.
    const fake = document.createElement('div') as HTMLDivElement & { invalid: boolean }
    fake.invalid = false
    el.appendChild(fake)
    document.body.appendChild(el)
    el.setAttribute('invalid', '')
    await Promise.resolve()
    expect(fake.invalid).toBe(true)
    expect(fake.hasAttribute('aria-invalid')).toBe(false)
    el.remove()
  })

  it('falls back to aria-invalid on plain controls without the property', async () => {
    const el = document.createElement(tag)
    const input = document.createElement('input')
    el.appendChild(input)
    document.body.appendChild(el)
    el.setAttribute('invalid', '')
    await Promise.resolve()
    expect(input.getAttribute('aria-invalid')).toBe('true')
    el.remove()
  })

  it('only the first non-named child is wired as the canonical control', async () => {
    const el = document.createElement(tag)
    const a = document.createElement('input')
    a.id = 'a'
    const b = document.createElement('input')
    b.id = 'b'
    el.appendChild(a)
    el.appendChild(b)
    document.body.appendChild(el)
    await Promise.resolve()
    expect(a.getAttribute('aria-describedby')).toBeTruthy()
    expect(b.getAttribute('aria-describedby')).toBeNull()
    el.remove()
  })

  it('re-wires when a control is appended after mount', async () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    await Promise.resolve()
    const input = document.createElement('input')
    el.appendChild(input)
    // Allow the MutationObserver tick to fire.
    await new Promise(r => setTimeout(r, 0))
    expect(input.getAttribute('aria-describedby')).toBeTruthy()
    el.remove()
  })
})
