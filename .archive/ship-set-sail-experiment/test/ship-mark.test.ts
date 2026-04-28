import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-mark (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-mark.set')
  }, 120_000)

  it('renders an <svg> with viewBox + aria-hidden', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const svg = el.querySelector('svg')!
    expect(svg).not.toBeNull()
    expect(svg.getAttribute('viewBox')).toContain('-260.519')
    expect(svg.getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('exposes the documented part="svg"', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('[part="svg"]')).not.toBeNull()
    el.remove()
  })

  it('reflects size attribute', () => {
    const el = document.createElement(tag)
    el.setAttribute('size', 'lg')
    document.body.appendChild(el)
    expect(el.getAttribute('size')).toBe('lg')
    el.remove()
  })

  it('renders the linear-gradient defs (stop classes drive theme tinting)', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const defs = el.querySelector('defs')!
    expect(defs).not.toBeNull()
    expect(defs.querySelectorAll('linearGradient').length).toBe(5)
    el.remove()
  })

  it('size matrix renders without error', () => {
    for (const size of ['sm', 'md', 'lg']) {
      const el = document.createElement(tag)
      el.setAttribute('size', size)
      document.body.appendChild(el)
      expect(el.querySelector('svg')).not.toBeNull()
      el.remove()
    }
  })
})
