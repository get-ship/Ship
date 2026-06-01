// Behavioral spec for <ship-capability-request>, via renderComponent with the
// paired controller. Queried by what the operator perceives (visible text,
// button labels, the duration picker's pressed state), never CSS.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives' // register composed sui-dialog / sui-button
import controller from '../src/ship-capability-request/ship-capability-request.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-capability-request/ship-capability-request.sui'),
  'utf8',
)
const TAG = 'ship-capability-request'

const REQUEST = {
  id: 'r1', agent: 'Editor', space: 'Paperback', kind: 'capability',
  scope: 'fs:read', path: '~/Documents/drafts/',
  why: 'to import workspace-tab.md into the current draft',
  risk: 'medium', ageLabel: '38s',
}

function mount(attrs: Record<string, string> = {}) {
  return renderComponent(SRC, TAG, {
    controller,
    attrs: { request: JSON.stringify(REQUEST), open: 'true', ...attrs },
  })
}
function buttonByText(root: ParentNode, text: string): Element {
  const btn = Array.from(root.querySelectorAll('sui-button')).find(
    (b) => b.textContent?.trim() === text,
  )
  if (!btn) throw new Error(`no sui-button labelled "${text}"`)
  return btn
}
function durationByLabel(root: ParentNode, label: string): HTMLButtonElement {
  const btn = Array.from(root.querySelectorAll('button.duration')).find(
    (b) => b.textContent?.trim() === label,
  )
  if (!btn) throw new Error(`no duration option "${label}"`)
  return btn as HTMLButtonElement
}

describe('<ship-capability-request>', () => {
  it('lays out scope, why, and the duration options', () => {
    const c = mount()
    const text = c.el.textContent ?? ''
    expect(text).toContain('Capability request')
    expect(text).toContain('Editor')
    expect(text).toContain('Paperback')
    expect(text).toContain('Scope')
    expect(text).toContain('fs:read ~/Documents/drafts/')
    expect(text).toContain('Why')
    expect(text).toContain('to import workspace-tab.md into the current draft')
    expect(text).toContain('For how long')
    expect(text).toContain('Once')
    expect(text).toContain('This session')
    expect(text).toContain('Always')
    c.unmount()
  })

  it('defaults the duration to "This session"', () => {
    const c = mount()
    expect(durationByLabel(c.el, 'This session').getAttribute('aria-pressed')).toBe('true')
    expect(durationByLabel(c.el, 'Once').getAttribute('aria-pressed')).toBe('false')
    c.unmount()
  })

  it('selecting a duration moves the pressed state', () => {
    const c = mount()
    durationByLabel(c.el, 'Always').dispatchEvent(new Event('click', { bubbles: true }))
    expect(durationByLabel(c.el, 'Always').getAttribute('aria-pressed')).toBe('true')
    expect(durationByLabel(c.el, 'This session').getAttribute('aria-pressed')).toBe('false')
    c.unmount()
  })

  it('emits ship-decide{allow} with the chosen duration', () => {
    const c = mount()
    let detail: unknown = null
    c.el.addEventListener('ship-decide', (e) => { detail = (e as CustomEvent).detail })
    durationByLabel(c.el, 'Once').dispatchEvent(new Event('click', { bubbles: true }))
    buttonByText(c.el, 'Allow').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ id: 'r1', decision: 'allow', duration: 'once' })
    c.unmount()
  })

  it('emits ship-decide{deny} when Deny is clicked', () => {
    const c = mount()
    let detail: unknown = null
    c.el.addEventListener('ship-decide', (e) => { detail = (e as CustomEvent).detail })
    buttonByText(c.el, 'Deny').dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ id: 'r1', decision: 'deny', duration: 'session' })
    c.unmount()
  })
})
