import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pipeline-step/sui-pipeline-step.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const part = (el: HTMLElement, name: string) =>
  shadow(el).querySelector(`[part="${name}"]`)

describe('<sui-pipeline-step>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-pipeline-step')).toBeDefined()
  })

  it('renders the type as the kind label, defaulting to agent', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', {}, 'build')
    await flush()
    expect(part(el, 'kind')?.textContent).toBe('agent')
  })

  it('reflects the type prop and updates the kind label', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', { type: 'gate' })
    await flush()
    expect(part(el, 'kind')?.textContent).toBe('gate')
    expect(el.getAttribute('type')).toBe('gate')
  })

  it('maps each state to its chip label (awaiting -> review)', async () => {
    const cases: Array<[string, string]> = [
      ['ok', 'ok'],
      ['running', 'running'],
      ['awaiting', 'review'],
      ['err', 'err'],
    ]
    for (const [state, label] of cases) {
      const { el } = mount(reg, 'sui-pipeline-step', { state })
      await flush()
      expect(part(el, 'chip')?.textContent).toBe(label)
      expect(el.getAttribute('state')).toBe(state)
    }
  })

  it('renders no chip label in the default (none) state', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', {})
    await flush()
    expect(part(el, 'chip')?.textContent).toBe('')
  })

  it('reflects the compact boolean prop', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', { compact: true })
    await flush()
    expect(el.hasAttribute('compact')).toBe(true)
  })

  it('projects the default slot as the step id', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', {}, 'deploy-step')
    await flush()
    const slot = shadow(el).querySelector('.id slot') as HTMLSlotElement
    const assigned = slot.assignedNodes().map((n) => n.textContent).join('')
    expect(assigned).toBe('deploy-step')
  })
})
