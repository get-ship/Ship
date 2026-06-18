// Behavioral spec for <ship-capability-bar> — the agent's standing grants made
// legible. Data-driven (can / cant JSON props) and COMPOSES <ship-blast-tag> for
// every tier chip. Driven through the production compiler via renderComponent.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import { compileSui, evalCompiledJs } from '@ship/sui-dom/test-utils'
import controller from '../src/ship-capability-bar/ship-capability-bar.controller'

function register(rel: string, tag: string) {
  const src = readFileSync(resolve(__dirname, rel), 'utf8')
  evalCompiledJs(compileSui(src, { componentName: tag, mode: 'component' }).js)
}
register('../src/ship-blast-tag/ship-blast-tag.sui', 'ship-blast-tag')

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-capability-bar/ship-capability-bar.sui'),
  'utf8',
)
const TAG = 'ship-capability-bar'

const CAN = [
  { domain: 'collections', tier: 'C2' },
  { domain: 'mcp', tier: 'C1' },
]
const CANT = [{ domain: 'account', tier: 'C3', why: 'operator-only' }]

function mount(attrs: Record<string, string>) {
  return renderComponent(SRC, TAG, { controller, attrs })
}

describe('<ship-capability-bar>', () => {
  it('renders each granted domain composing a blast-tag at its tier', () => {
    const c = mount({ can: JSON.stringify(CAN), cant: '[]' })
    expect(c.el.textContent).toContain('collections')
    expect(c.el.textContent).toContain('mcp')
    const tiers = Array.from(c.el.querySelectorAll('ship-blast-tag')).map((t) =>
      t.getAttribute('tier'),
    )
    expect(tiers).toEqual(['C2', 'C1'])
    c.unmount()
  })

  it('renders the denied group with the domain, tier and reason', () => {
    const c = mount({ can: JSON.stringify(CAN), cant: JSON.stringify(CANT) })
    expect(c.el.textContent).toContain("can't")
    expect(c.el.textContent).toContain('account')
    expect(c.el.textContent).toContain('operator-only')
    const denied = c.el.querySelector('.chip.locked ship-blast-tag')
    expect(denied?.getAttribute('tier')).toBe('C3')
    c.unmount()
  })

  it("omits the can't group entirely when no denials are passed", () => {
    const c = mount({ can: JSON.stringify(CAN), cant: '[]' })
    expect(c.el.querySelector('.chip.locked')).toBeNull()
    expect(c.el.textContent).not.toContain("can't")
    c.unmount()
  })

  it('renders nothing in the grant group for empty / invalid JSON', () => {
    const c = mount({ can: 'not json', cant: '[]' })
    expect(c.el.querySelectorAll('ship-blast-tag').length).toBe(0)
    c.unmount()
  })
})
