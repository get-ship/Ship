import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-provider-blob (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-provider-blob.set') }, 120_000)

  it('renders a tile with the Claude SVG by default', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const tile = el.querySelector('.tile')!
    expect(tile).not.toBeNull()
    expect(tile.getAttribute('role')).toBe('img')
    expect(tile.getAttribute('aria-label')).toBe('Claude')
    const svg = tile.querySelector('svg')
    expect(svg).not.toBeNull()
    expect(svg!.getAttribute('fill')).toBe('#D97757')
    el.remove()
  })

  it('uses brand-tinted host channel via --blob-hue inline style', () => {
    const el = document.createElement(tag) as HTMLElement
    el.setAttribute('provider', 'claude')
    document.body.appendChild(el)
    expect(el.style.getPropertyValue('--blob-hue')).toBe('oklch(0.70 0.16 35)')
    el.remove()
  })

  it('switches the SVG, label, and hue when provider changes', () => {
    const el = document.createElement(tag) as HTMLElement
    el.setAttribute('provider', 'claude')
    document.body.appendChild(el)
    el.setAttribute('provider', 'gemini')
    const tile = el.querySelector('.tile')!
    expect(tile.getAttribute('aria-label')).toBe('Gemini')
    const svg = tile.querySelector('svg')!
    expect(svg.getAttribute('fill')).toBe('#8E75B2')
    expect(el.style.getPropertyValue('--blob-hue')).toBe('oklch(0.62 0.14 285)')
    el.remove()
  })

  it('renders the Cursor mark with currentColor fill', () => {
    const el = document.createElement(tag)
    el.setAttribute('provider', 'cursor')
    document.body.appendChild(el)
    const svg = el.querySelector('.tile svg')!
    expect(svg.getAttribute('fill')).toBe('currentColor')
    expect(el.querySelector('.tile')!.getAttribute('aria-label')).toBe('Cursor')
    el.remove()
  })

  it('codex renders the OpenAI mark', () => {
    const el = document.createElement(tag)
    el.setAttribute('provider', 'codex')
    document.body.appendChild(el)
    const tile = el.querySelector('.tile')!
    expect(tile.getAttribute('aria-label')).toBe('Codex')
    // The OpenAI mark uses a 721x721 viewBox; use that as a stable signature.
    const svg = tile.querySelector('svg')!
    expect(svg.getAttribute('viewBox')).toBe('0 0 721 721')
    el.remove()
  })

  it('opencode renders its 240x300 monoblossom', () => {
    const el = document.createElement(tag)
    el.setAttribute('provider', 'opencode')
    document.body.appendChild(el)
    const svg = el.querySelector('.tile svg')!
    expect(svg.getAttribute('viewBox')).toBe('0 0 240 300')
    expect(el.querySelector('.tile')!.getAttribute('aria-label')).toBe('OpenCode')
    el.remove()
  })

  it('falls back to Claude for an unknown provider', () => {
    const el = document.createElement(tag)
    el.setAttribute('provider', 'mystery')
    document.body.appendChild(el)
    const tile = el.querySelector('.tile')!
    expect(tile.getAttribute('aria-label')).toBe('Claude')
    el.remove()
  })

  it('reflects size and variant attrs on the host', () => {
    const el = document.createElement(tag)
    el.setAttribute('size', 'lg')
    el.setAttribute('variant', 'glyph')
    document.body.appendChild(el)
    expect(el.getAttribute('size')).toBe('lg')
    expect(el.getAttribute('variant')).toBe('glyph')
    el.remove()
  })

  it('default slot renders label text inside the .label region', () => {
    const el = document.createElement(tag)
    el.setAttribute('provider', 'claude')
    el.appendChild(document.createTextNode('Claude'))
    document.body.appendChild(el)
    const label = el.querySelector('.label')!
    expect(label.textContent).toContain('Claude')
    el.remove()
  })

  it('exposes part="tile" and part="label" for external styling', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('[part="tile"]')).not.toBeNull()
    expect(el.querySelector('[part="label"]')).not.toBeNull()
    el.remove()
  })
})
