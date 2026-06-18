// Behavioral spec for <ship-file-tree> — titled navigator panel.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives'
import controller from '../src/ship-file-tree/ship-file-tree.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-file-tree/ship-file-tree.sui'),
  'utf8',
)
const TAG = 'ship-file-tree'

function mount(attrs: Record<string, string | boolean>) {
  return renderComponent(SRC, TAG, { controller, attrs })
}
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-file-tree>', () => {
  it('renders the context label and scope (not a generic "Explorer")', () => {
    const c = mount({ label: 'Views', scope: '.ship/' })
    const text = root(c).textContent ?? ''
    expect(text).toContain('Views')
    expect(text).toContain('.ship/')
    expect(text).not.toContain('Explorer')
    c.unmount()
  })

  it('emits ship-new when the New header action is clicked', () => {
    const c = mount({ label: 'Views' })
    let fired = false
    c.el.addEventListener('ship-new', () => { fired = true })
    const btn = root(c).querySelector('button[title="New"]') as HTMLElement
    btn.dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })

  it('emits ship-collapse-all when the Collapse all action is clicked', () => {
    const c = mount({ label: 'Views' })
    let fired = false
    c.el.addEventListener('ship-collapse-all', () => { fired = true })
    const btn = root(c).querySelector('button[title="Collapse all"]') as HTMLElement
    btn.dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })
})
