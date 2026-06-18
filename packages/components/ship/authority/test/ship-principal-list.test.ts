// Behavioral spec for <ship-principal-list> — the "who" of authority. Data-driven
// (principals JSON prop), COMPOSES <ship-blast-tag> for granted-tier chips, and
// emits ship-edit{name} when a row's "Edit grants" is pressed.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import { compileSui, evalCompiledJs } from '@ship/sui-dom/test-utils'
import controller from '../src/ship-principal-list/ship-principal-list.controller'

function register(rel: string, tag: string) {
  const src = readFileSync(resolve(__dirname, rel), 'utf8')
  evalCompiledJs(compileSui(src, { componentName: tag, mode: 'component' }).js)
}
register('../src/ship-blast-tag/ship-blast-tag.sui', 'ship-blast-tag')

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-principal-list/ship-principal-list.sui'),
  'utf8',
)
const TAG = 'ship-principal-list'

const PRINCIPALS = [
  {
    name: 'lead-tracker',
    harness: 'floki',
    tone: 'primary',
    grants: [
      { domain: 'collections', tier: 'C2' },
      { domain: 'mcp', tier: 'C1' },
    ],
    denials: ['account'],
  },
  {
    name: 'code-reviewer',
    harness: 'claude',
    grants: [{ domain: 'runtime', tier: 'C2' }],
    denials: [],
  },
]

function mount(principals: unknown) {
  return renderComponent(SRC, TAG, {
    controller,
    attrs: { principals: JSON.stringify(principals) },
  })
}
function rowFor(c: { el: HTMLElement }, name: string): HTMLElement {
  const nm = Array.from(c.el.querySelectorAll('.nm')).find(
    (s) => s.textContent?.trim() === name,
  )
  if (!nm) throw new Error(`no row for "${name}"`)
  return nm.closest('.row') as HTMLElement
}

describe('<ship-principal-list>', () => {
  it('renders a row per principal with name, harness and derived initials', () => {
    const c = mount(PRINCIPALS)
    expect(c.el.querySelectorAll('.row').length).toBe(2)
    const row = rowFor(c, 'lead-tracker')
    expect(row.textContent).toContain('floki')
    expect(row.querySelector('.tile')?.textContent).toBe('LT')
    c.unmount()
  })

  it('composes a blast-tag per granted domain and renders denials', () => {
    const c = mount(PRINCIPALS)
    const row = rowFor(c, 'lead-tracker')
    const tiers = Array.from(row.querySelectorAll('ship-blast-tag')).map((t) =>
      t.getAttribute('tier'),
    )
    expect(tiers).toEqual(['C2', 'C1'])
    expect(row.textContent).toContain('account ✕')
    c.unmount()
  })

  it('emits ship-edit{name} when "Edit grants" is pressed', () => {
    const c = mount(PRINCIPALS)
    let detail: unknown = null
    c.el.addEventListener('ship-edit', (e) => { detail = (e as CustomEvent).detail })
    const row = rowFor(c, 'code-reviewer')
    row.querySelector('button.edit')!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ name: 'code-reviewer' })
    c.unmount()
  })

  it('renders no rows for empty input', () => {
    const c = mount([])
    expect(c.el.querySelectorAll('.row').length).toBe(0)
    c.unmount()
  })
})
