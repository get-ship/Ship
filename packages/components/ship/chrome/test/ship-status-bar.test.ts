// Behavioral spec for <ship-status-bar> — glance-only ambient state strip.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives'
import controller from '../src/ship-status-bar/ship-status-bar.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-status-bar/ship-status-bar.sui'),
  'utf8',
)
const TAG = 'ship-status-bar'

function mount(attrs: Record<string, string | boolean>) {
  return renderComponent(SRC, TAG, { controller, attrs })
}
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-status-bar>', () => {
  it('reflects daemon/sync and renders the live counts + space', () => {
    const c = mount({ daemon: 'busy', sync: 'syncing', sessions: '3', approvals: '2', space: 'Ship' })
    expect(c.el.getAttribute('daemon')).toBe('busy')
    expect(c.el.getAttribute('sync')).toBe('syncing')
    expect(c.el.getAttribute('approvals')).toBe('2')
    const text = root(c).textContent ?? ''
    expect(text).toContain('3')
    expect(text).toContain('2')
    expect(text).toContain('Ship')
    c.unmount()
  })

  it('reflects approvals=0 so the host CSS can hide the gate segment', () => {
    const c = mount({ approvals: '0' })
    expect(c.el.getAttribute('approvals')).toBe('0')
    c.unmount()
  })

  it('emits ship-open-runs{source:sessions} when the sessions segment is clicked', () => {
    const c = mount({ sessions: '2' })
    let detail: unknown = null
    c.el.addEventListener('ship-open-runs', (e) => { detail = (e as CustomEvent).detail })
    const seg = root(c).querySelector('.sessions') as HTMLElement
    seg.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ source: 'sessions' })
    c.unmount()
  })

  it('emits ship-open-runs{source:approvals} when the approvals segment is clicked', () => {
    const c = mount({ approvals: '1' })
    let detail: unknown = null
    c.el.addEventListener('ship-open-runs', (e) => { detail = (e as CustomEvent).detail })
    const seg = root(c).querySelector('.approvals') as HTMLElement
    seg.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ source: 'approvals' })
    c.unmount()
  })
})
