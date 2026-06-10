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
function durationLabels(root: ParentNode): string[] {
  return Array.from(root.querySelectorAll('sui-toggle')).map((t) => t.textContent?.trim() ?? '')
}
function toggleGroup(root: ParentNode): HTMLElement {
  const g = root.querySelector('sui-toggle-group')
  if (!g) throw new Error('no sui-toggle-group')
  return g as HTMLElement
}
// The toggle-group's click→selection→sui-change behavior is the primitive's
// own concern (covered by its suite); here we test our contract WITH it —
// reacting to the sui-change it emits and binding our duration back to value.
function chooseDuration(root: ParentNode, value: string): void {
  toggleGroup(root).dispatchEvent(
    new CustomEvent('sui-change', { detail: { value }, bubbles: true }),
  )
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
    expect(durationLabels(c.el)).toEqual(['Once', 'This session', 'Always'])
    c.unmount()
  })

  it('binds the default duration (session) to the toggle group', () => {
    const c = mount()
    expect(toggleGroup(c.el).getAttribute('value')).toBe('session')
    c.unmount()
  })

  it('reflects a chosen duration back into the toggle group value', () => {
    const c = mount()
    chooseDuration(c.el, 'always')
    expect(toggleGroup(c.el).getAttribute('value')).toBe('always')
    c.unmount()
  })

  it('emits ship-decide{allow} with the chosen duration', () => {
    const c = mount()
    let detail: unknown = null
    c.el.addEventListener('ship-decide', (e) => { detail = (e as CustomEvent).detail })
    chooseDuration(c.el, 'once')
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
