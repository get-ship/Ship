import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-harness-blob/sui-harness-blob.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-harness-blob>', () => {

  it('renders a tile + label slot in shadow DOM', async () => {
    const { el } = mount(reg, 'sui-harness-blob', { harness: 'claude' }, 'Claude')
    await flush()
    expect(shadow(el).querySelector('.tile')).not.toBeNull()
    expect(shadow(el).querySelector('.label slot')).not.toBeNull()
  })

  it('injects the brand SVG for the default (claude) harness', async () => {
    const { el } = mount(reg, 'sui-harness-blob')
    await flush()
    const svg = shadow(el).querySelector('.tile svg')
    expect(svg).not.toBeNull()
    expect(svg!.querySelector('title')?.textContent).toBe('Claude')
  })

  it('swaps the SVG when harness changes', async () => {
    const { el } = mount(reg, 'sui-harness-blob', { harness: 'gemini' })
    await flush()
    expect(shadow(el).querySelector('.tile svg title')?.textContent).toBe('Google Gemini')

    el.setAttribute('harness', 'cursor')
    await flush()
    expect(shadow(el).querySelector('.tile svg title')?.textContent).toBe('Cursor')
  })

  it('sets --blob-hue inline on the host per harness', async () => {
    const { el } = mount(reg, 'sui-harness-blob', { harness: 'claude' })
    await flush()
    expect(el.style.getPropertyValue('--blob-hue')).toBe('oklch(0.70 0.16 35)')
  })

  it('aria-label on the tile reflects the harness label', async () => {
    const { el } = mount(reg, 'sui-harness-blob', { harness: 'opencode' })
    await flush()
    expect(shadow(el).querySelector('.tile')?.getAttribute('aria-label')).toBe('OpenCode')
  })



  it('falls back to claude when given an unknown harness', async () => {
    const { el } = mount(reg, 'sui-harness-blob', { harness: 'mystery' })
    await flush()
    expect(shadow(el).querySelector('.tile svg title')?.textContent).toBe('Claude')
  })
})
