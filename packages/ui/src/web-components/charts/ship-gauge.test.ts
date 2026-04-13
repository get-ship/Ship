import { afterEach, describe, expect, it } from 'vitest'
import './ship-gauge'
import type { ShipGauge } from './ship-gauge'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipGauge
  await el.updateComplete
  return el
}

describe('ship-gauge', () => {
  afterEach(cleanupTestDom)

  it('renders track + value arcs', async () => {
    const el = await mount('<ship-gauge value="40" max="100"></ship-gauge>')
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(2)
  })

  it('skips value arc when ratio is 0', async () => {
    const el = await mount('<ship-gauge value="0" max="100"></ship-gauge>')
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(1)
  })

  it('reacts to value changes', async () => {
    const el = await mount('<ship-gauge value="10" max="100"></ship-gauge>')
    const before = el.shadowRoot!.querySelectorAll('path')[1].getAttribute('d')
    el.value = 80
    await el.updateComplete
    const after = el.shadowRoot!.querySelectorAll('path')[1].getAttribute('d')
    expect(after).not.toEqual(before)
  })
})
