import { afterEach, describe, expect, it } from 'vitest'
// sui-pipeline-dag was single-sourced into @ship/pipelines (#151) and removed
// from ui-components/src. Suite skipped (component gone here), import dropped.
// import '../src/sui-pipeline-dag/sui-pipeline-dag.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe.skip('<sui-pipeline-dag> (moved to @ship/pipelines #151)', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-pipeline-dag')).toBeDefined()
  })

  it('renders a shadow root with a default slot', () => {
    const { el } = mount(reg, 'sui-pipeline-dag', {}, '<span>step</span>')
    expect(shadow(el).querySelector('slot')).not.toBeNull()
  })

  it('sets role=list by default', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag')
    await flush()
    expect(el.getAttribute('role')).toBe('list')
  })

  it('sets aria-label to "pipeline steps" by default', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag')
    await flush()
    expect(el.getAttribute('aria-label')).toBe('pipeline steps')
  })

  it('does not overwrite a caller-provided role', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { role: 'group' })
    await flush()
    expect(el.getAttribute('role')).toBe('group')
  })

  it('does not overwrite a caller-provided aria-label', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { 'aria-label': 'custom' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('custom')
  })
})
