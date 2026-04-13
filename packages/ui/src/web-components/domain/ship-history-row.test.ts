import { afterEach, describe, expect, it } from 'vitest'
import './ship-history-row'
import type { ShipHistoryRow } from './ship-history-row'
import { cleanupTestDom } from '../util'

describe('ship-history-row', () => {
  afterEach(cleanupTestDom)

  it('renders 5 named slots for columns', async () => {
    document.body.insertAdjacentHTML('beforeend', '<ship-history-row><span slot="slug">a</span></ship-history-row>')
    const el = document.body.lastElementChild as ShipHistoryRow
    await el.updateComplete
    const names = Array.from(el.shadowRoot!.querySelectorAll('slot')).map(s => s.getAttribute('name'))
    expect(names).toEqual(['slug', 'pipeline', 'status', 'duration', 'completed'])
  })
})
