import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-error-state (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-error-state.set')
  }, 120_000)

  it('renders the box with role=alert', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    document.body.appendChild(el)
    const box = el.querySelector('.box')!
    expect(box).not.toBeNull()
    expect(box.getAttribute('role')).toBe('alert')
    el.remove()
  })

  it('renders title attr as default title text', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'Boom')
    document.body.appendChild(el)
    expect(el.querySelector('.title')!.textContent).toContain('Boom')
    el.remove()
  })

  it('falls back to "Something went wrong" when no title attr', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('.title')!.textContent).toContain('Something went wrong')
    el.remove()
  })

  it('default slot renders into .desc', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    el.appendChild(document.createTextNode('Try again.'))
    document.body.appendChild(el)
    expect(el.querySelector('.desc')!.textContent).toContain('Try again.')
    el.remove()
  })

  it('body attr falls through to .desc when slot is empty', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    el.setAttribute('body', 'Network is down.')
    document.body.appendChild(el)
    expect(el.querySelector('.desc')!.textContent).toContain('Network is down.')
    el.remove()
  })

  it('omits the details toggle when no details slot is supplied', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    document.body.appendChild(el)
    expect(el.querySelector('.details-toggle')).toBeNull()
    el.remove()
  })

  it('renders a details toggle when [slot=details] content is supplied', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    const pre = document.createElement('pre')
    pre.setAttribute('slot', 'details')
    pre.textContent = 'stack trace here'
    el.appendChild(pre)
    document.body.appendChild(el)
    const toggle = el.querySelector('.details-toggle') as HTMLButtonElement
    expect(toggle).not.toBeNull()
    expect(toggle.getAttribute('aria-expanded')).toBe('false')
    expect(toggle.textContent).toContain('Show details')
    el.remove()
  })

  it('toggles details panel + aria-expanded on click', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    const pre = document.createElement('pre')
    pre.setAttribute('slot', 'details')
    pre.textContent = 'stack'
    el.appendChild(pre)
    document.body.appendChild(el)
    const toggle = el.querySelector('.details-toggle') as HTMLButtonElement
    toggle.click()
    expect(toggle.getAttribute('aria-expanded')).toBe('true')
    expect(toggle.textContent).toContain('Hide details')
    el.remove()
  })

  it('renders ship-icon glyph by default (icon=alert-circle)', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    document.body.appendChild(el)
    const glyph = el.querySelector('.icon ship-icon')
    expect(glyph).not.toBeNull()
    expect(glyph!.getAttribute('name')).toBe('alert-circle')
    el.remove()
  })

  it('reflects size + variant attributes on the host', () => {
    const el = document.createElement(tag)
    el.setAttribute('size', 'lg')
    el.setAttribute('variant', 'inline')
    document.body.appendChild(el)
    expect(el.getAttribute('size')).toBe('lg')
    expect(el.getAttribute('variant')).toBe('inline')
    el.remove()
  })

  it('action slot routes content into .actions', () => {
    const el = document.createElement(tag)
    el.setAttribute('title', 'x')
    const btn = document.createElement('button')
    btn.setAttribute('slot', 'action')
    btn.textContent = 'Retry'
    el.appendChild(btn)
    document.body.appendChild(el)
    expect(el.querySelector('.actions')!.textContent).toContain('Retry')
    el.remove()
  })
})
