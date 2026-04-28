import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-metachip (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-metachip.set')
  }, 120_000)

  it('renders label + value from attrs', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Status')
    el.setAttribute('value', 'Draft')
    document.body.appendChild(el)
    const k = el.querySelector('.k')!
    const v = el.querySelector('.v')!
    expect(k.textContent).toContain('Status')
    expect(v.textContent).toContain('Draft')
    el.remove()
  })

  it('default slot content overrides value attr', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Pipeline')
    el.appendChild(document.createTextNode('tdd'))
    document.body.appendChild(el)
    const k = el.querySelector('.k')!
    const v = el.querySelector('.v')!
    expect(k.textContent).toContain('Pipeline')
    expect(v.textContent).toContain('tdd')
    expect(v.textContent).not.toContain('Pipeline')
    el.remove()
  })

  it('label slot overrides label attr', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'fallback')
    el.setAttribute('value', 'V')
    const code = document.createElement('code')
    code.setAttribute('slot', 'label')
    code.textContent = 'AGENT'
    el.appendChild(code)
    document.body.appendChild(el)
    const k = el.querySelector('.k')!
    expect(k.textContent).toContain('AGENT')
    expect(k.textContent).not.toContain('fallback')
    el.remove()
  })

  it('persists label + value attributes on the host', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Agent')
    el.setAttribute('value', 'test-writer')
    document.body.appendChild(el)
    expect(el.getAttribute('label')).toBe('Agent')
    expect(el.getAttribute('value')).toBe('test-writer')
    el.remove()
  })

  it('exposes label + value parts', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'A')
    el.setAttribute('value', 'B')
    document.body.appendChild(el)
    expect(el.querySelector('[part="label"]')).not.toBeNull()
    expect(el.querySelector('[part="value"]')).not.toBeNull()
    el.remove()
  })

  it('injects scoped CSS into document head exactly once', () => {
    const a = document.createElement(tag)
    const b = document.createElement(tag)
    document.body.appendChild(a)
    document.body.appendChild(b)
    const styles = document.head.querySelectorAll('style[data-sail="ship-metachip"]')
    expect(styles.length).toBe(1)
    const css = styles[0]!.textContent!
    expect(css).toContain('[data-component="ship-metachip"]')
    expect(css).not.toContain(':host')
    a.remove()
    b.remove()
  })
})
