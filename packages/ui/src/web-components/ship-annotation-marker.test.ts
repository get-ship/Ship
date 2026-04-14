import { afterEach, describe, expect, it } from 'vitest'
import './ship-annotation-marker'
import type { ShipAnnotationMarker } from './ship-annotation-marker'
import { cleanupTestDom } from './util'

async function mount(markup: string): Promise<ShipAnnotationMarker> {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipAnnotationMarker
  await el.updateComplete
  return el
}

describe('ship-annotation-marker', () => {
  afterEach(cleanupTestDom)

  it('renders a point pin by default', async () => {
    const el = await mount('<ship-annotation-marker kind="point" index="0"></ship-annotation-marker>')
    const pin = el.shadowRoot!.querySelector('[part="pin"]') as HTMLElement
    expect(pin).toBeTruthy()
    expect(pin.textContent).toBe('1')
  })

  it('positions the point pin at x/y', async () => {
    const el = await mount('<ship-annotation-marker kind="point"></ship-annotation-marker>')
    el.x = 50
    el.y = 80
    await el.updateComplete
    const pin = el.shadowRoot!.querySelector('[part="pin"]') as HTMLElement
    expect(pin.style.left).toBe('50px')
    expect(pin.style.top).toBe('80px')
  })

  it('renders a region box when kind=region and rect is set', async () => {
    const el = await mount(
      '<ship-annotation-marker kind="region" rect="10,20,100,40"></ship-annotation-marker>',
    )
    const box = el.shadowRoot!.querySelector('[part="box"]') as HTMLElement
    expect(box).toBeTruthy()
    expect(box.style.left).toBe('10px')
    expect(box.style.width).toBe('100px')
  })

  it('renders nothing when region has no rect', async () => {
    const el = await mount('<ship-annotation-marker kind="region"></ship-annotation-marker>')
    expect(el.shadowRoot!.querySelector('[part="box"]')).toBeNull()
  })

  it('reflects active attribute on the host', async () => {
    const el = await mount('<ship-annotation-marker kind="point" active></ship-annotation-marker>')
    expect(el.hasAttribute('active')).toBe(true)
  })

  it('ignores malformed rect', async () => {
    const el = await mount(
      '<ship-annotation-marker kind="region" rect="bad,values"></ship-annotation-marker>',
    )
    expect(el.shadowRoot!.querySelector('[part="box"]')).toBeNull()
  })

  it('shows the index + 1 as the pin label', async () => {
    const el = await mount('<ship-annotation-marker kind="point" index="3"></ship-annotation-marker>')
    const pin = el.shadowRoot!.querySelector('[part="pin"]') as HTMLElement
    expect(pin.textContent).toBe('4')
  })
})
