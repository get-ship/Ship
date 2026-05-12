import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-area-chart/dist/sui-area-chart.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-area-chart>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-area-chart')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-area-chart')
    expect(shadow(el)).not.toBeNull()
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sui-area-chart')
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label set from label prop', async () => {
    const { el } = mount(reg, 'sui-area-chart', { label: 'Revenue' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('Revenue')
  })

  it('default aria-label is "area chart"', async () => {
    const { el } = mount(reg, 'sui-area-chart')
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('area chart')
  })

  it('renders 2 paths per series (area fill + stroke line)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 3, 2, 4] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(2)
  })

  it('renders 4 paths for 2 series', async () => {
    const series = JSON.stringify([
      { name: 'a', data: [1, 2, 3] },
      { name: 'b', data: [3, 2, 1] },
    ])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(4)
  })

  it('first path (area fill) has fill-opacity 0.3', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    const firstPath = shadow(el).querySelector('path')
    expect(firstPath?.getAttribute('fill-opacity')).toBe('0.3')
  })

  it('renders circles when points attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series, points: true })
    await flush()
    expect(shadow(el).querySelectorAll('circle').length).toBe(3)
  })

  it('renders 3 grid lines when grid attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series, grid: true })
    await flush()
    expect(shadow(el).querySelectorAll('line.grid-line').length).toBe(3)
  })

  it('renders no paths for series with fewer than 2 data points', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('renders no paths for empty data', async () => {
    const { el } = mount(reg, 'sui-area-chart')
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('points reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-area-chart', { points: true })
    expect(el.hasAttribute('points')).toBe(true)
  })

  it('grid reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-area-chart', { grid: true })
    expect(el.hasAttribute('grid')).toBe(true)
  })

  it('series reflects to host attribute', () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    expect(el.getAttribute('series')).toBe(series)
  })

  it('slot for legend is present in shadow DOM', async () => {
    const { el } = mount(reg, 'sui-area-chart')
    await flush()
    expect(shadow(el).querySelector('slot')).not.toBeNull()
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-area-chart')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  it('reacts to series attribute change', async () => {
    const series1 = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const series2 = JSON.stringify([
      { name: 'a', data: [1, 2, 3] },
      { name: 'b', data: [3, 2, 1] },
    ])
    const { el } = mount(reg, 'sui-area-chart', { series: series1 })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(2)
    el.setAttribute('series', series2)
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(4)
  })
})
