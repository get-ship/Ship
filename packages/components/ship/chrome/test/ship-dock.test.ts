// Behavioral spec for <ship-dock> — pure slotted container, no controller.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-dock/ship-dock.sui'),
  'utf8',
)
const TAG = 'ship-dock'
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-dock>', () => {
  it('exposes a default slot and projects its children', () => {
    const c = renderComponent(SRC, TAG)
    expect(root(c).querySelector('slot')).not.toBeNull()
    c.el.innerHTML = '<span class="tile">A</span><span class="div"></span>'
    expect(c.el.textContent).toContain('A')
    c.unmount()
  })
})
