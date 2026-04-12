import { afterEach, describe, expect, it } from 'vitest'
import './ship-stat'
import type { ShipStat } from './ship-stat'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipStat
  await el.updateComplete
  return el
}

describe('ship-stat', () => {
  afterEach(cleanupTestDom)

  it('renders label + value + hint slots', async () => {
    const el = await mount(`
      <ship-stat accent="primary">
        <span slot="label">Active</span>
        42
        <span slot="hint">live</span>
      </ship-stat>
    `)
    expect(el.shadowRoot!.querySelectorAll('slot').length).toBe(3)
    expect(el.accent).toBe('primary')
  })
})
