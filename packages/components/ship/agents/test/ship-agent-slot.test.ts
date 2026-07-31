import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-agent-slot/ship-agent-slot.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)

describe('<ship-agent-slot>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-slot')).toBeDefined()
  })

  it('reflects state and renders the empty-state label', async () => {
    const { el } = mount(reg, 'ship-agent-slot', { state: 'empty', label: 'Awaiting render' })
    await flush()
    expect(el.getAttribute('state')).toBe('empty')
    expect(q(el, 'slot-label')?.textContent).toBe('Awaiting render')
  })

  it('exposes a content slot for injected output', async () => {
    const { el } = mount(reg, 'ship-agent-slot', { state: 'filled' }, '<p>hi</p>')
    await flush()
    expect(shadow(el).querySelector('.content slot')).not.toBeNull()
  })

  it('emits ship-save with the label on Save click', async () => {
    const { el } = mount(reg, 'ship-agent-slot', { state: 'filled', label: 'chart' })
    await flush()
    const onSave = vi.fn()
    el.addEventListener('ship-save', onSave)
    ;(q(el, 'slot-save') as HTMLButtonElement).click()
    expect(onSave).toHaveBeenCalledTimes(1)
    expect((onSave.mock.calls[0][0] as CustomEvent).detail).toEqual({ label: 'chart' })
  })
})
