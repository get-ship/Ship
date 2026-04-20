import { afterEach, describe, expect, it } from 'vitest'
import './ship-bar-chart'
import type { ShipBarChart } from './ship-bar-chart'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipBarChart
  await el.updateComplete
  return el
}

describe('ship-bar-chart', () => {
  afterEach(cleanupTestDom)

  it('renders a rect per datum', async () => {
    const el = await mount('<ship-bar-chart data="1,2,3,4"></ship-bar-chart>')
    expect(el.shadowRoot!.querySelectorAll('rect').length).toBe(4)
  })

  it('horizontal orientation flips layout', async () => {
    const el = await mount(
      '<ship-bar-chart data="1,2" orientation="horizontal" width="100" height="100"></ship-bar-chart>',
    )
    const rects = el.shadowRoot!.querySelectorAll('rect')
    // First horizontal bar should start at x=0
    expect(rects[0].getAttribute('x')).toBe('0')
  })

  it('reacts to data changes', async () => {
    const el = await mount('<ship-bar-chart data="1,2,3"></ship-bar-chart>')
    el.setAttribute('data', '1,2,3,4,5')
    await el.updateComplete
    expect(el.shadowRoot!.querySelectorAll('rect').length).toBe(5)
  })
})
