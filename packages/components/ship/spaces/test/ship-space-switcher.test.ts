// Behavioral spec for <ship-space-switcher> — the always-present SHELL picker.
// Data-driven (spaces JSON + active). The trigger shows the active label; opening
// the menu lists rows (active marked); choosing a non-active row emits
// ship-activate{id}; the register row emits ship-add. Composes <sui-icon>.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import controller from '../src/ship-space-switcher/ship-space-switcher.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-space-switcher/ship-space-switcher.sail'),
  'utf8',
)
const TAG = 'ship-space-switcher'

const SPACES = [
  { id: 's1', name: 'ship-core', path: '~/dev/ship' },
  { id: 's2', name: 'spaces-app', path: '~/dev/spaces' },
]

function mount(attrs: Record<string, string | boolean> = {}) {
  return renderComponent(SRC, TAG, {
    controller,
    attrs: { spaces: JSON.stringify(SPACES), ...attrs },
  })
}
function rowFor(c: { el: HTMLElement }, name: string): HTMLElement {
  const nm = Array.from(c.el.querySelectorAll('.row .r-name')).find(
    (s) => s.textContent?.trim() === name,
  )
  if (!nm) throw new Error(`no row for "${name}"`)
  return nm.closest('.row') as HTMLElement
}

describe('<ship-space-switcher>', () => {
  it('shows the active space name in the trigger label', () => {
    const c = mount({ active: 's2' })
    expect(c.el.querySelector('.t-label')?.textContent).toBe('spaces-app')
    expect(c.el.querySelector('.t-label')?.getAttribute('data-empty')).toBe('false')
    c.unmount()
  })

  it('falls back to the placeholder when no space is active', () => {
    const c = mount({ placeholder: 'Pick one' })
    const label = c.el.querySelector('.t-label')!
    expect(label.textContent).toBe('Pick one')
    expect(label.getAttribute('data-empty')).toBe('true')
    c.unmount()
  })

  it('lists a row per space and marks the active one when open', () => {
    const c = mount({ open: true, active: 's1' })
    expect(c.el.querySelectorAll('.menu .row').length).toBe(2)
    expect(rowFor(c, 'ship-core').classList.contains('active')).toBe(true)
    expect(rowFor(c, 'ship-core').getAttribute('aria-selected')).toBe('true')
    expect(rowFor(c, 'spaces-app').classList.contains('active')).toBe(false)
    c.unmount()
  })

  it('renders no menu while closed', () => {
    const c = mount({})
    expect(c.el.querySelector('.menu')).toBeNull()
    c.unmount()
  })

  it('emits ship-activate{id} when a non-active row is chosen', () => {
    const c = mount({ open: true, active: 's1' })
    let detail: unknown = null
    c.el.addEventListener('ship-activate', (e) => { detail = (e as CustomEvent).detail })
    rowFor(c, 'spaces-app').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ id: 's2' })
    c.unmount()
  })

  it('emits ship-add from the register row', () => {
    const c = mount({ open: true })
    let fired = false
    c.el.addEventListener('ship-add', () => { fired = true })
    c.el.querySelector('[part="add"]')!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })

  it('toggles open on the trigger', () => {
    const c = mount({})
    expect(c.el.hasAttribute('open')).toBe(false)
    c.el.querySelector('[part="trigger"]')!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(c.el.hasAttribute('open')).toBe(true)
    c.unmount()
  })
})
