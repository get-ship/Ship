// Behavioral spec for <ship-gate-log> — the live audit stream of gate verdicts.
// Data-driven (entries JSON prop) and COMPOSES <ship-blast-tag> + <ship-gate-badge>
// per row. Driven through the production compiler via renderComponent.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import { compileSui, evalCompiledJs } from '@ship/sui-dom/test-utils'
import controller from '../src/ship-gate-log/ship-gate-log.controller'

function register(rel: string, tag: string) {
  const src = readFileSync(resolve(__dirname, rel), 'utf8')
  evalCompiledJs(compileSui(src, { componentName: tag, mode: 'component' }).js)
}
register('../src/ship-blast-tag/ship-blast-tag.sui', 'ship-blast-tag')
register('../src/ship-gate-badge/ship-gate-badge.sui', 'ship-gate-badge')

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-gate-log/ship-gate-log.sui'),
  'utf8',
)
const TAG = 'ship-gate-log'

const ENTRIES = [
  { time: '12:00:32', session: '·0001', op: 'mcp.create', blast: 'C2', decision: 'approved' },
  { time: '12:00:02', session: '·0001', op: 'collections.read', blast: 'C0', decision: 'allowed' },
  { time: '11:50:09', session: '·0004', op: 'runtime.dispatch', blast: 'C2', decision: 'denied' },
]

function mount(attrs: Record<string, string>) {
  return renderComponent(SRC, TAG, { controller, attrs })
}

describe('<ship-gate-log>', () => {
  it('renders a row per entry with time and op, composing both atoms', () => {
    const c = mount({ entries: JSON.stringify(ENTRIES), space: 'space-acme' })
    expect(c.el.querySelectorAll('.row').length).toBe(3)
    expect(c.el.textContent).toContain('12:00:32')
    expect(c.el.textContent).toContain('mcp.create')
    expect(
      Array.from(c.el.querySelectorAll('ship-blast-tag')).map((t) => t.getAttribute('tier')),
    ).toEqual(['C2', 'C0', 'C2'])
    expect(
      Array.from(c.el.querySelectorAll('ship-gate-badge')).map((g) =>
        g.getAttribute('decision'),
      ),
    ).toEqual(['approved', 'allowed', 'denied'])
    c.unmount()
  })

  it('shows the live indicator and space label in the header by default', () => {
    const c = mount({ entries: JSON.stringify(ENTRIES), space: 'space-acme' })
    expect(c.el.querySelector('.live')).not.toBeNull()
    expect(c.el.textContent).toContain('space-acme')
    c.unmount()
  })

  it('drops the live indicator when live="false"', () => {
    const c = mount({ entries: '[]', live: 'false' })
    expect(c.el.querySelector('.live')).toBeNull()
    expect(c.el.querySelectorAll('.row').length).toBe(0)
    c.unmount()
  })
})
