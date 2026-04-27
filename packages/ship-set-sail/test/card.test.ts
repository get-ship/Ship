import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-card (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-card.set')
  }, 120_000)

  it('renders header / content / footer slot wrappers', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('.header')).not.toBeNull()
    expect(el.querySelector('.content')).not.toBeNull()
    expect(el.querySelector('.footer')).not.toBeNull()
    el.remove()
  })

  it('default slot renders body content', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('Body text'))
    document.body.appendChild(el)
    expect(el.querySelector('.content')!.textContent).toContain('Body text')
    el.remove()
  })

  it('header slot renders heading content', () => {
    const el = document.createElement(tag)
    const heading = document.createElement('h2')
    heading.setAttribute('slot', 'header')
    heading.textContent = 'Card Title'
    el.appendChild(heading)
    el.appendChild(document.createTextNode('Body'))
    document.body.appendChild(el)
    expect(el.querySelector('.header')!.textContent).toContain('Card Title')
    el.remove()
  })

  it('footer slot renders meta content', () => {
    const el = document.createElement(tag)
    const meta = document.createElement('span')
    meta.setAttribute('slot', 'footer')
    meta.textContent = 'Updated 2h ago'
    el.appendChild(document.createTextNode('Body'))
    el.appendChild(meta)
    document.body.appendChild(el)
    expect(el.querySelector('.footer')!.textContent).toContain('Updated 2h ago')
    el.remove()
  })

  it('interactive sets role + tabindex and responds to Enter/Space', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('tabindex')).toBe('0')

    let clicks = 0
    el.addEventListener('click', () => { clicks++ })

    // Enter triggers click
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(clicks).toBe(1)

    // Space triggers click
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(clicks).toBe(2)

    // Other keys do not
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', bubbles: true }))
    expect(clicks).toBe(2)

    el.remove()
  })

  it('removing interactive cleans up role / tabindex', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('button')
    el.removeAttribute('interactive')
    expect(el.getAttribute('role')).toBeNull()
    expect(el.getAttribute('tabindex')).toBeNull()
    el.remove()
  })

  it('tone × variant × padding matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const variants = ['default', 'outline']
    const paddings = ['none', 'sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const variant of variants) {
        for (const padding of paddings) {
          const el = document.createElement(tag)
          el.setAttribute('tone', tone)
          el.setAttribute('variant', variant)
          el.setAttribute('padding', padding)
          document.body.appendChild(el)
          expect(el.querySelector('.content')).not.toBeNull()
          el.remove()
          n++
        }
      }
    }
    expect(n).toBe(7 * 2 * 4)
  })

  it('boolean flags (fill, accent-bar, status-rail) reflect to host', () => {
    for (const flag of ['fill', 'accent-bar', 'status-rail']) {
      const el = document.createElement(tag)
      el.setAttribute(flag, '')
      document.body.appendChild(el)
      expect(el.hasAttribute(flag)).toBe(true)
      el.remove()
    }
  })

  it('non-interactive cards do not handle keyboard', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    let clicks = 0
    el.addEventListener('click', () => { clicks++ })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(clicks).toBe(0)
    el.remove()
  })
})
