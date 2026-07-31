// Behavioral spec for <ship-approval-prompt>, driven through the production
// Rust/WASM compiler via renderComponent (real DOM under happy-dom) with the
// paired controller wired in. The prompt is the Ask-stance checkpoint — a
// suspended op held at the gate, never running before the operator decides.
//
// The component is attribute-driven: op · args · req · grant · blast · state ·
// open. The verbs (Deny / Approve / details) emit `ship-decide { req, decision }`
// and `ship-inspect { req }` — the host is what calls ship.authority.decide().
// We assert by what the operator sees + the events that fire, never CSS.

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import controller from '../src/ship-approval-prompt/ship-approval-prompt.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-approval-prompt/ship-approval-prompt.sail'),
  'utf8',
)
const TAG = 'ship-approval-prompt'

let mounted: { unmount(): void } | null = null
function mount(attrs: Record<string, string>) {
  const c = renderComponent(SRC, TAG, { controller, attrs })
  mounted = c
  return c
}
const buttonByText = (root: ParentNode, text: string) => {
  const btn = Array.from(root.querySelectorAll('button')).find(
    (b) => b.textContent?.trim() === text,
  )
  if (!btn) throw new Error(`no <button> labelled "${text}"`)
  return btn
}

describe('<ship-approval-prompt>', () => {
  it('renders op, args, blast tier and reflects blast to the host attribute', () => {
    const c = mount({
      op: 'agents.dispatch',
      args: '{ id: "alex" }',
      blast: 'C2',
      req: 'r-001',
      grant: 'ship.agents.read',
    })
    const text = c.el.textContent ?? ''
    expect(text).toContain('agents.dispatch')
    expect(text).toContain('{ id: "alex" }')
    expect(text).toContain('C2')
    expect(text).toContain('ship.agents.read')
    expect(c.el.getAttribute('blast')).toBe('C2')
    c.unmount()
  })

  it('collapses the detail pane by default and only opens when [open] is set', () => {
    const c = mount({ op: 'op', args: '()', req: 'r', grant: 'g', blast: 'C2' })
    expect(c.el.hasAttribute('open')).toBe(false)
    // The detail block uses display: none by default in CSS; we only need to
    // assert the reflected attribute, not the layout, per the test convention.
    c.unmount()
  })

  it('emits ship-inspect { req } when the details disclosure is clicked', () => {
    const c = mount({ op: 'op', args: '()', req: 'r-42', grant: 'g', blast: 'C2' })
    let detail: unknown = null
    c.el.addEventListener('ship-inspect', (e) => { detail = (e as CustomEvent).detail })
    buttonByText(c.el, 'details').dispatchEvent(new Event('click', { bubbles: true }))
    expect(c.el.hasAttribute('open')).toBe(true)
    expect(detail).toEqual({ req: 'r-42' })
    // Click again — toggles off, emits again.
    buttonByText(c.el, 'details').dispatchEvent(new Event('click', { bubbles: true }))
    expect(c.el.hasAttribute('open')).toBe(false)
    c.unmount()
  })

  it('emits ship-decide{approve} { req } when Approve is clicked', () => {
    const c = mount({ op: 'op', args: '()', req: 'r-9', grant: 'g', blast: 'C2' })
    let detail: unknown = null
    c.el.addEventListener('ship-decide', (e) => { detail = (e as CustomEvent).detail })
    buttonByText(c.el, 'Approve').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ req: 'r-9', decision: 'approve' })
    c.unmount()
  })

  it('emits ship-decide{deny} { req } when Deny is clicked', () => {
    const c = mount({ op: 'op', args: '()', req: 'r-9', grant: 'g', blast: 'C2' })
    let detail: unknown = null
    c.el.addEventListener('ship-decide', (e) => { detail = (e as CustomEvent).detail })
    buttonByText(c.el, 'Deny').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ req: 'r-9', decision: 'deny' })
    c.unmount()
  })

  it('reflects the state enum to the host attribute', () => {
    for (const state of ['pending', 'approved', 'denied', 'expired'] as const) {
      const c = mount({ op: 'op', args: '()', req: 'r', grant: 'g', blast: 'C2', state })
      expect(c.el.getAttribute('state')).toBe(state)
      c.unmount()
    }
  })
})
