// Characterization spec for <ship-blast-tag> — the presentational blast-tier
// atom. No controller: the tier reflects to the host attribute (CSS picks the
// tone) and the matching C# label is selected by CSS. Driven through the
// production compiler via renderComponent.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-blast-tag/ship-blast-tag.sui'),
  'utf8',
)
const TAG = 'ship-blast-tag'

function mount(tier?: string) {
  return renderComponent(SRC, TAG, { attrs: tier ? { tier } : {} })
}

describe('<ship-blast-tag>', () => {
  it('exposes a [part="tag"] element with all four tier labels present', () => {
    const c = mount('C0')
    const tag = c.el.querySelector('[part="tag"]')
    expect(tag).not.toBeNull()
    const labels = Array.from(tag!.querySelectorAll('.l')).map((s) => s.textContent)
    expect(labels).toEqual(['C0', 'C1', 'C2', 'C3'])
    c.unmount()
  })

  for (const tier of ['C0', 'C1', 'C2', 'C3']) {
    it(`reflects tier="${tier}" to the host attribute`, () => {
      const c = mount(tier)
      expect(c.el.getAttribute('tier')).toBe(tier)
      expect(c.el.querySelector(`.l${tier[1]}`)).not.toBeNull()
      c.unmount()
    })
  }
})
