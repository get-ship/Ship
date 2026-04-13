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

  it('renders label + value + trend + hint slots', async () => {
    const el = await mount(`
      <ship-stat accent="primary">
        <span slot="label">Active</span>
        42
        <span slot="trend">+12%</span>
        <span slot="hint">live</span>
      </ship-stat>
    `)
    const slotNames = Array.from(el.shadowRoot!.querySelectorAll('slot')).map(
      (s) => s.getAttribute('name') ?? '',
    )
    expect(slotNames).toEqual(expect.arrayContaining(['label', 'trend', 'hint', '']))
    expect(el.accent).toBe('primary')
  })
})
