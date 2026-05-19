import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pipeline-run/sui-pipeline-run.compiled.js'
// sui-card / sui-badge live in @ship/primitives.
import '@ship/primitives'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-pipeline-run>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-pipeline-run')).toBeDefined()
  })

  it('composes a sui-card with status-rail', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r-1', state: 'running' })
    await flush()
    const card = shadow(el).querySelector('sui-card')
    expect(card).not.toBeNull()
    expect(card!.hasAttribute('status-rail')).toBe(true)
  })

  it('renders the slug in the header', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'run-42' })
    await flush()
    expect(shadow(el).querySelector('.slug')?.textContent).toBe('run-42')
  })

  it('renders a sui-badge with the state text', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r', state: 'completed' })
    await flush()
    const badge = shadow(el).querySelector('sui-badge')
    expect(badge?.textContent?.trim()).toBe('completed')
  })

  it('maps running → info tone on the card', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r', state: 'running' })
    await flush()
    expect(shadow(el).querySelector('sui-card')?.getAttribute('tone')).toBe('info')
  })

  it('maps completed → success tone', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r', state: 'completed' })
    await flush()
    expect(shadow(el).querySelector('sui-card')?.getAttribute('tone')).toBe('success')
  })

  it('maps failed → destructive tone', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r', state: 'failed' })
    await flush()
    expect(shadow(el).querySelector('sui-card')?.getAttribute('tone')).toBe('destructive')
  })

  it('maps gatepending → warning tone', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r', state: 'gatepending' })
    await flush()
    expect(shadow(el).querySelector('sui-card')?.getAttribute('tone')).toBe('warning')
  })

  it('maps merged → accent tone', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r', state: 'merged' })
    await flush()
    expect(shadow(el).querySelector('sui-card')?.getAttribute('tone')).toBe('accent')
  })

  it('renders agent in a <code> block when set', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r', agent: 'ada' })
    await flush()
    expect(shadow(el).querySelector('.agent code')?.textContent).toBe('ada')
  })

  it('omits the agent block when unset', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r' })
    await flush()
    expect(shadow(el).querySelector('.agent')).toBeNull()
  })

  it('sets role=article', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', { slug: 'r' })
    await flush()
    expect(el.getAttribute('role')).toBe('article')
  })

  it('updates aria-label with slug + agent + state', async () => {
    const { el } = mount(reg, 'sui-pipeline-run', {
      slug: 'r-1',
      agent: 'ada',
      state: 'running',
    })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Pipeline run r-1 on ada: running')
  })
})
