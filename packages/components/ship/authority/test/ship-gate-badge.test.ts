// Characterization spec for <ship-gate-badge> — the presentational gate-verdict
// atom. No controller: the decision reflects to the host attribute (CSS picks
// the tone + dot) and the matching label shows via CSS.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-gate-badge/ship-gate-badge.sui'),
  'utf8',
)
const TAG = 'ship-gate-badge'

const DECISIONS = ['allowed', 'approved', 'needs-approval', 'denied', 'running']

function mount(decision?: string) {
  return renderComponent(SRC, TAG, { attrs: decision ? { decision } : {} })
}

describe('<ship-gate-badge>', () => {
  it('exposes a [part="badge"] with a status dot and all verdict labels', () => {
    const c = mount('allowed')
    const badge = c.el.querySelector('[part="badge"]')
    expect(badge).not.toBeNull()
    expect(badge!.querySelector('.d')).not.toBeNull()
    const labels = Array.from(badge!.querySelectorAll('.l')).map((s) => s.textContent)
    expect(labels).toEqual(['allowed', 'approved', 'needs approval', 'denied', 'running'])
    c.unmount()
  })

  for (const decision of DECISIONS) {
    it(`reflects decision="${decision}"`, () => {
      const c = mount(decision)
      expect(c.el.getAttribute('decision')).toBe(decision)
      expect(c.el.querySelector('[part="badge"]')).not.toBeNull()
      c.unmount()
    })
  }
})
