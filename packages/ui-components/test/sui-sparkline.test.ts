import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-sparkline/dist/sui-sparkline.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-sparkline>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-sparkline')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-sparkline')
    expect(shadow(el)).not.toBeNull()
  })

  it('renders a stroke path for valid data', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3,4,5' })
    await flush()
    const paths = shadow(el).querySelectorAll('path')
    expect(paths.length).toBeGreaterThanOrEqual(1)
    expect(paths[0]?.getAttribute('d')).toMatch(/^M/)
  })

  it('SVG has role=img when data is valid', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label set from label prop', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3', label: 'Trend' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('Trend')
  })

  it('default aria-label is "sparkline"', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('sparkline')
  })

  it('renders 2 paths when fill attr is set', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3,4', fill: true })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(2)
  })

  it('area fill path has fill-opacity 0.18', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3,4', fill: true })
    await flush()
    const first = shadow(el).querySelector('path')
    expect(first?.getAttribute('fill-opacity')).toBe('0.18')
  })

  it('renders no paths for single data point', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '5' })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('renders no paths for empty data', async () => {
    const { el } = mount(reg, 'sui-sparkline')
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('empty SVG is rendered (no crash) for empty data', async () => {
    const { el } = mount(reg, 'sui-sparkline')
    await flush()
    expect(shadow(el).querySelector('svg')).not.toBeNull()
  })

  it('empty SVG has aria-hidden for empty data', async () => {
    const { el } = mount(reg, 'sui-sparkline')
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-hidden')).toBe('true')
  })

  it('tone reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-sparkline', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('fill reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-sparkline', { fill: true })
    expect(el.hasAttribute('fill')).toBe(true)
  })

  it('data reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    expect(el.getAttribute('data')).toBe('1,2,3')
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-sparkline')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    await flush()
    const before = shadow(el).querySelector('path')?.getAttribute('d')
    el.setAttribute('data', '5,4,3,2,1')
    await flush()
    const after = shadow(el).querySelector('path')?.getAttribute('d')
    expect(after).not.toEqual(before)
  })
})
