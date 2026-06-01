// Behavioral spec for <ship-approval-card>, driven through the production
// Rust/WASM compiler via renderComponent (real DOM under happy-dom) with the
// paired controller wired in. Queried by what the operator perceives (visible
// text, button labels), never CSS.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives' // register composed sui-badge / sui-button
import controller from '../src/ship-approval-card/ship-approval-card.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-approval-card/ship-approval-card.sui'),
  'utf8',
)
const TAG = 'ship-approval-card'

const CAPABILITY = {
  id: 'r1', agent: 'Editor', space: 'Paperback', kind: 'capability',
  scope: 'fs:read', path: '~/Documents/drafts/',
  why: 'to import workspace-tab.md into the current draft',
  risk: 'medium', ageLabel: '38s',
}
const COMMAND = {
  id: 'r2', agent: 'Rust Lane', space: 'Ship', kind: 'command',
  command: 'git push --force-with-lease origin main',
  why: 'push 3 rebased commits', risk: 'high', ageLabel: '1m',
}

let mounted: { unmount(): void } | null = null
function mount(req: object) {
  const c = renderComponent(SRC, TAG, {
    controller,
    attrs: { request: JSON.stringify(req) },
  })
  mounted = c
  return c
}
function buttonByText(root: ParentNode, text: string): Element {
  const btn = Array.from(root.querySelectorAll('sui-button')).find(
    (b) => b.textContent?.trim() === text,
  )
  if (!btn) throw new Error(`no sui-button labelled "${text}"`)
  return btn
}

describe('<ship-approval-card>', () => {
  it('renders a capability request: agent, space, risk, scope, why', () => {
    const c = mount(CAPABILITY)
    const text = c.el.textContent ?? ''
    expect(text).toContain('Editor')
    expect(text).toContain('Paperback')
    expect(text).toContain('medium')
    expect(text).toContain('fs:read ~/Documents/drafts/')
    expect(text).toContain('Requests capability')
    expect(text).toContain('to import workspace-tab.md into the current draft')
    c.unmount()
  })

  it('renders a command request: the command and "Wants to run"', () => {
    const c = mount(COMMAND)
    const text = c.el.textContent ?? ''
    expect(text).toContain('git push --force-with-lease origin main')
    expect(text).toContain('Wants to run')
    c.unmount()
  })

  it('emits ship-decide{allow_session} when "Allow this session" is clicked', () => {
    const c = mount(CAPABILITY)
    let detail: unknown = null
    c.el.addEventListener('ship-decide', (e) => { detail = (e as CustomEvent).detail })
    buttonByText(c.el, 'Allow this session').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ id: 'r1', decision: 'allow_session' })
    c.unmount()
  })

  it('emits ship-decide{deny} when "Deny" is clicked', () => {
    const c = mount(CAPABILITY)
    let detail: unknown = null
    c.el.addEventListener('ship-decide', (e) => { detail = (e as CustomEvent).detail })
    buttonByText(c.el, 'Deny').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ id: 'r1', decision: 'deny' })
    c.unmount()
  })

  it('emits ship-inspect when the capability agent is clicked', () => {
    const c = mount(CAPABILITY)
    let detail: unknown = null
    c.el.addEventListener('ship-inspect', (e) => { detail = (e as CustomEvent).detail })
    const agentBtn = c.el.querySelector('[data-test="inspect"]')
    expect(agentBtn).not.toBeNull()
    agentBtn!.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ id: 'r1' })
    c.unmount()
  })

  it('a command request shows no inspect affordance (agent is plain text)', () => {
    const c = mount(COMMAND)
    expect(c.el.querySelector('[data-test="inspect"]')).toBeNull()
    c.unmount()
  })
})
