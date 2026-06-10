// Behavioral spec for <ship-activity-bar> — vertical context switcher.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives'
import controller from '../src/ship-activity-bar/ship-activity-bar.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-activity-bar/ship-activity-bar.sui'),
  'utf8',
)
const TAG = 'ship-activity-bar'

function mount(attrs: Record<string, string | boolean> = {}) {
  return renderComponent(SRC, TAG, { controller, attrs })
}
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-activity-bar>', () => {
  it('defaults active to views and reflects it', () => {
    const c = mount({ active: 'views' })
    expect(c.el.getAttribute('active')).toBe('views')
    c.unmount()
  })

  it('clicking a context sets active and emits ship-context-change{context}', () => {
    const c = mount({ active: 'views' })
    let detail: unknown = null
    c.el.addEventListener('ship-context-change', (e) => { detail = (e as CustomEvent).detail })
    const agents = root(c).querySelector('.a-agents') as HTMLElement
    agents.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ context: 'agents' })
    expect(c.el.getAttribute('active')).toBe('agents')
    c.unmount()
  })

  it('exposes the item part on each context for host styling', () => {
    const c = mount()
    expect(root(c).querySelectorAll('[part="item"]').length).toBe(8)
    c.unmount()
  })
})
