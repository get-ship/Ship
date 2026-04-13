import { afterEach, describe, expect, it } from 'vitest'
import './ship-status-dot'
import type { ShipStatusDot } from './ship-status-dot'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipStatusDot
  await el.updateComplete
  return el
}

describe('ship-status-dot', () => {
  afterEach(cleanupTestDom)

  it('renders a dot', async () => {
    const el = await mount('<ship-status-dot></ship-status-dot>')
    expect(el.shadowRoot!.querySelector('.dot')).toBeTruthy()
  })

  it('reflects pulse attribute', async () => {
    const el = await mount('<ship-status-dot pulse></ship-status-dot>')
    expect(el.pulse).toBe(true)
  })
})
