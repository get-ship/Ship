import { afterEach, describe, expect, it } from 'vitest'
import './ship-line-chart'
import type { ShipLineChart } from './ship-line-chart'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipLineChart
  await el.updateComplete
  return el
}

describe('ship-line-chart', () => {
  afterEach(cleanupTestDom)

  it('parses series JSON + renders a path per series', async () => {
    const series = JSON.stringify([
      { name: 'a', data: [1, 2, 3, 4], tone: 'info' },
      { name: 'b', data: [4, 3, 2, 1], tone: 'success' },
    ])
    const el = await mount(`<ship-line-chart series='${series}'></ship-line-chart>`)
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(2)
  })

  it('renders points when points attr set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const el = await mount(`<ship-line-chart series='${series}' points></ship-line-chart>`)
    expect(el.shadowRoot!.querySelectorAll('circle').length).toBe(3)
  })

  it('renders grid lines when grid attr set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const el = await mount(`<ship-line-chart series='${series}' grid></ship-line-chart>`)
    expect(el.shadowRoot!.querySelectorAll('line.grid-line').length).toBe(3)
  })

  it('tolerates bad JSON (renders empty svg)', async () => {
    const el = await mount(`<ship-line-chart series="not-json"></ship-line-chart>`)
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(0)
  })
})
