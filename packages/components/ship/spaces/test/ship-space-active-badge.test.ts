// Characterization spec for <ship-space-active-badge> — the read-only chip that
// reflects the daemon-global DEFAULT POINTER (active_id). No controller: the
// name/path props drive the active vs cold (no active space) rendering.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-space-active-badge/ship-space-active-badge.sail'),
  'utf8',
)
const TAG = 'ship-space-active-badge'

function mount(attrs: Record<string, string> = {}) {
  return renderComponent(SRC, TAG, { attrs })
}

describe('<ship-space-active-badge>', () => {
  it('renders the active space name + path when name is set', () => {
    const c = mount({ name: 'ship-core', path: '~/dev/ship' })
    const badge = c.el.querySelector('[part="badge"]')
    expect(badge).not.toBeNull()
    expect(badge!.classList.contains('cold')).toBe(false)
    expect(badge!.querySelector('[part="label"]')?.textContent).toBe('ship-core')
    expect(badge!.textContent).toContain('~/dev/ship')
    c.unmount()
  })

  it('omits the path span when path is empty', () => {
    const c = mount({ name: 'ship-core' })
    const badge = c.el.querySelector('[part="badge"]')!
    expect(badge.querySelector('.path')).toBeNull()
    c.unmount()
  })

  it('renders the cold "No active space" empty state when name is absent', () => {
    const c = mount({})
    const badge = c.el.querySelector('[part="badge"]')!
    expect(badge.classList.contains('cold')).toBe(true)
    expect(badge.querySelector('[part="label"]')?.textContent).toBe('No active space')
    c.unmount()
  })
})
