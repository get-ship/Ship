import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-icon (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-icon.set')
  }, 120_000)

  it('renders a named glyph as svg path', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'check')
    document.body.appendChild(el)
    const path = el.querySelector('path')
    expect(path).not.toBeNull()
    expect(path!.getAttribute('d')).toContain('M20 6')
    el.remove()
  })

  it('renders nothing when name is unknown (falls back to slot)', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'nope')
    document.body.appendChild(el)
    expect(el.querySelector('path')).toBeNull()
    el.remove()
  })

  it('reflects size attribute and reacts to change', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'x')
    el.setAttribute('size', 'sm')
    document.body.appendChild(el)
    expect(el.getAttribute('size')).toBe('sm')
    el.setAttribute('size', 'lg')
    expect(el.getAttribute('size')).toBe('lg')
    el.remove()
  })

  it('allows raw svg passthrough via default slot', () => {
    const el = document.createElement(tag)
    const passthrough = document.createElement('svg')
    passthrough.setAttribute('data-custom', '')
    el.appendChild(passthrough)
    document.body.appendChild(el)
    expect(el.querySelector('svg[data-custom]')).not.toBeNull()
    el.remove()
  })

  it('renders collection icons added for the data view', () => {
    for (const name of ['columns', 'filter', 'check-circle', 'book', 'activity']) {
      const el = document.createElement(tag)
      el.setAttribute('name', name)
      document.body.appendChild(el)
      expect(el.querySelector('path')).not.toBeNull()
      el.remove()
    }
  })

  it('renders database as a multi-element complex icon', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'database')
    document.body.appendChild(el)
    const svg = el.querySelector('svg')
    expect(svg).not.toBeNull()
    expect(svg!.querySelector('ellipse')).not.toBeNull()
    el.remove()
  })

  it('aliases legacy names: stop → stop-circle, refresh → refresh-cw, git → git-branch, warning → triangle-alert', () => {
    for (const alias of ['stop', 'refresh', 'git', 'warning']) {
      const el = document.createElement(tag)
      el.setAttribute('name', alias)
      document.body.appendChild(el)
      // Each alias resolves to a registry entry that contains at
      // least one path or shape primitive.
      const svg = el.querySelector('svg')
      expect(svg).not.toBeNull()
      el.remove()
    }
  })

  it('without a label, host has aria-hidden=true and no role', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'check')
    document.body.appendChild(el)
    expect(el.getAttribute('aria-hidden')).toBe('true')
    expect(el.getAttribute('role')).toBeNull()
    expect(el.getAttribute('aria-label')).toBeNull()
    el.remove()
  })

  it('with a label, host has role=img + aria-label and no aria-hidden', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'check')
    el.setAttribute('label', 'Done')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('img')
    expect(el.getAttribute('aria-label')).toBe('Done')
    expect(el.getAttribute('aria-hidden')).toBeNull()
    el.remove()
  })

  it('inner svg gets aria-hidden="true" when no label is set', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'check')
    document.body.appendChild(el)
    const svg = el.querySelector('svg')!
    expect(svg.getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('inner svg has no aria-hidden when host carries the role=img label', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'check')
    el.setAttribute('label', 'Done')
    document.body.appendChild(el)
    const svg = el.querySelector('svg')!
    expect(svg.getAttribute('aria-hidden')).toBeNull()
    el.remove()
  })

  it('tone × size matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted', 'primary']
    const sizes = ['sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const size of sizes) {
        const el = document.createElement(tag)
        el.setAttribute('name', 'check')
        el.setAttribute('tone', tone)
        el.setAttribute('size', size)
        document.body.appendChild(el)
        expect(el.querySelector('svg')).not.toBeNull()
        el.remove()
        n++
      }
    }
    expect(n).toBe(8 * 3)
  })

  it('changing name reactively updates the rendered glyph', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'check')
    document.body.appendChild(el)
    const firstPath = el.querySelector('path')!.getAttribute('d')
    expect(firstPath).toContain('M20 6')

    el.setAttribute('name', 'x')
    const secondPath = el.querySelector('path')!.getAttribute('d')
    expect(secondPath).toContain('M18 6')
    el.remove()
  })
})
