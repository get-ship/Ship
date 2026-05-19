import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-slider/sui-slider.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-slider>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-slider')).toBeDefined()
  })

  it('renders with default values', async () => {
    const { el } = mount(reg, 'sui-slider')
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]')!
    expect(thumb.getAttribute('aria-valuenow')).toBe('0')
    expect(thumb.getAttribute('aria-valuemin')).toBe('0')
    expect(thumb.getAttribute('aria-valuemax')).toBe('100')
  })

  it('reflects value attribute and positions fill', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '50', min: '0', max: '100' })
    await flush()
    const fill = el.shadowRoot!.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('50%')
    const thumb = el.shadowRoot!.querySelector('.thumb') as HTMLElement
    expect(thumb.style.left).toBe('50%')
  })

  it('thumb aria-label uses label prop', async () => {
    const { el } = mount(reg, 'sui-slider', { label: 'Volume' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]')!
    expect(thumb.getAttribute('aria-label')).toBe('Volume')
  })

  it('ArrowRight increments value by step and emits sui-change', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '10', step: '5' })
    await flush()
    let detail: { value: number } | null = null
    el.addEventListener('sui-change', (e) => { detail = (e as CustomEvent).detail })
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('15')
    expect(detail).toEqual({ value: 15 })
  })

  it('ArrowLeft decrements value', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '10', step: '5' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('5')
  })

  it('ArrowUp increments value', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '10', step: '5' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('15')
  })

  it('ArrowDown decrements value', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '10', step: '5' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('5')
  })

  it('Home sets value to min', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '50', min: '0', max: '100' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('0')
  })

  it('End sets value to max', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '50', min: '0', max: '100' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('100')
  })

  it('PageUp increments by 10 steps', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '10', step: '1', max: '100' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'PageUp', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('20')
  })

  it('PageDown decrements by 10 steps', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '30', step: '1', min: '0' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'PageDown', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('20')
  })

  it('clamps value to max', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '98', min: '0', max: '100', step: '5' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await flush()
    expect(Number(el.getAttribute('value'))).toBeLessThanOrEqual(100)
  })

  it('clamps value to min', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '2', min: '0', max: '100', step: '5' })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))
    await flush()
    expect(Number(el.getAttribute('value'))).toBeGreaterThanOrEqual(0)
  })

  it('disabled slider does not respond to keyboard', async () => {
    const { el } = mount(reg, 'sui-slider', { value: '50', disabled: true })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('50')
  })

  it('thumb has tabindex=-1 when disabled', async () => {
    const { el } = mount(reg, 'sui-slider', { disabled: true })
    await flush()
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    expect(thumb.getAttribute('tabindex')).toBe('-1')
  })

  it('tone reflects on host', () => {
    const { el } = mount(reg, 'sui-slider', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('size reflects on host', () => {
    const { el } = mount(reg, 'sui-slider', { size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('parts: track, fill, thumb present', () => {
    const { el } = mount(reg, 'sui-slider')
    expect(el.shadowRoot!.querySelector('[part~="track"]')).not.toBeNull()
    expect(el.shadowRoot!.querySelector('[part~="fill"]')).not.toBeNull()
    expect(el.shadowRoot!.querySelector('[part~="thumb"]')).not.toBeNull()
  })

  it('form participation: FormData contains name=value', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form>
        <sui-slider name="volume" value="75"></sui-slider>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('volume')).toBe('75')
  })

  it('form participation: FormData updates after keyboard interaction', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form>
        <sui-slider name="vol" value="50" step="10"></sui-slider>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    const slider = container.querySelector('sui-slider') as HTMLElement
    await flush()
    const thumb = slider.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await flush()
    const data = new FormData(form)
    expect(data.get('vol')).toBe('60')
  })
})
