import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-heatmap/sui-heatmap.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-heatmap>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-heatmap')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-heatmap')
    expect(shadow(el)).not.toBeNull()
  })

  it('empty data renders no rect elements', async () => {
    const { el } = mount(reg, 'sui-heatmap', { rows: 'a,b', cols: 'x,y' })
    await flush()
    expect(shadow(el).querySelectorAll('rect').length).toBe(0)
  })

  it('1D data renders rows × cols rects', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a,b',
      cols: 'x,y,z',
      data: '0.1,0.5,0.9,0.2,0.6,1.0',
    })
    await flush()
    expect(shadow(el).querySelectorAll('rect').length).toBe(6)
  })

  it('2D JSON data renders rows × cols rects', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a,b',
      cols: 'x,y',
      data: '[[0.5,0.8],[0.2,0.9]]',
    })
    await flush()
    expect(shadow(el).querySelectorAll('rect').length).toBe(4)
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.5',
    })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label from label prop', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.5',
      label: 'Activity grid',
    })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('Activity grid')
  })

  it('default label is "heatmap"', async () => {
    const { el } = mount(reg, 'sui-heatmap', { rows: 'a', cols: 'x', data: '0.5' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('heatmap')
  })

  it('tone reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-heatmap', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('data reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-heatmap', { data: '0.1,0.2' })
    expect(el.getAttribute('data')).toBe('0.1,0.2')
  })

  it('rect elements have part=cell', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.5',
    })
    await flush()
    const rect = shadow(el).querySelector('rect')
    expect(rect?.getAttribute('part')).toBe('cell')
  })

  it('non-zero value cells have color-mix fill style', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.8',
    })
    await flush()
    const rect = shadow(el).querySelector('rect')
    expect(rect?.getAttribute('style') ?? '').toContain('color-mix')
  })

  it('zero-value cell has no fill style', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a,b',
      cols: 'x',
      data: '0,1',
    })
    await flush()
    const rects = shadow(el).querySelectorAll('rect')
    // First row value is 0 (min), last is 1 (max). Min-valued cell gets no fill override.
    expect(rects[0]?.getAttribute('style') ?? '').toBe('')
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a',
      cols: 'x,y',
      data: '0,0',
    })
    await flush()
    expect(shadow(el).querySelectorAll('rect').length).toBe(2)
    el.setAttribute('data', '0.5,0.8')
    await flush()
    const rects = shadow(el).querySelectorAll('rect')
    expect(rects.length).toBe(2)
    // Now both have non-zero values relative to min/max.
    const hasColorMix = Array.from(rects).some((r) =>
      (r.getAttribute('style') ?? '').includes('color-mix'),
    )
    expect(hasColorMix).toBe(true)
  })

  it('viewBox reflects grid dimensions', async () => {
    const { el } = mount(reg, 'sui-heatmap', {
      rows: 'a,b',
      cols: 'x,y,z',
      data: '[[1,2,3],[4,5,6]]',
      'cell-size': '20',
      gap: '3',
    })
    await flush()
    // width = 3*20 + 2*3 = 66, height = 2*20 + 1*3 = 43
    const vb = shadow(el).querySelector('svg')?.getAttribute('viewBox')
    expect(vb).toBe('0 0 66 43')
  })
})
