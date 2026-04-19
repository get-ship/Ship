import { afterEach, describe, expect, it } from 'vitest'
import './ship-slider'
import type { ShipSlider } from './ship-slider'
import { cleanupTestDom } from './util'

async function mount(attrs = '') {
  document.body.insertAdjacentHTML('beforeend', `
    <ship-slider ${attrs}></ship-slider>
  `)
  const el = document.body.lastElementChild as ShipSlider
  await el.updateComplete
  return el
}

describe('ship-slider', () => {
  afterEach(cleanupTestDom)

  it('renders with default values', async () => {
    const el = await mount()
    expect(el.value).toBe(0)
    expect(el.min).toBe(0)
    expect(el.max).toBe(100)
    expect(el.step).toBe(1)
    const thumb = el.shadowRoot!.querySelector('[role="slider"]')!
    expect(thumb.getAttribute('aria-valuenow')).toBe('0')
    expect(thumb.getAttribute('aria-valuemin')).toBe('0')
    expect(thumb.getAttribute('aria-valuemax')).toBe('100')
  })

  it('reflects value attribute', async () => {
    const el = await mount('value="50" min="0" max="100"')
    expect(el.value).toBe(50)
    const fill = el.shadowRoot!.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('50%')
  })

  it('keyboard ArrowRight increments value and emits ship-change', async () => {
    const el = await mount('value="10" step="5"')
    let detail: { value: number } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })

    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await el.updateComplete

    expect(el.value).toBe(15)
    expect(detail).toEqual({ value: 15 })
  })

  it('clamps value to min/max', async () => {
    const el = await mount('value="98" min="0" max="100" step="5"')
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await el.updateComplete
    expect(el.value).toBe(100)
  })

  it('Home and End set min/max respectively', async () => {
    const el = await mount('value="50" min="0" max="100"')
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement

    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }))
    await el.updateComplete
    expect(el.value).toBe(0)

    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    await el.updateComplete
    expect(el.value).toBe(100)
  })

  it('disabled slider does not respond to keyboard', async () => {
    const el = await mount('value="50" disabled')
    const thumb = el.shadowRoot!.querySelector('[role="slider"]') as HTMLElement
    thumb.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await el.updateComplete
    expect(el.value).toBe(50)
  })
})
