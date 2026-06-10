// Behavioral spec for <ship-policy-matrix> — the capability × blast grid. Data-
// driven (rows JSON prop): each domain shows a grant at every tier ≤ its max and
// a lock beyond. Driven through the production compiler via renderComponent.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import controller from '../src/ship-policy-matrix/ship-policy-matrix.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-policy-matrix/ship-policy-matrix.sui'),
  'utf8',
)
const TAG = 'ship-policy-matrix'

const ROWS = [
  { domain: 'collections', granted: 'C2', note: 'granted · C2' },
  { domain: 'account', granted: '', note: 'operator-only' },
]

function mount(rows: unknown) {
  return renderComponent(SRC, TAG, { controller, attrs: { rows: JSON.stringify(rows) } })
}
function rowFor(c: { el: HTMLElement }, domain: string): HTMLElement {
  const cell = Array.from(c.el.querySelectorAll('td.dom')).find(
    (td) => td.textContent?.trim() === domain,
  )
  if (!cell) throw new Error(`no row for "${domain}"`)
  return cell.parentElement as HTMLElement
}

describe('<ship-policy-matrix>', () => {
  it('grants every tier ≤ the max and locks beyond it', () => {
    const c = mount(ROWS)
    const row = rowFor(c, 'collections')
    // C0,C1,C2 granted (3) · C3 locked (1)
    expect(row.querySelectorAll('.grant').length).toBe(3)
    expect(row.querySelectorAll('.lock').length).toBe(1)
    expect(row.textContent).toContain('granted · C2')
    c.unmount()
  })

  it('locks every tier for an operator-only (granted: "") domain', () => {
    const c = mount(ROWS)
    const row = rowFor(c, 'account')
    expect(row.querySelectorAll('.grant').length).toBe(0)
    expect(row.querySelectorAll('.lock').length).toBe(4)
    expect(row.textContent).toContain('operator-only')
    c.unmount()
  })

  it('renders no body rows for empty input', () => {
    const c = mount([])
    expect(c.el.querySelectorAll('tbody tr').length).toBe(0)
    c.unmount()
  })
})
