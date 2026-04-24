import { afterEach, describe, expect, it } from 'vitest'
import './ship-area-chart'
import type { ShipAreaChart } from './ship-area-chart'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipAreaChart
  await el.updateComplete
  return el
}

describe('ship-area-chart', () => {
  afterEach(cleanupTestDom)

  it('renders 2 paths per series (area + stroke)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 3, 2, 4], tone: 'info' }])
    const el = await mount(`<ship-area-chart series='${series}'></ship-area-chart>`)
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(2)
  })

  it('applies 0.3 fill-opacity on area path', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const el = await mount(`<ship-area-chart series='${series}'></ship-area-chart>`)
    const first = el.shadowRoot!.querySelector('path')!
    expect(first.getAttribute('fill-opacity')).toBe('0.3')
  })

  it('reacts to series change', async () => {
    const el = await mount(
      `<ship-area-chart series='[{"name":"a","data":[1,2,3]}]'></ship-area-chart>`,
    )
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(2)
    el.setAttribute('series', '[{"name":"a","data":[1,2]},{"name":"b","data":[2,3]}]')
    await el.updateComplete
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(4)
  })
})
