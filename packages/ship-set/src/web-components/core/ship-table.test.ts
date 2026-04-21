import { afterEach, describe, expect, it } from 'vitest'
import './ship-table'
import type { ShipTable } from './ship-table'
import { cleanupTestDom } from './util'

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

  it('sticky-first-col attribute reflects', async () => {
    const el = await mount('<ship-table sticky-first-col></ship-table>')
    expect(el.stickyFirstCol).toBe(true)
    expect(el.hasAttribute('sticky-first-col')).toBe(true)
  })

  it('gutter attribute reflects', async () => {
    const el = await mount('<ship-table gutter></ship-table>')
    expect(el.gutter).toBe(true)
    expect(el.hasAttribute('gutter')).toBe(true)
  })

  it('ship-row-click fires when a data-row-id row td is clicked', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-table variant="native">
        <table>
          <thead><tr><th>Name</th></tr></thead>
          <tbody>
            <tr data-row-id="abc"><td>Alpha</td></tr>
          </tbody>
        </table>
      </ship-table>
    `)
    const el = document.body.lastElementChild as ShipTable
    await el.updateComplete
    let detail: { id: string; index: number } | null = null
    el.addEventListener('ship-row-click', (e) => { detail = (e as CustomEvent).detail })
    const td = el.querySelector('td') as HTMLTableCellElement
    td.click()
    expect(detail).not.toBeNull()
    expect(detail!.id).toBe('abc')
  })

  it('ship-row-click does not fire for gutter cells', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-table variant="native">
        <table>
          <tbody>
            <tr data-row-id="abc">
              <td class="ship-table-gutter-cell"><input type="checkbox"></td>
              <td>Name</td>
            </tr>
          </tbody>
        </table>
      </ship-table>
    `)
    const el = document.body.lastElementChild as ShipTable
    await el.updateComplete
    let fired = false
    el.addEventListener('ship-row-click', () => { fired = true })
    const gutterTd = el.querySelector('.ship-table-gutter-cell') as HTMLTableCellElement
    gutterTd.click()
    expect(fired).toBe(false)
  })

  it('ship-row-click does not fire for rows without data-row-id', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-table variant="native">
        <table><tbody><tr><td>No ID</td></tr></tbody></table>
      </ship-table>
    `)
    const el = document.body.lastElementChild as ShipTable
    await el.updateComplete
    let fired = false
    el.addEventListener('ship-row-click', () => { fired = true })
    ;(el.querySelector('td') as HTMLTableCellElement).click()
    expect(fired).toBe(false)
  })
})
