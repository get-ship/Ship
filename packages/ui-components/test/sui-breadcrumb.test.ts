import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-breadcrumb/sui-breadcrumb.compiled.js'
import '../src/sui-breadcrumb/sui-breadcrumb-item.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

function mountBreadcrumb(innerHTML: string) {
  const container = reg.mk()
  container.innerHTML = `<sui-breadcrumb>${innerHTML}</sui-breadcrumb>`
  return container.querySelector('sui-breadcrumb') as HTMLElement
}

// ── <sui-breadcrumb> ──────────────────────────────────────────────────────────

describe('<sui-breadcrumb>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-breadcrumb')).toBeDefined()
  })

  it('attaches a shadow root with a <nav> element', () => {
    const { el } = mount(reg, 'sui-breadcrumb')
    expect(shadow(el).querySelector('nav')).not.toBeNull()
    expect(shadow(el).querySelector('nav')!.getAttribute('aria-label')).toBe('Breadcrumb')
    expect(shadow(el).querySelector('nav')!.getAttribute('part')).toBe('nav')
  })

  it('renders N-1 separators for N items', async () => {
    const el = mountBreadcrumb(`
      <sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>
      <sui-breadcrumb-item href="/a">A</sui-breadcrumb-item>
      <sui-breadcrumb-item active>B</sui-breadcrumb-item>
    `)
    await flush()
    const seps = shadow(el).querySelectorAll('.sep')
    expect(seps.length).toBe(2)
  })

  it('renders zero separators for a single item', async () => {
    const el = mountBreadcrumb('<sui-breadcrumb-item>Only</sui-breadcrumb-item>')
    await flush()
    expect(shadow(el).querySelectorAll('.sep').length).toBe(0)
  })

  it('renders no separators when empty', () => {
    const { el } = mount(reg, 'sui-breadcrumb')
    expect(shadow(el).querySelectorAll('.sep').length).toBe(0)
  })

  it('default separator is a chevron SVG', async () => {
    const el = mountBreadcrumb(`
      <sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>
      <sui-breadcrumb-item>Current</sui-breadcrumb-item>
    `)
    await flush()
    const sep = shadow(el).querySelector('.sep')!
    expect(sep.querySelector('svg')).not.toBeNull()
    expect(sep.getAttribute('aria-hidden')).toBe('true')
  })

  it('custom separator uses text content', async () => {
    const el = mountBreadcrumb(`
      <sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>
      <sui-breadcrumb-item>Page</sui-breadcrumb-item>
    `)
    el.setAttribute('separator', '>')
    await flush()
    const sep = shadow(el).querySelector('.sep')!
    expect(sep.textContent).toBe('>')
    expect(sep.querySelector('svg')).toBeNull()
  })

  it('separator prop reflects', () => {
    const { el } = mount(reg, 'sui-breadcrumb', { separator: '|' })
    expect(el.getAttribute('separator')).toBe('|')
  })

  it('assigns slot names to children so slots resolve', async () => {
    const el = mountBreadcrumb(`
      <sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>
      <sui-breadcrumb-item>Page</sui-breadcrumb-item>
    `)
    await flush()
    const items = el.querySelectorAll('sui-breadcrumb-item')
    expect(items[0].getAttribute('slot')).toBe('i0')
    expect(items[1].getAttribute('slot')).toBe('i1')
  })

  it('adds a new separator when a child is dynamically appended', async () => {
    const el = mountBreadcrumb('<sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>')
    await flush()
    expect(shadow(el).querySelectorAll('.sep').length).toBe(0)

    const item2 = document.createElement('sui-breadcrumb-item')
    item2.textContent = 'New'
    el.appendChild(item2)
    await flush()
    expect(shadow(el).querySelectorAll('.sep').length).toBe(1)
  })
})

// ── <sui-breadcrumb-item> ─────────────────────────────────────────────────────

describe('<sui-breadcrumb-item>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-breadcrumb-item')).toBeDefined()
  })

  it('renders a <span> by default (no href)', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', {}, 'Label')
    const span = shadow(el).querySelector('.label')
    expect(span).not.toBeNull()
    expect(shadow(el).querySelector('a')).toBeNull()
  })

  it('renders an <a> when href is provided and active is not set', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', { href: '/page' }, 'Page')
    const link = shadow(el).querySelector('a')
    expect(link).not.toBeNull()
    expect(link!.getAttribute('href')).toBe('/page')
    expect(link!.getAttribute('part')).toBe('link')
  })

  it('renders a <span> (not <a>) when active is set, even with href', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', { href: '/page', active: true }, 'Page')
    expect(shadow(el).querySelector('a')).toBeNull()
    const span = shadow(el).querySelector('.label')
    expect(span).not.toBeNull()
  })

  it('active item span has aria-current="page"', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', { active: true }, 'Current')
    const span = shadow(el).querySelector('.label')!
    expect(span.getAttribute('aria-current')).toBe('page')
  })

  it('inactive item span has aria-current="false"', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', {}, 'Item')
    const span = shadow(el).querySelector('.label')!
    expect(span.getAttribute('aria-current')).toBe('false')
  })

  it('link has part="link"', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', { href: '/x' }, 'X')
    expect(shadow(el).querySelector('a')!.getAttribute('part')).toBe('link')
  })

  it('span has part="label"', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', {}, 'X')
    expect(shadow(el).querySelector('.label')!.getAttribute('part')).toBe('label')
  })

  it('href reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', { href: '/about' })
    expect(el.getAttribute('href')).toBe('/about')
  })

  it('active reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', { active: true })
    expect(el.hasAttribute('active')).toBe(true)
  })

  it('toggling active updates the rendered element type', async () => {
    const { el } = mount(reg, 'sui-breadcrumb-item', { href: '/page' }, 'Page')
    expect(shadow(el).querySelector('a')).not.toBeNull()
    el.setAttribute('active', '')
    await flush()
    expect(shadow(el).querySelector('a')).toBeNull()
    expect(shadow(el).querySelector('.label')).not.toBeNull()
  })
})

// ── parent + item integration ─────────────────────────────────────────────────

describe('<sui-breadcrumb> + <sui-breadcrumb-item> integration', () => {
  it('separator is hidden from AT (aria-hidden)', async () => {
    const el = mountBreadcrumb(`
      <sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>
      <sui-breadcrumb-item active>Page</sui-breadcrumb-item>
    `)
    await flush()
    const seps = shadow(el).querySelectorAll('.sep')
    for (const sep of seps) {
      expect(sep.getAttribute('aria-hidden')).toBe('true')
    }
  })

  it('aria-current is set on the current item', async () => {
    const el = mountBreadcrumb(`
      <sui-breadcrumb-item href="/">Home</sui-breadcrumb-item>
      <sui-breadcrumb-item href="/a">A</sui-breadcrumb-item>
      <sui-breadcrumb-item active>Active</sui-breadcrumb-item>
    `)
    await flush()
    const items = el.querySelectorAll('sui-breadcrumb-item')
    const last = items[items.length - 1] as HTMLElement
    const span = last.shadowRoot!.querySelector('.label')!
    expect(span.getAttribute('aria-current')).toBe('page')
  })
})
