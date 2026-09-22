// Behavioral spec for <ship-space-registry> — the SHELL registry list. Data-driven
// (spaces JSON prop + active id), marks the active row, and emits ship-activate{id}
// on a non-active row press / ship-add when "Register a space" is pressed.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import controller from '../src/ship-space-registry/ship-space-registry.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-space-registry/ship-space-registry.sail'),
  'utf8',
)
const TAG = 'ship-space-registry'

const NOW = Date.now()
const SPACES = [
  { id: 's1', name: 'ship-core', path: '~/dev/ship', last_seen_at: new Date(NOW).toISOString() },
  {
    id: 's2',
    name: 'spaces-app',
    path: '~/dev/spaces',
    last_seen_at: new Date(NOW - 3 * 3600_000).toISOString(),
  },
]

function mount(spaces: unknown, attrs: Record<string, string> = {}) {
  return renderComponent(SRC, TAG, {
    controller,
    attrs: { spaces: JSON.stringify(spaces), ...attrs },
  })
}
function rowFor(c: { el: HTMLElement }, name: string): HTMLElement {
  const nm = Array.from(c.el.querySelectorAll('.row .name')).find(
    (s) => s.textContent?.trim() === name,
  )
  if (!nm) throw new Error(`no row for "${name}"`)
  return nm.closest('.row') as HTMLElement
}

describe('<ship-space-registry>', () => {
  it('renders a row per space with name, path and relative last-seen', () => {
    const c = mount(SPACES)
    expect(c.el.querySelectorAll('.row').length).toBe(2)
    const row = rowFor(c, 'spaces-app')
    expect(row.querySelector('.path')?.textContent).toBe('~/dev/spaces')
    expect(row.querySelector('.seen')?.textContent).toBe('3h')
    c.unmount()
  })

  it('marks the active row (aria-current + active state) from the active prop', () => {
    const c = mount(SPACES, { active: 's2' })
    const active = rowFor(c, 'spaces-app')
    const inactive = rowFor(c, 'ship-core')
    expect(active.classList.contains('active')).toBe(true)
    expect(active.getAttribute('aria-current')).toBe('true')
    expect(active.querySelector('.state')?.textContent).toBe('active')
    expect(inactive.classList.contains('active')).toBe(false)
    expect(inactive.querySelector('.state')?.textContent).toBe('open')
    c.unmount()
  })

  it('emits ship-activate{id} when a non-active row is pressed', () => {
    const c = mount(SPACES, { active: 's1' })
    let detail: unknown = null
    c.el.addEventListener('ship-activate', (e) => { detail = (e as CustomEvent).detail })
    rowFor(c, 'spaces-app').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ id: 's2' })
    c.unmount()
  })

  it('does not emit ship-activate when the active row is pressed', () => {
    const c = mount(SPACES, { active: 's1' })
    let fired = false
    c.el.addEventListener('ship-activate', () => { fired = true })
    rowFor(c, 'ship-core').dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(false)
    c.unmount()
  })

  it('emits ship-add when "Register a space" is pressed', () => {
    const c = mount(SPACES)
    let fired = false
    c.el.addEventListener('ship-add', () => { fired = true })
    c.el.querySelector('[part="add"]')!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })

  it('renders the empty state (with its own add affordance) for no spaces', () => {
    const c = mount([])
    expect(c.el.querySelectorAll('.row').length).toBe(0)
    expect(c.el.querySelector('[part="empty"]')).not.toBeNull()
    let fired = false
    c.el.addEventListener('ship-add', () => { fired = true })
    c.el.querySelector('[part="empty"] [data-action="add"]')!
      .dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })
})
