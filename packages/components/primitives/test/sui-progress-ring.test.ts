import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-progress/sui-progress-ring.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const svg = (el: HTMLElement) => shadow(el).querySelector<SVGSVGElement>('[role="progressbar"]')
const trackCircle = (el: HTMLElement) => shadow(el).querySelector<SVGCircleElement>('.track')
const barCircle = (el: HTMLElement) => shadow(el).querySelector<SVGCircleElement>('.bar')
const ringWrapper = (el: HTMLElement) => shadow(el).querySelector<HTMLDivElement>('.ring')

describe('<sui-progress-ring>', () => {
  // ── Registration ──────────────────────────────────────────────────────────

  it('registers as a custom element', () => {
    expect(customElements.get('sui-progress-ring')).toBeDefined()
  })

  // ── Shadow DOM structure ──────────────────────────────────────────────────

  it('attaches a shadow root containing the SVG and circles', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(shadow(el)).not.toBeNull()
    expect(svg(el)).not.toBeNull()
    expect(trackCircle(el)).not.toBeNull()
    expect(barCircle(el)).not.toBeNull()
  })

  // ── A11y: role + aria ─────────────────────────────────────────────────────

  it('SVG has role=progressbar', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(svg(el)?.getAttribute('role')).toBe('progressbar')
  })

  it('aria-valuemin is always "0"', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(svg(el)?.getAttribute('aria-valuemin')).toBe('0')
  })

  it('aria-valuemax reflects the max prop', () => {
    const { el } = mount(reg, 'sui-progress-ring', { max: '200' })
    expect(svg(el)?.getAttribute('aria-valuemax')).toBe('200')
  })

  it('aria-valuenow defaults to "0" when value=0', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(svg(el)?.getAttribute('aria-valuenow')).toBe('0')
  })

  it('aria-valuenow reflects the value prop', () => {
    const { el } = mount(reg, 'sui-progress-ring', { value: '37' })
    expect(svg(el)?.getAttribute('aria-valuenow')).toBe('37')
  })

  it('aria-valuenow is absent when indeterminate is set', () => {
    const { el } = mount(reg, 'sui-progress-ring', { indeterminate: true })
    expect(svg(el)?.hasAttribute('aria-valuenow')).toBe(false)
  })

  it('aria-label is set when label prop is provided', () => {
    const { el } = mount(reg, 'sui-progress-ring', { label: 'Uploading' })
    expect(svg(el)?.getAttribute('aria-label')).toBe('Uploading')
  })

  it('aria-label is absent when label is empty (default)', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(svg(el)?.hasAttribute('aria-label')).toBe(false)
  })

  // ── Value clamping (0..max) ───────────────────────────────────────────────

  it('value above max is clamped to max in aria-valuenow', () => {
    const { el } = mount(reg, 'sui-progress-ring', { value: '250', max: '100' })
    expect(svg(el)?.getAttribute('aria-valuenow')).toBe('100')
  })

  it('negative value is clamped to 0 in aria-valuenow', () => {
    const { el } = mount(reg, 'sui-progress-ring', { value: '-20' })
    expect(svg(el)?.getAttribute('aria-valuenow')).toBe('0')
  })

  // ── Tone propagation ──────────────────────────────────────────────────────

  it('tone reflects to the host attribute', () => {
    const { el } = mount(reg, 'sui-progress-ring', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('tone updates reactively when attribute changes', async () => {
    const { el } = mount(reg, 'sui-progress-ring', { tone: 'info' })
    expect(el.getAttribute('tone')).toBe('info')
    el.setAttribute('tone', 'warning')
    await flush()
    expect(el.getAttribute('tone')).toBe('warning')
  })

  // ── Indeterminate ─────────────────────────────────────────────────────────

  it('indeterminate reflects to the host attribute', () => {
    const { el } = mount(reg, 'sui-progress-ring', { indeterminate: true })
    expect(el.hasAttribute('indeterminate')).toBe(true)
  })

  it('indeterminate attribute is absent by default', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(el.hasAttribute('indeterminate')).toBe(false)
  })

  // ── Size channel ──────────────────────────────────────────────────────────

  it('size prop drives the ring wrapper inline style', () => {
    const { el } = mount(reg, 'sui-progress-ring', { size: '64' })
    expect(ringWrapper(el)?.getAttribute('style')).toBe('width:64px;height:64px')
  })

  it('default size of 48 is applied to the wrapper', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(ringWrapper(el)?.getAttribute('style')).toBe('width:48px;height:48px')
  })

  // ── Parts ─────────────────────────────────────────────────────────────────

  it('part="track" is present on the background circle', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(shadow(el).querySelector('[part="track"]')).not.toBeNull()
  })

  it('part="ring" is present on the progress arc circle', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    expect(shadow(el).querySelector('[part="ring"]')).not.toBeNull()
  })

  // ── Reduced-motion ────────────────────────────────────────────────────────

  it('shadow styles include a prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-progress-ring')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  // ── Reactive updates ──────────────────────────────────────────────────────

  it('aria-valuenow updates reactively when value attribute changes', async () => {
    const { el } = mount(reg, 'sui-progress-ring', { value: '10' })
    expect(svg(el)?.getAttribute('aria-valuenow')).toBe('10')
    el.setAttribute('value', '75')
    await flush()
    expect(svg(el)?.getAttribute('aria-valuenow')).toBe('75')
  })

  it('dashoffset changes when value changes (bar updates)', async () => {
    const { el } = mount(reg, 'sui-progress-ring', { value: '0' })
    const before = barCircle(el)?.getAttribute('stroke-dashoffset')
    el.setAttribute('value', '100')
    await flush()
    const after = barCircle(el)?.getAttribute('stroke-dashoffset')
    expect(after).not.toEqual(before)
  })
})
