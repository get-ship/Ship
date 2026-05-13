import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-donut/sui-donut.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

const seg = (label: string, value: number, tone?: string) =>
  JSON.stringify([{ label, value, tone }])

describe('<sui-donut>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-donut')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-donut')
    expect(shadow(el)).not.toBeNull()
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label set from label prop', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1), label: 'Sales mix' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('Sales mix')
  })

  it('default aria-label is "donut chart"', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-label')).toBe('donut chart')
  })

  it('track circle always rendered', async () => {
    const { el } = mount(reg, 'sui-donut')
    await flush()
    expect(shadow(el).querySelector('circle[part="track"]')).not.toBeNull()
  })

  it('empty data renders no segment paths', async () => {
    const { el } = mount(reg, 'sui-donut')
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('renders one path per positive-value segment', async () => {
    const data = JSON.stringify([
      { label: 'a', value: 3, tone: 'info' },
      { label: 'b', value: 1, tone: 'success' },
      { label: 'c', value: 2, tone: 'warning' },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(3)
  })

  it('zero-value segments are filtered out', async () => {
    const data = JSON.stringify([
      { label: 'a', value: 5 },
      { label: 'b', value: 0 },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(1)
  })

  it('single 100%-value segment renders one path', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('only', 10) })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(1)
  })

  it('each segment path has a title child', async () => {
    const data = JSON.stringify([
      { label: 'x', value: 2 },
      { label: 'y', value: 3 },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    const paths = shadow(el).querySelectorAll('path[part="segment"]')
    for (const p of paths) {
      expect(p.querySelector('title')).not.toBeNull()
    }
  })

  it('data reflects to host attribute', () => {
    const d = seg('a', 1)
    const { el } = mount(reg, 'sui-donut', { data: d })
    expect(el.getAttribute('data')).toBe(d)
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sui-donut', {
      data: JSON.stringify([{ label: 'a', value: 1 }]),
    })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(1)
    el.setAttribute(
      'data',
      JSON.stringify([
        { label: 'a', value: 1 },
        { label: 'b', value: 2 },
        { label: 'c', value: 3 },
      ]),
    )
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(3)
  })

  it('slot renders center content', async () => {
    const { el } = mount(reg, 'sui-donut', {}, '42')
    await flush()
    expect(shadow(el).querySelector('.center')).not.toBeNull()
    expect(el.textContent?.trim()).toBe('42')
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-donut')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  it('segment paths have part=segment attribute', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 5) })
    await flush()
    const path = shadow(el).querySelector('path')
    expect(path?.getAttribute('part')).toBe('segment')
  })
})
