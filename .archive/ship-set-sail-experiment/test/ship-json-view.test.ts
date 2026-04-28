import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-json-view (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-json-view.set')
  }, 120_000)

  it('renders without error when mounted with no data', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el).not.toBeNull()
    el.remove()
  })

  it('host carries role=tree and aria-label=JSON tree', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('tree')
    expect(el.getAttribute('aria-label')).toBe('JSON tree')
    el.remove()
  })

  it('renders string, number, boolean, null primitives', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', JSON.stringify({ a: 1, b: true, c: null, d: 'hi' }))
    document.body.appendChild(el)
    const txt = el.textContent!
    expect(txt).toContain('"a"')
    expect(txt).toContain('1')
    expect(txt).toContain('true')
    expect(txt).toContain('null')
    expect(txt).toContain('"hi"')
    el.remove()
  })

  it('renders part=root container for valid JSON', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{"x":1}')
    document.body.appendChild(el)
    expect(el.querySelector('[part="root"]')).not.toBeNull()
    el.remove()
  })

  it('shows error with role=alert for invalid JSON', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{bad json}')
    document.body.appendChild(el)
    const alert = el.querySelector('[role="alert"]')
    expect(alert).not.toBeNull()
    expect(alert!.textContent).toContain('Invalid JSON')
    el.remove()
  })

  it('depth=0 collapses the root object', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{"outer":{"inner":1}}')
    el.setAttribute('depth', '0')
    document.body.appendChild(el)
    expect(el.querySelectorAll('details[open]').length).toBe(0)
    el.remove()
  })

  it('depth=2 expands two levels', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{"outer":{"inner":1}}')
    el.setAttribute('depth', '2')
    document.body.appendChild(el)
    expect(el.querySelectorAll('details[open]').length).toBeGreaterThanOrEqual(2)
    el.remove()
  })

  it('expanded=true force-expands all levels regardless of depth', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{"a":{"b":{"c":1}}}')
    el.setAttribute('depth', '0')
    el.setAttribute('expanded', '')
    document.body.appendChild(el)
    expect(el.querySelectorAll('details[open]').length).toBeGreaterThan(0)
    el.remove()
  })

  it('renders array with items count in meta span', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '[1,2,3]')
    document.body.appendChild(el)
    expect(el.textContent).toContain('3 items')
    el.remove()
  })

  it('renders object with keys count in meta span', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{"a":1,"b":2}')
    document.body.appendChild(el)
    expect(el.textContent).toContain('2 keys')
    el.remove()
  })

  it('empty object renders without a details element', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{}')
    document.body.appendChild(el)
    expect(el.querySelector('details')).toBeNull()
    expect(el.textContent).toContain('{}')
    el.remove()
  })

  it('empty array renders without a details element', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '[]')
    document.body.appendChild(el)
    expect(el.querySelector('details')).toBeNull()
    expect(el.textContent).toContain('[]')
    el.remove()
  })

  it('reacts to data attribute change', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '"hello"')
    document.body.appendChild(el)
    expect(el.textContent).toContain('hello')
    el.setAttribute('data', '"world"')
    expect(el.textContent).toContain('world')
    el.remove()
  })

  it('reacts to expanded attribute toggled on', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', '{"outer":{"inner":1}}')
    el.setAttribute('depth', '0')
    document.body.appendChild(el)
    expect(el.querySelectorAll('details[open]').length).toBe(0)
    el.setAttribute('expanded', '')
    expect(el.querySelectorAll('details[open]').length).toBeGreaterThan(0)
    el.remove()
  })

  it('XSS-safe: HTML in string values is escaped and not injected as elements', () => {
    const el = document.createElement(tag)
    el.setAttribute('data', JSON.stringify({ xss: '<script>alert(1)</script>' }))
    document.body.appendChild(el)
    expect(el.textContent).toContain('<script>')
    expect(el.querySelector('script')).toBeNull()
    el.remove()
  })
})
