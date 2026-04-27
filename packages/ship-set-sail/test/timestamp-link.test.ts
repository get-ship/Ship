import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-timestamp (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-timestamp.set') }, 120_000)

  it('renders a relative time by default', () => {
    const el = document.createElement(tag)
    const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000).toISOString()
    el.setAttribute('value', oneHourAgo)
    document.body.appendChild(el)
    const time = el.querySelector('time')!
    expect(time.textContent).toContain('1h ago')
    expect(time.getAttribute('datetime')).toBe(oneHourAgo)
    el.remove()
  })

  it('format=absolute renders the ISO string', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '2024-01-15T10:30:00Z')
    el.setAttribute('format', 'absolute')
    document.body.appendChild(el)
    expect(el.querySelector('time')!.textContent).toContain('2024-01-15')
    el.remove()
  })

  it('format=both renders relative + absolute side by side', () => {
    const el = document.createElement(tag)
    const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000).toISOString()
    el.setAttribute('value', oneHourAgo)
    el.setAttribute('format', 'both')
    document.body.appendChild(el)
    expect(el.querySelector('time')!.textContent).toContain('1h ago')
    expect(el.querySelector('.abs')!.textContent).toContain(oneHourAgo)
    el.remove()
  })

  it('"just now" for sub-minute differences', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', new Date().toISOString())
    document.body.appendChild(el)
    expect(el.querySelector('time')!.textContent).toBe('just now')
    el.remove()
  })

  it('future timestamps say "from now"', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', new Date(Date.now() + 2 * 60 * 60 * 1000).toISOString())
    document.body.appendChild(el)
    expect(el.querySelector('time')!.textContent).toContain('from now')
    el.remove()
  })

  it('empty value renders an empty span (no time element)', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('time')).toBeNull()
    el.remove()
  })

  it('exposes the absolute ISO via title attribute', () => {
    const el = document.createElement(tag)
    el.setAttribute('value', '2024-01-15T10:30:00Z')
    document.body.appendChild(el)
    expect(el.title).toBe('2024-01-15T10:30:00.000Z')
    el.remove()
  })
})

describe('ship-link (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-link.set') }, 120_000)

  it('wraps default slot in an anchor with href', () => {
    const el = document.createElement(tag)
    el.setAttribute('href', '/dashboard')
    el.appendChild(document.createTextNode('Dashboard'))
    document.body.appendChild(el)
    const a = el.querySelector('a') as HTMLAnchorElement
    expect(a).not.toBeNull()
    expect(a.getAttribute('href')).toBe('/dashboard')
    expect(a.textContent).toContain('Dashboard')
    el.remove()
  })

  it('external sets target=_blank and rel + renders ext icon', () => {
    const el = document.createElement(tag)
    el.setAttribute('href', 'https://example.com')
    el.setAttribute('external', '')
    el.appendChild(document.createTextNode('External'))
    document.body.appendChild(el)
    const a = el.querySelector('a') as HTMLAnchorElement
    expect(a.getAttribute('target')).toBe('_blank')
    expect(a.getAttribute('rel')).toBe('noopener noreferrer')
    expect(el.querySelector('.ext')).not.toBeNull()
    expect(el.querySelector('.sr-only')!.textContent).toContain('opens in new tab')
    el.remove()
  })

  it('non-external links omit target/rel + ext icon', () => {
    const el = document.createElement(tag)
    el.setAttribute('href', '/local')
    document.body.appendChild(el)
    const a = el.querySelector('a') as HTMLAnchorElement
    expect(a.getAttribute('target')).toBe('')
    expect(el.querySelector('.ext')).toBeNull()
    el.remove()
  })

  it('tone=muted reflects to host', () => {
    const el = document.createElement(tag)
    el.setAttribute('tone', 'muted')
    document.body.appendChild(el)
    expect(el.getAttribute('tone')).toBe('muted')
    el.remove()
  })
})
