import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-empty-state (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-empty-state.set')
  }, 120_000)

  it('renders the box with default role=note and aria-live=off', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const box = el.querySelector('.box')!
    expect(box).not.toBeNull()
    expect(box.getAttribute('role')).toBe('note')
    expect(box.getAttribute('aria-live')).toBe('off')
    el.remove()
  })

  it('connecting variant uses role=status + aria-live=polite', () => {
    const el = document.createElement(tag)
    el.setAttribute('variant', 'connecting')
    document.body.appendChild(el)
    const box = el.querySelector('.box')!
    expect(box.getAttribute('role')).toBe('status')
    expect(box.getAttribute('aria-live')).toBe('polite')
    el.remove()
  })

  it('placeholder variant keeps role=note + aria-live=off', () => {
    const el = document.createElement(tag)
    el.setAttribute('variant', 'placeholder')
    document.body.appendChild(el)
    const box = el.querySelector('.box')!
    expect(box.getAttribute('role')).toBe('note')
    expect(box.getAttribute('aria-live')).toBe('off')
    el.remove()
  })

  it('inline variant keeps role=note + aria-live=off', () => {
    const el = document.createElement(tag)
    el.setAttribute('variant', 'inline')
    document.body.appendChild(el)
    expect(el.querySelector('.box')!.getAttribute('role')).toBe('note')
    el.remove()
  })

  it('renders all four structural regions (icon, title, desc, actions)', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('.icon')).not.toBeNull()
    expect(el.querySelector('.title')).not.toBeNull()
    expect(el.querySelector('.desc')).not.toBeNull()
    expect(el.querySelector('.actions')).not.toBeNull()
    el.remove()
  })

  it('default slot routes content into .desc', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('Try another query.'))
    document.body.appendChild(el)
    expect(el.querySelector('.desc')!.textContent).toContain('Try another query.')
    el.remove()
  })

  it('title slot routes content into .title', () => {
    const el = document.createElement(tag)
    const t = document.createElement('span')
    t.setAttribute('slot', 'title')
    t.textContent = 'No results'
    el.appendChild(t)
    document.body.appendChild(el)
    expect(el.querySelector('.title')!.textContent).toContain('No results')
    el.remove()
  })

  it('icon slot routes content into .icon (slot wins over icon attr)', () => {
    const el = document.createElement(tag)
    el.setAttribute('icon', 'inbox') // attr present but slot should win
    const glyph = document.createElement('span')
    glyph.setAttribute('slot', 'icon')
    glyph.textContent = 'GLYPH'
    el.appendChild(glyph)
    document.body.appendChild(el)
    const tile = el.querySelector('.icon')!
    expect(tile.textContent).toContain('GLYPH')
    // Fallback <ship-icon> should NOT be rendered when slot is filled.
    expect(tile.querySelector('ship-icon')).toBeNull()
    el.remove()
  })

  it('action slot routes content into .actions', () => {
    const el = document.createElement(tag)
    const btn = document.createElement('button')
    btn.setAttribute('slot', 'action')
    btn.textContent = 'Start'
    el.appendChild(btn)
    document.body.appendChild(el)
    expect(el.querySelector('.actions')!.textContent).toContain('Start')
    el.remove()
  })

  it('icon attr renders fallback <ship-icon> inside the tile when slot is empty', () => {
    const el = document.createElement(tag)
    el.setAttribute('icon', 'inbox')
    document.body.appendChild(el)
    const tile = el.querySelector('.icon')!
    const glyph = tile.querySelector('ship-icon')
    expect(glyph).not.toBeNull()
    expect(glyph!.getAttribute('name')).toBe('inbox')
    expect(glyph!.getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('no icon attr and empty icon slot leaves the tile empty (collapses via :empty)', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const tile = el.querySelector('.icon')!
    // No fallback rendered, no slot fill — tile is empty.
    expect(tile.querySelector('ship-icon')).toBeNull()
    expect(tile.children.length).toBe(0)
    el.remove()
  })

  it('reflects variant attribute on the host', () => {
    const el = document.createElement(tag)
    el.setAttribute('variant', 'connecting')
    document.body.appendChild(el)
    expect(el.getAttribute('variant')).toBe('connecting')
    el.remove()
  })

  it('reflects size and tone attributes on the host', () => {
    const el = document.createElement(tag)
    el.setAttribute('size', 'lg')
    el.setAttribute('tone', 'success')
    document.body.appendChild(el)
    expect(el.getAttribute('size')).toBe('lg')
    expect(el.getAttribute('tone')).toBe('success')
    el.remove()
  })

  it('icon tile carries aria-hidden=true', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('.icon')!.getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('variant × size × tone matrix renders without error', () => {
    const variants = ['default', 'inline', 'placeholder', 'connecting']
    const sizes = ['sm', 'md', 'lg']
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    let n = 0
    for (const variant of variants) {
      for (const size of sizes) {
        for (const tone of tones) {
          const el = document.createElement(tag)
          el.setAttribute('variant', variant)
          el.setAttribute('size', size)
          el.setAttribute('tone', tone)
          document.body.appendChild(el)
          expect(el.querySelector('.box')).not.toBeNull()
          el.remove()
          n++
        }
      }
    }
    expect(n).toBe(4 * 3 * 7)
  })

  it('updating variant reactively swaps role + aria-live (default → connecting)', async () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const box = el.querySelector('.box')!
    expect(box.getAttribute('role')).toBe('note')
    el.setAttribute('variant', 'connecting')
    // Allow microtask flush for reactive bindings.
    await Promise.resolve()
    expect(box.getAttribute('role')).toBe('status')
    expect(box.getAttribute('aria-live')).toBe('polite')
    el.remove()
  })

  it('exposes the documented part names (box, icon, title, desc, actions)', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('[part="box"]')).not.toBeNull()
    expect(el.querySelector('[part="icon"]')).not.toBeNull()
    expect(el.querySelector('[part="title"]')).not.toBeNull()
    expect(el.querySelector('[part="desc"]')).not.toBeNull()
    expect(el.querySelector('[part="actions"]')).not.toBeNull()
    el.remove()
  })
})
