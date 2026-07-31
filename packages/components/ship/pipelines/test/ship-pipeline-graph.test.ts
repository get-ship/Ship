import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-pipeline-graph/ship-pipeline-graph.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
// The flow host carries bind:this AND an authored class/part. Querying through
// `.flow` pins #285: static attributes must survive alongside the ref binding.
const flow = (el: HTMLElement) => shadow(el).querySelector('.flow')!
const stepsAttr = (steps: unknown) => JSON.stringify(steps)

const FIXTURE = [
  { id: 'build', label: 'Build', state: 'done', sub: '12s', progress: 100 },
  { id: 'test', label: 'Test', state: 'running', sub: 'now', progress: 62, blast: 'C2' },
  { id: 'ship', label: 'Ship', state: 'queued', after: ['test'] },
]

describe('<ship-pipeline-graph>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-pipeline-graph')).toBeDefined()
  })

  it('keeps the authored class and part on the bind:this host (#285)', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: '[]' })
    await flush()
    const host = shadow(el).querySelector('.flow')
    expect(host).not.toBeNull()
    expect(host!.getAttribute('part')).toBe('flow')
  })

  it('renders the header from the name, status and run props', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', {
      steps: stepsAttr(FIXTURE),
      name: 'deploy',
      status: 'running',
      run: '#42 · 1m ago',
    })
    await flush()
    const hd = shadow(el).querySelector('[part="header"]')!
    expect(hd.querySelector('.nm')?.textContent).toBe('deploy')
    expect(hd.querySelector('.badge')?.textContent).toBe('running')
    expect(hd.querySelector('.runmeta')?.textContent).toBe('#42 · 1m ago')
    expect(el.getAttribute('status')).toBe('running')
  })

  it('renders one node per step with a stable state class and label', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: stepsAttr(FIXTURE) })
    await flush()
    const nodes = flow(el).querySelectorAll('.node[data-node-id]')
    expect(nodes.length).toBe(3)
    expect([...nodes].map((n) => n.getAttribute('data-node-id'))).toEqual([
      'build', 'test', 'ship',
    ])
    expect(nodes[0].classList.contains('st-done')).toBe(true)
    expect(nodes[1].classList.contains('st-running')).toBe(true)
    // Unknown/queued state falls back to st-queued.
    expect(nodes[2].classList.contains('st-queued')).toBe(true)
    expect(nodes[0].querySelector('.lbl')?.textContent).toBe('Build')
  })

  it('exposes each node as a focusable button for keyboard activation', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: stepsAttr(FIXTURE) })
    await flush()
    const node = flow(el).querySelector('.node')!
    expect(node.getAttribute('role')).toBe('button')
    expect(node.getAttribute('tabindex')).toBe('0')
    expect(node.getAttribute('aria-label')).toContain('Build')
  })

  it('renders the blast badge when a tier is supplied', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: stepsAttr(FIXTURE) })
    await flush()
    const badge = flow(el).querySelector('.blast.bl-C2')
    expect(badge?.textContent).toBe('C2')
  })

  it('emits ship-select with the node id on click', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: stepsAttr(FIXTURE) })
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-select', onSelect)
    ;(flow(el).querySelectorAll('.node')[1] as HTMLElement).click()
    expect(onSelect).toHaveBeenCalledTimes(1)
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ node: 'test' })
  })

  it('emits ship-select on Enter and Space keydown', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: stepsAttr(FIXTURE) })
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-select', onSelect)
    const node = flow(el).querySelector('.node') as HTMLElement
    for (const key of ['Enter', ' ']) {
      node.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }))
    }
    expect(onSelect).toHaveBeenCalledTimes(2)
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ node: 'build' })
  })

  it('renders the empty placeholder for no steps', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: '[]' })
    await flush()
    expect(flow(el).querySelector('.empty')?.textContent).toBe('no steps')
  })

  it('falls back to the empty placeholder on invalid JSON', async () => {
    const { el } = mount(reg, 'ship-pipeline-graph', { steps: 'nope' })
    await flush()
    expect(flow(el).querySelector('.empty')?.textContent).toBe('no steps')
  })
})
