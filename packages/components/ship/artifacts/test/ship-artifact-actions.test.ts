import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-artifact-actions/ship-artifact-actions.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

// Returns a promise that resolves with the next matching event. Attach
// the listener FIRST, then drive the action — no real-clock timer.
function nextEvent(target: EventTarget, name: string): Promise<CustomEvent> {
  const { promise, resolve } = Promise.withResolvers<CustomEvent>()
  target.addEventListener(name, (e) => resolve(e as CustomEvent), { once: true })
  return promise
}

function clickButton(el: HTMLElement, testAttr: string): void {
  const root = shadow(el)
  const btn = root.querySelector(`[data-test="${testAttr}"]`) as HTMLElement | null
  expect(btn).not.toBeNull()
  btn!.click()
}

describe('<ship-artifact-actions>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-artifact-actions')).toBeDefined()
  })

  it('renders Promote / Rename / Delete buttons plus the artifact name', async () => {
    const { el } = mount(reg, 'ship-artifact-actions', {
      record: JSON.stringify({ id: 'a1', name: 'arch.png' }),
    })
    await flush()
    const root = shadow(el)
    expect(root.querySelector('[data-test="action-promote"]')).not.toBeNull()
    expect(root.querySelector('[data-test="action-rename"]')).not.toBeNull()
    expect(root.querySelector('[data-test="action-delete"]')).not.toBeNull()
    expect(root.querySelector('[data-test="actions-name"]')?.textContent).toContain('arch.png')
  })

  it('emits ship-artifact-promote { id } on Promote click', async () => {
    const { el } = mount(reg, 'ship-artifact-actions', {
      record: JSON.stringify({ id: 'a1', name: 'arch.png' }),
    })
    await flush()
    const p = nextEvent(el, 'ship-artifact-promote')
    clickButton(el, 'action-promote')
    const evt = await p
    expect((evt as CustomEvent<{ id: string }>).detail).toEqual({ id: 'a1' })
  })

  it('opens the rename form on Rename click and emits ship-artifact-rename on submit', async () => {
    const { el } = mount(reg, 'ship-artifact-actions', {
      record: JSON.stringify({ id: 'a1', name: 'arch.png' }),
    })
    await flush()
    clickButton(el, 'action-rename')
    await flush()
    expect(shadow(el).querySelector('[data-test="rename-form"]')).not.toBeNull()
    const p = nextEvent(el, 'ship-artifact-rename')
    clickButton(el, 'rename-submit')
    const evt = await p
    expect((evt as CustomEvent<{ id: string; name: string }>).detail.id).toBe('a1')
    expect((evt as CustomEvent<{ id: string; name: string }>).detail.name.length).toBeGreaterThan(0)
  })

  it('opens the delete dialog on Delete click and emits ship-artifact-delete on confirm', async () => {
    const { el } = mount(reg, 'ship-artifact-actions', {
      record: JSON.stringify({ id: 'a1', name: 'arch.png' }),
    })
    await flush()
    clickButton(el, 'action-delete')
    await flush()
    expect(shadow(el).querySelector('[data-test="delete-dialog"]')).not.toBeNull()
    const p = nextEvent(el, 'ship-artifact-delete')
    clickButton(el, 'delete-confirm')
    const evt = await p
    expect((evt as CustomEvent<{ id: string }>).detail).toEqual({ id: 'a1' })
  })
})
