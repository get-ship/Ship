import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-app-icon (sail)', () => {
  let tag: string

  beforeAll(async () => {
    // ship-app-icon embeds <ship-icon> in its template, so make sure the
    // inner element is registered before any test renders.
    await loadSailComponent('ship-icon.set')
    tag = await loadSailComponent('ship-app-icon.set')
  }, 120_000)

  it('renders a frame with the expected glyph for a known app', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'dashboard')
    document.body.appendChild(el)
    const inner = el.querySelector('ship-icon')
    expect(inner).not.toBeNull()
    expect(inner!.getAttribute('name')).toBe('layout-dashboard')
    el.remove()
  })

  it('reflects the canonical glyph for every known app', () => {
    const map: Record<string, string> = {
      dashboard: 'layout-dashboard',
      pipelines: 'workflow',
      agents: 'bot',
      skills: 'sparkles',
      git: 'git-branch',
      collections: 'database',
      canvas: 'frame',
      settings: 'settings',
      chat: 'messages-square',
      events: 'radio',
      specs: 'file-text',
      theme: 'palette',
      setup: 'rocket',
      overview: 'compass',
      mesh: 'network',
    }
    for (const [app, glyph] of Object.entries(map)) {
      const el = document.createElement(tag)
      el.setAttribute('name', app)
      document.body.appendChild(el)
      const inner = el.querySelector('ship-icon')!
      expect(inner.getAttribute('name')).toBe(glyph)
      el.remove()
    }
  })

  it('aliases theme-studio → theme palette glyph', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'theme-studio')
    document.body.appendChild(el)
    expect(el.querySelector('ship-icon')!.getAttribute('name')).toBe('palette')
    el.remove()
  })

  it('falls back to a neutral "dot" glyph for unknown app ids', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'nope')
    document.body.appendChild(el)
    expect(el.querySelector('ship-icon')!.getAttribute('name')).toBe('dot')
    el.remove()
  })

  it('reactively updates the glyph when name changes', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'dashboard')
    document.body.appendChild(el)
    expect(el.querySelector('ship-icon')!.getAttribute('name')).toBe('layout-dashboard')
    el.setAttribute('name', 'agents')
    expect(el.querySelector('ship-icon')!.getAttribute('name')).toBe('bot')
    el.remove()
  })

  it('without a label, host has aria-hidden=true and no role/aria-label', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'dashboard')
    document.body.appendChild(el)
    expect(el.getAttribute('aria-hidden')).toBe('true')
    expect(el.getAttribute('role')).toBeNull()
    expect(el.getAttribute('aria-label')).toBeNull()
    el.remove()
  })

  it('with a label, host gets role=img + aria-label and no aria-hidden', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'dashboard')
    el.setAttribute('label', 'Workspace dashboard')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('img')
    expect(el.getAttribute('aria-label')).toBe('Workspace dashboard')
    expect(el.getAttribute('aria-hidden')).toBeNull()
    el.remove()
  })

  it('inner glyph carries aria-hidden when the host is unlabeled', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'dashboard')
    document.body.appendChild(el)
    expect(el.querySelector('ship-icon')!.getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('host carries the label, role=img, and aria-label end-to-end', () => {
    // The inner ship-icon manages its OWN aria-hidden based on whether
    // it has a label of its own (it doesn't — it inherits hidden status
    // from its host context). Per the Lit original, aria-hidden
    // bookkeeping on the inner glyph is owned by ship-icon itself, so we
    // assert host-level a11y here, not the inner element.
    const el = document.createElement(tag)
    el.setAttribute('name', 'dashboard')
    el.setAttribute('label', 'Workspace dashboard')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('img')
    expect(el.getAttribute('aria-label')).toBe('Workspace dashboard')
    expect(el.getAttribute('aria-hidden')).toBeNull()
    expect(el.querySelector('ship-icon')).not.toBeNull()
    el.remove()
  })

  it('respects a custom slotted glyph (escape hatch)', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'dashboard')
    const custom = document.createElement('img')
    custom.setAttribute('data-custom', '')
    custom.setAttribute('src', '/marketplace/foo.svg')
    el.appendChild(custom)
    document.body.appendChild(el)
    expect(el.querySelector('img[data-custom]')).not.toBeNull()
    el.remove()
  })

  it('variant × size × radius matrix renders without error', () => {
    const variants = ['glyph', 'tile', 'solid']
    const sizes = ['sm', 'md', 'lg']
    const radii = ['', 'sm', 'md', 'lg', 'full']
    let n = 0
    for (const variant of variants) {
      for (const size of sizes) {
        for (const radius of radii) {
          const el = document.createElement(tag)
          el.setAttribute('name', 'dashboard')
          el.setAttribute('variant', variant)
          el.setAttribute('size', size)
          if (radius) el.setAttribute('radius', radius)
          document.body.appendChild(el)
          expect(el.querySelector('.frame')).not.toBeNull()
          el.remove()
          n++
        }
      }
    }
    expect(n).toBe(3 * 3 * 5)
  })
})
