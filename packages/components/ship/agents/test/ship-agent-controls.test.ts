import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-agent-controls/ship-agent-controls.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLButtonElement | null

// Which verbs are *visible* per state is pure CSS (reflected-attribute
// selectors, no JS) — happy-dom doesn't resolve `:host([state])` cascade in
// getComputedStyle, so we assert the contract that IS observable: the state
// reflects, every verb is wired, and each verb emits its canonical event with
// the agent + current state in the detail. The per-state show/hide is covered
// by the reflected `state` attribute + the .sail selectors (design ratify grid).
const VERBS: Array<[string, string]> = [
  ['verb-start', 'ship-start'],
  ['verb-steer', 'ship-steer'],
  ['verb-approve', 'ship-approve'],
  ['verb-deny', 'ship-deny'],
  ['verb-rerun', 'ship-rerun'],
  ['verb-stop', 'ship-stop'],
]

describe('<ship-agent-controls>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-controls')).toBeDefined()
  })

  it('reflects the state attribute and renders the agent label', async () => {
    const { el } = mount(reg, 'ship-agent-controls', { state: 'suspended', agent: 'lead-tracker' })
    await flush()
    expect(el.getAttribute('state')).toBe('suspended')
    expect(q(el, 'controls-agent')?.textContent).toBe('lead-tracker')
  })

  it('wires all six context verbs (start · steer · approve · deny · rerun · stop)', async () => {
    const { el } = mount(reg, 'ship-agent-controls')
    await flush()
    for (const [verb] of VERBS) expect(q(el, verb)).not.toBeNull()
  })

  it('each verb emits its canonical event with agent + state', async () => {
    const { el } = mount(reg, 'ship-agent-controls', { state: 'active', agent: 'code-reviewer' })
    await flush()
    for (const [verb, evt] of VERBS) {
      const spy = vi.fn()
      el.addEventListener(evt, spy)
      q(el, verb)!.click()
      expect(spy, evt).toHaveBeenCalledTimes(1)
      expect((spy.mock.calls[0][0] as CustomEvent).detail).toEqual({
        agent: 'code-reviewer',
        state: 'active',
      })
    }
  })

  it('terminal states reflect for the Rerun verb (per SPEC, not Start)', async () => {
    const { el } = mount(reg, 'ship-agent-controls', { state: 'completed' })
    await flush()
    expect(el.getAttribute('state')).toBe('completed')
    el.setAttribute('state', 'failed')
    await flush()
    expect(el.getAttribute('state')).toBe('failed')
    // Rerun is the verb wired for terminal states; Start stays for idle only.
    expect(q(el, 'verb-rerun')).not.toBeNull()
  })
})
