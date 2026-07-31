import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pipeline-run/sui-pipeline-run.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const part = (el: HTMLElement, name: string) =>
  shadow(el).querySelector(`[part="${name}"]`)

describe('<sui-pipeline-run>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-pipeline-run')).toBeDefined()
  })

  it('renders the run + header + rail parts', async () => {
    const { el } = mount(reg, 'sui-pipeline-run')
    await flush()
    expect(part(el, 'run')).not.toBeNull()
    expect(part(el, 'header')).not.toBeNull()
    expect(part(el, 'rail')).not.toBeNull()
  })

  it('projects the default slot as the run name', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', {}, 'run #42')
    await flush()
    const slot = shadow(el).querySelector('.name slot') as HTMLSlotElement
    const assigned = slot.assignedNodes().map((n) => n.textContent).join('')
    expect(assigned).toBe('run #42')
  })

  it('maps each state to its chip label (awaiting -> review)', async () => {
    const cases: Array<[string, string]> = [
      ['ok', 'ok'],
      ['running', 'running'],
      ['awaiting', 'review'],
      ['err', 'err'],
    ]
    for (const [state, label] of cases) {
      const { el } = mount(reg, 'sui-pipeline-run', { state })
      await flush()
      expect(part(el, 'chip')?.textContent).toBe(label)
      expect(el.getAttribute('state')).toBe(state)
    }
  })

  it('renders no chip label in the default (none) state', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', {})
    await flush()
    expect(part(el, 'chip')?.textContent).toBe('')
  })

  it('exposes a meta slot and a rail slot for step cells', async () => {
    const { el } = mount(reg, 'sui-pipeline-run')
    await flush()
    expect(shadow(el).querySelector('slot[name="meta"]')).not.toBeNull()
    expect(shadow(el).querySelector('slot[name="rail"]')).not.toBeNull()
  })
})
