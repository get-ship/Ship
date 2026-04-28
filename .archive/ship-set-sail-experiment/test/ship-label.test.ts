import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-label (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-label.set')
  }, 120_000)

  it('renders slotted text inside a <label>', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('Status'))
    document.body.appendChild(el)
    const label = el.querySelector('label')!
    expect(label).not.toBeNull()
    expect(label.textContent).toContain('Status')
    el.remove()
  })

  it('renders required marker when required attr is set', () => {
    const el = document.createElement(tag)
    el.setAttribute('required', '')
    el.appendChild(document.createTextNode('Title'))
    document.body.appendChild(el)
    expect(el.querySelector('.req')).not.toBeNull()
    expect(el.querySelector('.req')!.getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('omits required marker by default', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('x'))
    document.body.appendChild(el)
    expect(el.querySelector('.req')).toBeNull()
    el.remove()
  })

  it('forwards the `for` attribute to the inner <label for=…>', () => {
    const el = document.createElement(tag)
    el.setAttribute('for', 'x')
    el.appendChild(document.createTextNode('x'))
    document.body.appendChild(el)
    expect(el.querySelector('label')!.getAttribute('for')).toBe('x')
    el.remove()
  })

  it('reflects size attribute on host', () => {
    const el = document.createElement(tag)
    el.setAttribute('size', 'md')
    document.body.appendChild(el)
    expect(el.getAttribute('size')).toBe('md')
    el.remove()
  })
})
