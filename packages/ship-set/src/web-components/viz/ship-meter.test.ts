import { afterEach, describe, expect, it } from 'vitest'
import './ship-meter'
import type { ShipMeter } from './ship-meter'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipMeter
  await el.updateComplete
  return el
}

describe('ship-meter', () => {
  afterEach(cleanupTestDom)

  it('renders one segment per entry', async () => {
    const el = await mount(
      `<ship-meter segments='[{"tone":"success","count":3},{"tone":"warning","count":1}]'></ship-meter>`,
    )
    expect(el.shadowRoot!.querySelectorAll('.seg').length).toBe(2)
  })

  it('respects total attr for denominator', async () => {
    const el = await mount(
      `<ship-meter total="10" segments='[{"tone":"info","count":5}]'></ship-meter>`,
    )
    const seg = el.shadowRoot!.querySelector('.seg') as HTMLElement
    expect(seg.getAttribute('style') ?? '').toContain('width: 50.000%')
  })

  it('reacts to segment changes', async () => {
    const el = await mount(
      `<ship-meter segments='[{"tone":"info","count":1}]'></ship-meter>`,
    )
    expect(el.shadowRoot!.querySelectorAll('.seg').length).toBe(1)
    el.setAttribute(
      'segments',
      '[{"tone":"info","count":1},{"tone":"success","count":2},{"tone":"warning","count":3}]',
    )
    await el.updateComplete
    expect(el.shadowRoot!.querySelectorAll('.seg').length).toBe(3)
  })
})
