import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-meter/sui-meter.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

const segs = (arr: Array<{ tone: string; count: number; label?: string }>) =>
  JSON.stringify(arr)

describe('<sui-meter>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-meter')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-meter')
    expect(shadow(el)).not.toBeNull()
  })

  it('empty segments renders no seg divs', async () => {
    const { el } = mount(reg, 'sui-meter')
    await flush()
    expect(shadow(el).querySelectorAll('.seg').length).toBe(0)
  })

  it('renders one seg div per positive-count segment', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([
        { tone: 'success', count: 3 },
        { tone: 'warning', count: 1 },
      ]),
    })
    await flush()
    expect(shadow(el).querySelectorAll('.seg').length).toBe(2)
  })

  it('zero-count segments are filtered out', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([
        { tone: 'success', count: 5 },
        { tone: 'info', count: 0 },
      ]),
    })
    await flush()
    expect(shadow(el).querySelectorAll('.seg').length).toBe(1)
  })

  it('bar element has role=img', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([{ tone: 'success', count: 3 }]),
    })
    await flush()
    expect(shadow(el).querySelector('.bar')?.getAttribute('role')).toBe('img')
  })

  it('aria-label from label prop', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([{ tone: 'info', count: 1 }]),
      label: 'Capacity',
    })
    await flush()
    expect(shadow(el).querySelector('.bar')?.getAttribute('aria-label')).toBe('Capacity')
  })

  it('default label is "meter"', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([{ tone: 'info', count: 1 }]),
    })
    await flush()
    expect(shadow(el).querySelector('.bar')?.getAttribute('aria-label')).toBe('meter')
  })

  it('segments reflects to host attribute', () => {
    const s = segs([{ tone: 'success', count: 3 }])
    const { el } = mount(reg, 'sui-meter', { segments: s })
    expect(el.getAttribute('segments')).toBe(s)
  })

  it('respects total prop as denominator', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([{ tone: 'info', count: 5 }]),
      total: '10',
    })
    await flush()
    const seg = shadow(el).querySelector('.seg') as HTMLElement
    expect(seg?.getAttribute('style') ?? '').toContain('width:50.000%')
  })

  it('seg div has title attribute', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([{ tone: 'success', count: 4, label: 'Active' }]),
    })
    await flush()
    const seg = shadow(el).querySelector('.seg')
    expect(seg?.getAttribute('title')).toBe('Active: 4')
  })

  it('reacts to segments attribute change', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([{ tone: 'info', count: 1 }]),
    })
    await flush()
    expect(shadow(el).querySelectorAll('.seg').length).toBe(1)
    el.setAttribute(
      'segments',
      segs([
        { tone: 'info', count: 1 },
        { tone: 'success', count: 2 },
        { tone: 'warning', count: 3 },
      ]),
    )
    await flush()
    expect(shadow(el).querySelectorAll('.seg').length).toBe(3)
  })

  it('seg without label uses tone in title', async () => {
    const { el } = mount(reg, 'sui-meter', {
      segments: segs([{ tone: 'warning', count: 7 }]),
    })
    await flush()
    const seg = shadow(el).querySelector('.seg')
    expect(seg?.getAttribute('title')).toBe('warning: 7')
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-meter')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })
})
