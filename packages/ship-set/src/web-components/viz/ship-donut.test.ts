import { afterEach, describe, expect, it } from 'vitest'
import './ship-donut'
import type { ShipDonut } from './ship-donut'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipDonut
  await el.updateComplete
  return el
}

describe('ship-donut', () => {
  afterEach(cleanupTestDom)

  it('renders one track + one circle per segment', async () => {
    const data = JSON.stringify([
      { label: 'a', value: 3, tone: 'info' },
      { label: 'b', value: 1, tone: 'success' },
    ])
    const el = await mount(`<ship-donut data='${data}'></ship-donut>`)
    // 1 track + 2 segments
    expect(el.shadowRoot!.querySelectorAll('circle').length).toBe(3)
  })

  it('renders slot for center content', async () => {
    const el = await mount(`<ship-donut data='[]'>42</ship-donut>`)
    expect(el.shadowRoot!.querySelector('.center')).toBeTruthy()
    expect(el.textContent?.trim()).toBe('42')
  })

  it('reacts to data changes', async () => {
    const el = await mount(`<ship-donut data='[{"label":"a","value":1}]'></ship-donut>`)
    expect(el.shadowRoot!.querySelectorAll('circle').length).toBe(2)
    el.setAttribute('data', '[{"label":"a","value":1},{"label":"b","value":2},{"label":"c","value":3}]')
    await el.updateComplete
    expect(el.shadowRoot!.querySelectorAll('circle').length).toBe(4)
  })
})
