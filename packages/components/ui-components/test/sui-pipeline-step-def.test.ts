import { afterEach, describe, expect, it } from 'vitest'
// sui-pipeline-step-def was removed from ui-components/src in the #151 reorg
// (not carried into @ship/pipelines). Suite skipped (component gone).
// import '../src/sui-pipeline-step-def/sui-pipeline-step-def.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe.skip('<sui-pipeline-step-def> (removed in #151 reorg)', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-pipeline-step-def')).toBeDefined()
  })

  it('renders the name and a step-type chip', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', {
      name: 'compile',
      'step-type': 'agent',
    })
    await flush()
    expect(shadow(el).querySelector('.name')?.textContent).toBe('compile')
    const chip = shadow(el).querySelector('sui-chip')
    expect(chip?.textContent?.trim()).toBe('agent')
  })

  it('maps step-type=agent → info tone on the chip', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', {
      name: 'compile',
      'step-type': 'agent',
    })
    await flush()
    expect(shadow(el).querySelector('sui-chip')?.getAttribute('tone')).toBe('info')
  })

  it('maps step-type=gate → warning tone', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', { name: 'g', 'step-type': 'gate' })
    await flush()
    expect(shadow(el).querySelector('sui-chip')?.getAttribute('tone')).toBe('warning')
  })

  it('maps step-type=merge → accent tone', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', { name: 'm', 'step-type': 'merge' })
    await flush()
    expect(shadow(el).querySelector('sui-chip')?.getAttribute('tone')).toBe('accent')
  })

  it('renders after-list as inline code entries', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', {
      name: 'deploy',
      after: 'build, test',
    })
    await flush()
    const codes = shadow(el).querySelectorAll('.after code')
    expect(codes.length).toBe(2)
    expect(codes[0].textContent).toBe('build')
    expect(codes[1].textContent).toBe('test')
  })

  it('renders chips for agent/harness/model when present', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', {
      name: 'x',
      agent: 'ada',
      harness: 'claude',
      model: 'opus',
    })
    await flush()
    const chips = shadow(el).querySelectorAll('.chips sui-chip')
    expect(chips.length).toBe(3)
  })

  it('omits the chips row when no agent/harness/model', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', { name: 'x' })
    await flush()
    expect(shadow(el).querySelector('.chips')).toBeNull()
  })

  it('renders a <pre> command block when command is set', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', {
      name: 'x',
      command: '  bun run test  ',
    })
    await flush()
    const pre = shadow(el).querySelector('pre')
    expect(pre).not.toBeNull()
    expect(pre!.textContent).toBe('bun run test')
  })

  it('omits the command block when unset', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', { name: 'x' })
    await flush()
    expect(shadow(el).querySelector('pre')).toBeNull()
  })

  it('exposes parts: row, header, command', async () => {
    const { el } = mount(reg, 'sui-pipeline-step-def', { name: 'x', command: 'ls' })
    await flush()
    expect(shadow(el).querySelector('[part="row"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part="header"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part="command"]')).not.toBeNull()
  })
})
