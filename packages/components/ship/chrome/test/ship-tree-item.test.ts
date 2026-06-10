// Behavioral spec for <ship-tree-item>, driven through the real compiler via
// renderComponent with the paired controller. Queried by what the operator
// perceives (visible text, reflected state, emitted events), never internal CSS.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives' // register composed sui-icon
import controller from '../src/ship-tree-item/ship-tree-item.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-tree-item/ship-tree-item.sui'),
  'utf8',
)
const TAG = 'ship-tree-item'

function mount(attrs: Record<string, string | boolean>) {
  return renderComponent(SRC, TAG, { controller, attrs })
}
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-tree-item>', () => {
  it('renders the row name and reflects file metadata', () => {
    const c = mount({ name: 'page.sui', kind: 'file', filetype: 'sui', status: 'dirty' })
    expect(root(c).textContent).toContain('page.sui')
    expect(c.el.getAttribute('kind')).toBe('file')
    expect(c.el.getAttribute('filetype')).toBe('sui')
    expect(c.el.getAttribute('status')).toBe('dirty')
    c.unmount()
  })

  it('a folder click toggles open and emits ship-toggle{open:true}', () => {
    const c = mount({ name: 'views', kind: 'folder' })
    let detail: unknown = null
    c.el.addEventListener('ship-toggle', (e) => { detail = (e as CustomEvent).detail })
    const rowEl = root(c).querySelector('[part="row"]') as HTMLElement
    rowEl.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ open: true })
    expect(c.el.hasAttribute('open')).toBe(true)
    c.unmount()
  })

  it('a file click emits ship-select (no toggle)', () => {
    const c = mount({ name: 'page.sui', kind: 'file' })
    let toggled = false
    let selected = false
    c.el.addEventListener('ship-toggle', () => { toggled = true })
    c.el.addEventListener('ship-select', () => { selected = true })
    const rowEl = root(c).querySelector('[part="row"]') as HTMLElement
    rowEl.dispatchEvent(new Event('click', { bubbles: true }))
    expect(selected).toBe(true)
    expect(toggled).toBe(false)
    c.unmount()
  })
})
