import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pipeline-card/sui-pipeline-card.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot ?? el
const part = (el: HTMLElement, name: string) =>
  shadow(el).querySelector(`[part="${name}"]`)

describe('<sui-pipeline-card>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-pipeline-card')).toBeDefined()
  })

  it('renders the card surface with the name in the default slot', async () => {
    const { el } = mount(reg, 'sui-pipeline-card', {}, 'deploy-prod')
    await flush()
    expect(part(el, 'card')).not.toBeNull()
    const slot = shadow(el).querySelector('.name slot') as HTMLSlotElement
    const assigned = slot.assignedNodes().map((n) => n.textContent).join('')
    expect(assigned).toBe('deploy-prod')
  })

  it('maps each state to its chip label (awaiting -> review)', async () => {
    const cases: Array<[string, string]> = [
      ['ok', 'ok'],
      ['running', 'running'],
      ['awaiting', 'review'],
      ['err', 'err'],
    ]
    for (const [state, label] of cases) {
      const { el } = mount(reg, 'sui-pipeline-card', { state })
      await flush()
      expect(part(el, 'chip')?.textContent).toBe(label)
      expect(el.getAttribute('state')).toBe(state)
    }
  })

  it('renders no chip label in the default (none) state', async () => {
    const { el } = mount(reg, 'sui-pipeline-card', {})
    await flush()
    expect(part(el, 'chip')?.textContent).toBe('')
  })

  it('reflects the interactive boolean prop', async () => {
    const { el } = mount(reg, 'sui-pipeline-card', { interactive: true })
    await flush()
    expect(el.hasAttribute('interactive')).toBe(true)
  })

  it('exposes named slots for desc, steps and recent', async () => {
    const { el } = mount(reg, 'sui-pipeline-card')
    await flush()
    for (const name of ['desc', 'steps', 'recent']) {
      expect(shadow(el).querySelector(`slot[name="${name}"]`)).not.toBeNull()
    }
  })
})
