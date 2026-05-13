import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-provider-blob/sui-provider-blob.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-provider-blob>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-provider-blob')).toBeDefined()
  })

  it('renders a tile + label slot in shadow DOM', async () => {
    const { el } = mount(reg, 'sui-provider-blob', { provider: 'claude' }, 'Claude')
    await flush()
    expect(shadow(el).querySelector('.tile')).not.toBeNull()
    expect(shadow(el).querySelector('.label slot')).not.toBeNull()
  })

  it('injects the brand SVG for the default (claude) provider', async () => {
    const { el } = mount(reg, 'sui-provider-blob')
    await flush()
    const svg = shadow(el).querySelector('.tile svg')
    expect(svg).not.toBeNull()
    expect(svg!.querySelector('title')?.textContent).toBe('Claude')
  })

  it('swaps the SVG when provider changes', async () => {
    const { el } = mount(reg, 'sui-provider-blob', { provider: 'gemini' })
    await flush()
    expect(shadow(el).querySelector('.tile svg title')?.textContent).toBe('Google Gemini')

    el.setAttribute('provider', 'cursor')
    await flush()
    expect(shadow(el).querySelector('.tile svg title')?.textContent).toBe('Cursor')
  })

  it('sets --blob-hue inline on the host per provider', async () => {
    const { el } = mount(reg, 'sui-provider-blob', { provider: 'claude' })
    await flush()
    expect(el.style.getPropertyValue('--blob-hue')).toBe('oklch(0.70 0.16 35)')
  })

  it('aria-label on the tile reflects the provider label', async () => {
    const { el } = mount(reg, 'sui-provider-blob', { provider: 'opencode' })
    await flush()
    expect(shadow(el).querySelector('.tile')?.getAttribute('aria-label')).toBe('OpenCode')
  })

  it('reflects size attribute', () => {
    const { el } = mount(reg, 'sui-provider-blob', { size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('reflects variant attribute', () => {
    const { el } = mount(reg, 'sui-provider-blob', { variant: 'glyph' })
    expect(el.getAttribute('variant')).toBe('glyph')
  })

  it('falls back to claude when given an unknown provider', async () => {
    const { el } = mount(reg, 'sui-provider-blob', { provider: 'mystery' })
    await flush()
    expect(shadow(el).querySelector('.tile svg title')?.textContent).toBe('Claude')
  })
})
