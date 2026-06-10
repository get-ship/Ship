// Behavioral spec for <ship-command-bar> — the cmdk keystone.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives'
import controller from '../src/ship-command-bar/ship-command-bar.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-command-bar/ship-command-bar.sui'),
  'utf8',
)
const TAG = 'ship-command-bar'

function mount(attrs: Record<string, string | boolean> = {}) {
  return renderComponent(SRC, TAG, { controller, attrs })
}
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-command-bar>', () => {
  it('reflects open and mode', () => {
    const c = mount({ open: true, mode: 'ai' })
    expect(c.el.hasAttribute('open')).toBe(true)
    expect(c.el.getAttribute('mode')).toBe('ai')
    c.unmount()
  })

  it('clicking a mode tab sets mode and emits ship-mode-change{mode}', () => {
    const c = mount({ open: true, mode: 'find' })
    let detail: unknown = null
    c.el.addEventListener('ship-mode-change', (e) => { detail = (e as CustomEvent).detail })
    const tab = root(c).querySelector('.mtab[data-m="ai"]') as HTMLElement
    tab.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ mode: 'ai' })
    expect(c.el.getAttribute('mode')).toBe('ai')
    c.unmount()
  })

  it('Enter in the input emits ship-submit{query}', () => {
    const c = mount({ open: true })
    let detail: unknown = null
    c.el.addEventListener('ship-submit', (e) => { detail = (e as CustomEvent).detail })
    const input = root(c).querySelector('[part="input"]') as HTMLInputElement
    input.value = 'open the editor'
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(detail).toEqual({ query: 'open the editor' })
    c.unmount()
  })

  it('Escape in the input closes the bar and emits ship-close', () => {
    const c = mount({ open: true })
    let fired = false
    c.el.addEventListener('ship-close', () => { fired = true })
    const input = root(c).querySelector('[part="input"]') as HTMLInputElement
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(fired).toBe(true)
    expect(c.el.hasAttribute('open')).toBe(false)
    c.unmount()
  })

  it('a slotted result with [data-id] emits ship-select{id} when clicked', () => {
    const c = mount({ open: true })
    let detail: unknown = null
    c.el.addEventListener('ship-select', (e) => { detail = (e as CustomEvent).detail })
    c.el.innerHTML = '<div data-id="r7">A result</div>'
    const result = c.el.querySelector('[data-id]') as HTMLElement
    result.dispatchEvent(new Event('click', { bubbles: true, composed: true } as EventInit))
    expect(detail).toEqual({ id: 'r7' })
    c.unmount()
  })

  it('⌘K toggles open and emits ship-close when already open', () => {
    const c = mount({ open: true })
    let fired = false
    c.el.addEventListener('ship-close', () => { fired = true })
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', metaKey: true, bubbles: true }))
    expect(fired).toBe(true)
    expect(c.el.hasAttribute('open')).toBe(false)
    c.unmount()
  })
})
