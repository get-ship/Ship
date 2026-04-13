import { afterEach, describe, expect, it } from 'vitest'
import './ship-table'
import type { ShipTable } from './ship-table'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipTable
  await el.updateComplete
  return el
}

describe('ship-table', () => {
  afterEach(cleanupTestDom)

  it('renders thead/tbody slots', async () => {
    const el = document.createElement('ship-table') as ShipTable
    const headRow = document.createElement('tr')
    headRow.slot = 'head'
    const th = document.createElement('th')
    th.textContent = 'Name'
    headRow.appendChild(th)
    const bodyRow = document.createElement('tr')
    const td = document.createElement('td')
    td.textContent = 'Alice'
    bodyRow.appendChild(td)
    el.append(headRow, bodyRow)
    document.body.appendChild(el)
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('thead')).toBeTruthy()
    expect(el.shadowRoot!.querySelector('tbody')).toBeTruthy()
    const headSlot = el.shadowRoot!.querySelector('slot[name="head"]') as HTMLSlotElement
    const assigned = headSlot.assignedElements()
    expect(assigned[0].querySelector('th')?.textContent).toBe('Name')
  })

  it('reflects dense attribute', async () => {
    const el = await mount('<ship-table dense></ship-table>')
    expect(el.dense).toBe(true)
    expect(el.hasAttribute('dense')).toBe(true)
  })

  it('defaults hoverable=true', async () => {
    const el = await mount('<ship-table></ship-table>')
    expect(el.hoverable).toBe(true)
  })
})
