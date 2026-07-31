import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-op-card/ship-op-card.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)

describe('<ship-op-card>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-op-card')).toBeDefined()
  })

  it('renders op name, blast tier and reflects decision', async () => {
    const { el } = mount(reg, 'ship-op-card', {
      op: 'collections.read',
      blast: 'C2',
      decision: 'needs-approval',
    })
    await flush()
    expect(q(el, 'op-name')?.textContent).toBe('collections.read')
    expect(q(el, 'op-blast')?.textContent).toBe('C2')
    expect(el.getAttribute('decision')).toBe('needs-approval')
  })

  it('renders the denial reason as data (no crash)', async () => {
    const { el } = mount(reg, 'ship-op-card', {
      decision: 'denied',
      reason: 'needs the runtime grant (C2)',
    })
    await flush()
    expect(q(el, 'op-reason')?.textContent).toBe('needs the runtime grant (C2)')
  })

  it('toggles open and emits ship-toggle on chevron click', async () => {
    const { el } = mount(reg, 'ship-op-card', { args: '{ id: 1 }' })
    await flush()
    const onToggle = vi.fn()
    el.addEventListener('ship-toggle', onToggle)
    expect(el.hasAttribute('open')).toBe(false)

    ;(q(el, 'op-toggle') as HTMLButtonElement).click()
    await flush()

    expect(el.hasAttribute('open')).toBe(true)
    expect(onToggle).toHaveBeenCalledTimes(1)
    expect((onToggle.mock.calls[0][0] as CustomEvent).detail).toEqual({ open: true })

    ;(q(el, 'op-toggle') as HTMLButtonElement).click()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })
})
