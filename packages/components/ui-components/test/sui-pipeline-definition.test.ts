import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pipeline-definition/sui-pipeline-definition.compiled.js'
// sui-card lives in @ship/sui-primitives.
import '@ship/sui-primitives'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-pipeline-definition>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-pipeline-definition')).toBeDefined()
  })

  it('composes a sui-card with interactive + accent-bar', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', { name: 'ship', steps: '3' })
    await flush()
    const card = shadow(el).querySelector('sui-card')
    expect(card).not.toBeNull()
    expect(card!.hasAttribute('interactive')).toBe(true)
    expect(card!.hasAttribute('accent-bar')).toBe(true)
  })

  it('renders name in the header', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', { name: 'release', steps: '4' })
    await flush()
    expect(shadow(el).querySelector('.name')?.textContent).toBe('release')
  })

  it('pluralizes the step count for !== 1', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', { name: 'x', steps: '4' })
    await flush()
    expect(shadow(el).querySelector('.steps')?.textContent).toBe('4 steps')
  })

  it('uses the singular form when steps === 1', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', { name: 'x', steps: '1' })
    await flush()
    expect(shadow(el).querySelector('.steps')?.textContent).toBe('1 step')
  })

  it('renders the description when set', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', {
      name: 'x',
      steps: '2',
      description: 'Build then deploy.',
    })
    await flush()
    const p = shadow(el).querySelector('p.desc')
    expect(p?.textContent).toBe('Build then deploy.')
    expect(p?.classList.contains('no-desc')).toBe(false)
  })

  it('renders the muted fallback when description is empty', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', { name: 'x', steps: '2' })
    await flush()
    const p = shadow(el).querySelector('p.desc')
    expect(p?.textContent).toBe('No description')
    expect(p?.classList.contains('no-desc')).toBe(true)
  })

  it('sets role=article', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', { name: 'x', steps: '2' })
    await flush()
    expect(el.getAttribute('role')).toBe('article')
  })

  it('updates aria-label with name + steps + description', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', {
      name: 'release',
      steps: '4',
      description: 'Ship it.',
    })
    await flush()
    expect(el.getAttribute('aria-label')).toBe(
      'Pipeline release, 4 steps: Ship it.',
    )
  })

  it('omits the description suffix from aria-label when empty', async () => {
    const { el } = mount(reg, 'sui-pipeline-definition', { name: 'x', steps: '1' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Pipeline x, 1 step')
  })
})
