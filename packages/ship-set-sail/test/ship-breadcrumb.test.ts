import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-breadcrumb (sail)', () => {
  let bcTag: string
  let itemTag: string
  beforeAll(async () => {
    itemTag = await loadSailComponent('ship-breadcrumb-item.set')
    bcTag = await loadSailComponent('ship-breadcrumb.set')
  }, 120_000)

  function mountBc(itemSpecs: Array<{ href?: string; active?: boolean; text: string }>) {
    const bc = document.createElement(bcTag)
    for (const spec of itemSpecs) {
      const it = document.createElement(itemTag)
      if (spec.href) it.setAttribute('href', spec.href)
      if (spec.active) it.setAttribute('active', '')
      it.appendChild(document.createTextNode(spec.text))
      bc.appendChild(it)
    }
    document.body.appendChild(bc)
    return bc
  }

  it('renders an outer <nav aria-label="Breadcrumb">', () => {
    const bc = mountBc([{ href: '/', text: 'Home' }])
    const nav = bc.querySelector('nav[part="nav"]')!
    expect(nav).not.toBeNull()
    expect(nav.getAttribute('aria-label')).toBe('Breadcrumb')
    bc.remove()
  })

  it('renders N-1 separators for N items', async () => {
    const bc = mountBc([
      { href: '/', text: 'Home' },
      { href: '/a', text: 'A' },
      { active: true, text: 'B' },
    ])
    // Allow the MutationObserver-driven sync one tick.
    await Promise.resolve()
    const seps = bc.querySelectorAll('.sep')
    expect(seps.length).toBe(2)
    bc.remove()
  })

  it('breadcrumb-item with href renders an <a href=…>', () => {
    const item = document.createElement(itemTag)
    item.setAttribute('href', '/x')
    item.appendChild(document.createTextNode('X'))
    document.body.appendChild(item)
    const a = item.querySelector('a') as HTMLAnchorElement
    expect(a).not.toBeNull()
    expect(a.getAttribute('href')).toBe('/x')
    item.remove()
  })

  it('active item renders the .label span with aria-current=page', () => {
    const item = document.createElement(itemTag)
    item.setAttribute('active', '')
    item.setAttribute('href', '/z')
    item.appendChild(document.createTextNode('Z'))
    document.body.appendChild(item)
    const label = item.querySelector('.label')!
    expect(label).not.toBeNull()
    expect(label.getAttribute('aria-current')).toBe('page')
    item.remove()
  })

  it('item without href renders a label span (not an anchor)', () => {
    const item = document.createElement(itemTag)
    item.appendChild(document.createTextNode('plain'))
    document.body.appendChild(item)
    expect(item.querySelector('a')).toBeNull()
    expect(item.querySelector('.label')).not.toBeNull()
    item.remove()
  })
})
