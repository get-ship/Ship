import { afterEach, describe, expect, it } from 'vitest'
import './ship-tooltip'
import type { ShipTooltip } from './ship-tooltip'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipTooltip
  await el.updateComplete
  return el
}

describe('ship-tooltip', () => {
  afterEach(cleanupTestDom)

  it('renders bubble with role=tooltip', async () => {
    const el = await mount('<ship-tooltip>hi<span slot="content">info</span></ship-tooltip>')
    const bubble = el.shadowRoot!.querySelector('.bubble')!
    expect(bubble.getAttribute('role')).toBe('tooltip')
  })

  it('reflects placement', async () => {
    const el = await mount('<ship-tooltip placement="right">x</ship-tooltip>')
    expect(el.getAttribute('placement')).toBe('right')
  })
})
