// Behavioral spec for <ship-approvals-queue> — the batched gate inbox. It shows
// the pending count, slots the per-op checkpoints, and emits the operator's
// batch intent (ship-approve-all / ship-deny-all). Driven through the production
// compiler via renderComponent with its paired controller.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import controller from '../src/ship-approvals-queue/ship-approvals-queue.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-approvals-queue/ship-approvals-queue.sail'),
  'utf8',
)
const TAG = 'ship-approvals-queue'

function mount(attrs: Record<string, string> = {}, innerHTML = '') {
  const c = renderComponent(SRC, TAG, { controller, attrs })
  if (innerHTML) {
    c.el.innerHTML = innerHTML
  }
  return c
}
function buttonByText(root: ParentNode, text: string): Element {
  const btn = Array.from(root.querySelectorAll('button')).find(
    (b) => b.textContent?.trim() === text,
  )
  if (!btn) throw new Error(`no button labelled "${text}"`)
  return btn
}

describe('<ship-approvals-queue>', () => {
  it('renders "{count} pending" from the count prop', () => {
    const c = mount({ count: '3' })
    expect(c.el.textContent).toContain('3 pending')
    c.unmount()
  })

  it('projects slotted checkpoints into the body', () => {
    const c = mount({ count: '1' }, '<div data-test="row">a checkpoint</div>')
    expect(c.el.querySelector('[data-test="row"]')).not.toBeNull()
    c.unmount()
  })

  it('emits ship-approve-all when "Approve all" is pressed', () => {
    const c = mount({ count: '2' })
    let fired = false
    c.el.addEventListener('ship-approve-all', () => { fired = true })
    buttonByText(c.el, 'Approve all').dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })

  it('emits ship-deny-all when "Deny all" is pressed', () => {
    const c = mount({ count: '2' })
    let fired = false
    c.el.addEventListener('ship-deny-all', () => { fired = true })
    buttonByText(c.el, 'Deny all').dispatchEvent(new Event('click', { bubbles: true }))
    expect(fired).toBe(true)
    c.unmount()
  })
})
