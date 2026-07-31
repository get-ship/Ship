import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-pipeline-dag/sui-pipeline-dag.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
// The dag host carries bind:this AND an authored class/part. Querying through
// `.dag` pins #285: static attributes must survive alongside the ref binding.
const dag = (el: HTMLElement) => shadow(el).querySelector('.dag')!
const stepsAttr = (steps: unknown) => JSON.stringify(steps)

const FIXTURE = [
  { id: 'plan', type: 'agent', state: 'ok' },
  { id: 'gate', type: 'gate', state: 'awaiting', after: ['plan'] },
  { id: 'merge', type: 'merge', state: 'never', after: ['gate'] },
]

describe('<sui-pipeline-dag>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-pipeline-dag')).toBeDefined()
  })

  it('keeps the authored class and part on the bind:this host (#285)', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { steps: '[]' })
    await flush()
    const host = shadow(el).querySelector('.dag')
    expect(host).not.toBeNull()
    expect(host!.getAttribute('part')).toBe('dag')
  })

  it('renders an svg with one node group per step', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { steps: stepsAttr(FIXTURE) })
    await flush()
    const svg = dag(el).querySelector('svg')!
    expect(svg).not.toBeNull()
    // Each step is a <g transform=…> with id + type-label <text> nodes.
    expect(svg.querySelectorAll('g[transform]').length).toBe(3)
    const text = svg.textContent ?? ''
    expect(text).toContain('plan')
    expect(text).toContain('GATE')
  })

  it('renders one edge path per resolved dependency', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { steps: stepsAttr(FIXTURE) })
    await flush()
    // plan->gate, gate->merge = 2 edges. Edge paths are direct svg children;
    // the two arrowhead marker <path>s live inside <defs>.
    const svg = dag(el).querySelector('svg')!
    const edges = [...svg.children].filter((c) => c.tagName.toLowerCase() === 'path')
    expect(edges.length).toBe(2)
  })

  it('animates the edge into a step that is running or awaiting', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { steps: stepsAttr(FIXTURE) })
    await flush()
    // The gate is awaiting → its inbound edge uses the active marker + animate.
    const active = dag(el).querySelector('svg path[marker-end="url(#pl-active)"]')
    expect(active).not.toBeNull()
    expect(active!.querySelector('animate')).not.toBeNull()
  })

  it('renders the empty placeholder for an empty step list', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { steps: '[]' })
    await flush()
    expect(dag(el).querySelector('.dag-empty')?.textContent).toBe('no steps')
  })

  it('falls back to the empty placeholder on invalid JSON', async () => {
    const { el } = mount(reg, 'sui-pipeline-dag', { steps: '{not json' })
    await flush()
    expect(dag(el).querySelector('.dag-empty')?.textContent).toBe('no steps')
  })
})
