import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-agent-canvas/ship-agent-canvas.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`) as HTMLButtonElement | null

describe('<ship-agent-canvas>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-canvas')).toBeDefined()
  })

  it('reflects activity and renders the control-bar identity', async () => {
    const { el } = mount(reg, 'ship-agent-canvas', { agent: 'spawner', activity: 'thinking' })
    await flush()
    expect(el.getAttribute('activity')).toBe('thinking')
    expect(q(el, 'canvas-agent')?.textContent).toBe('spawner')
  })

  it('exposes the field slot for agent-slots', async () => {
    const { el } = mount(reg, 'ship-agent-canvas', {}, '<ship-agent-slot></ship-agent-slot>')
    await flush()
    expect(shadow(el).querySelector('.field slot')).not.toBeNull()
  })

  it('emits ship-undo / ship-stop / ship-save from the control bar', async () => {
    const { el } = mount(reg, 'ship-agent-canvas', { agent: 'a', session: 's1' })
    await flush()
    for (const [verb, evt] of [
      ['canvas-undo', 'ship-undo'],
      ['canvas-stop', 'ship-stop'],
      ['canvas-save', 'ship-save'],
    ] as const) {
      const spy = vi.fn()
      el.addEventListener(evt, spy)
      q(el, verb)!.click()
      expect(spy).toHaveBeenCalledTimes(1)
      expect((spy.mock.calls[0][0] as CustomEvent).detail).toEqual({ agent: 'a', session: 's1' })
    }
  })
})
