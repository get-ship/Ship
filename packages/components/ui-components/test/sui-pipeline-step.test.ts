import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pipeline-step/sui-pipeline-step.compiled.js'
// sui-spinner lives in @ship/sui-primitives.
import '@ship/sui-primitives'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-pipeline-step>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-pipeline-step')).toBeDefined()
  })

  it('renders name + type + status in shadow DOM', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', {
      name: 'compile',
      'step-type': 'run',
      state: 'pending',
    })
    await flush()
    const s = shadow(el)
    expect(s.querySelector('.name')?.textContent).toContain('compile')
    expect(s.querySelector('.type')?.textContent).toContain('run')
    expect(s.querySelector('.status')?.textContent).toContain('pending')
  })

  it('sets role=listitem by default', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', { name: 'a' })
    await flush()
    expect(el.getAttribute('role')).toBe('listitem')
  })

  it('reflects state attribute', () => {
    const { el } = mount(reg, 'sui-pipeline-step', { state: 'running' })
    expect(el.getAttribute('state')).toBe('running')
  })

  it('renders a spinner when state=running', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', { name: 'a', state: 'running' })
    await flush()
    expect(shadow(el).querySelector('sui-spinner')).not.toBeNull()
  })

  it('renders a glyph icon when state is not running', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', { name: 'a', state: 'completed' })
    await flush()
    expect(shadow(el).querySelector('sui-spinner')).toBeNull()
    expect(shadow(el).querySelector('.icon')?.textContent).toBe('✓')
  })

  it('shows the failed glyph when state=failed', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', { name: 'a', state: 'failed' })
    await flush()
    expect(shadow(el).querySelector('.icon')?.textContent).toBe('✗')
  })

  it('shows the pending glyph by default', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', { name: 'a' })
    await flush()
    expect(shadow(el).querySelector('.icon')?.textContent).toBe('○')
  })

  it('aria-label summarizes the step', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', {
      name: 'lint',
      'step-type': 'agent',
      state: 'completed',
      agent: 'ada',
    })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('agent step lint (ada): completed')
  })

  it('appends agent label to the type line when set', async () => {
    const { el } = mount(reg, 'sui-pipeline-step', {
      name: 'lint',
      'step-type': 'agent',
      agent: 'ada',
    })
    await flush()
    const text = shadow(el).querySelector('.type')?.textContent || ''
    expect(text).toContain('agent')
    expect(text).toContain('ada')
  })
})
