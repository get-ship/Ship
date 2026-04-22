import { afterEach, describe, expect, it } from 'vitest'
import './ship-graph-canvas'
import type { ShipGraphCanvas } from './ship-graph-canvas'
import { cleanupTestDom } from './util'

async function mount(markup: string): Promise<ShipGraphCanvas> {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipGraphCanvas
  await el.updateComplete
  return el
}

const nodesPositioned = JSON.stringify([
  { id: 'a1', kind: 'agent', title: 'Alpha', sub: 'primary', x: 200, y: 300 },
  { id: 's1', kind: 'skill', title: 'Compile', x: 700, y: 200 },
  { id: 's2', kind: 'skill', title: 'Deploy', x: 700, y: 400 },
])
const edgesPositioned = JSON.stringify([
  { from: 'a1', to: 's1', kind: 'skill' },
  { from: 'a1', to: 's2', kind: 'skill', dashed: true },
])

describe('ship-graph-canvas', () => {
  afterEach(cleanupTestDom)

  it('renders all nodes and edge paths', async () => {
    const el = await mount(
      `<ship-graph-canvas nodes='${nodesPositioned}' edges='${edgesPositioned}'></ship-graph-canvas>`,
    )
    const nodes = el.shadowRoot!.querySelectorAll('.node')
    const paths = el.shadowRoot!.querySelectorAll('svg.edges path')
    expect(nodes.length).toBe(3)
    expect(paths.length).toBe(2)
  })

  it('positioned layout uses provided x/y as percentages of width/height', async () => {
    const el = await mount(
      `<ship-graph-canvas width="1000" height="600" nodes='${nodesPositioned}' edges='[]'></ship-graph-canvas>`,
    )
    const first = el.shadowRoot!.querySelector('.node[data-id="a1"]') as HTMLElement
    // 200/1000 = 20%, 300/600 = 50%
    expect(first.style.left).toBe('20%')
    expect(first.style.top).toBe('50%')
  })

  it('falls back to radial layout when positions are missing', async () => {
    const nodes = JSON.stringify([
      { id: 'a1', kind: 'agent', title: 'Alpha' },
      { id: 'sk1', kind: 'skill', title: 'Compile' },
      { id: 'mc1', kind: 'mcp', title: 'Fs' },
    ])
    const el = await mount(
      `<ship-graph-canvas nodes='${nodes}' edges='[]'></ship-graph-canvas>`,
    )
    const agent = el.shadowRoot!.querySelector('.node[data-id="a1"]') as HTMLElement
    const skill = el.shadowRoot!.querySelector('.node[data-id="sk1"]') as HTMLElement
    // Agent should land at the left cluster; skill further right in its band.
    const agentLeft = parseFloat(agent.style.left)
    const skillLeft = parseFloat(skill.style.left)
    expect(agentLeft).toBeLessThan(skillLeft)
    expect(agentLeft).toBeGreaterThan(0)
  })

  it('fires ship-node-select with {id, kind} on click', async () => {
    const el = await mount(
      `<ship-graph-canvas nodes='${nodesPositioned}' edges='[]'></ship-graph-canvas>`,
    )
    let detail: { id: string; kind: string } | null = null
    el.addEventListener('ship-node-select', (e: Event) => {
      detail = (e as CustomEvent).detail
    })
    const node = el.shadowRoot!.querySelector('.node[data-id="s1"]') as HTMLElement
    node.click()
    expect(detail).toEqual({ id: 's1', kind: 'skill' })
  })

  it('fires ship-node-select on Enter keypress', async () => {
    const el = await mount(
      `<ship-graph-canvas nodes='${nodesPositioned}' edges='[]'></ship-graph-canvas>`,
    )
    let fired = 0
    el.addEventListener('ship-node-select', () => fired++)
    const node = el.shadowRoot!.querySelector('.node[data-id="a1"]') as HTMLElement
    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fired).toBe(1)
  })

  it('selected-id highlights the matching node via [data-selected]', async () => {
    const el = await mount(
      `<ship-graph-canvas selected-id="s2" nodes='${nodesPositioned}' edges='[]'></ship-graph-canvas>`,
    )
    const selected = el.shadowRoot!.querySelector('.node[data-selected]') as HTMLElement
    expect(selected).toBeTruthy()
    expect(selected.getAttribute('data-id')).toBe('s2')
  })

  it('per-kind template substitutes ${title}, ${sub}, ${id}', async () => {
    const el = await mount(`
      <ship-graph-canvas nodes='${nodesPositioned}' edges='[]'>
        <template slot="node" data-kind="skill">
          <div class="skill-card">
            <span class="sid">\${id}</span>
            <span class="stitle">\${title}</span>
          </div>
        </template>
      </ship-graph-canvas>
    `)
    const nodeS1 = el.shadowRoot!.querySelector('.node[data-id="s1"]') as HTMLElement
    const title = nodeS1.querySelector('.stitle')
    const id = nodeS1.querySelector('.sid')
    expect(title?.textContent).toBe('Compile')
    expect(id?.textContent).toBe('s1')
  })

  it('renders an edge path with kind attribute and dashed flag', async () => {
    const el = await mount(
      `<ship-graph-canvas nodes='${nodesPositioned}' edges='${edgesPositioned}'></ship-graph-canvas>`,
    )
    const paths = el.shadowRoot!.querySelectorAll('svg.edges path')
    const kinds = Array.from(paths).map((p) => p.getAttribute('data-kind'))
    expect(kinds).toEqual(['skill', 'skill'])
    expect(paths[1].hasAttribute('data-dashed')).toBe(true)
    expect(paths[0].hasAttribute('data-dashed')).toBe(false)
  })

  it('sets tabindex=0, role=button, and a useful aria-label on each node', async () => {
    const el = await mount(
      `<ship-graph-canvas nodes='${nodesPositioned}' edges='[]'></ship-graph-canvas>`,
    )
    const first = el.shadowRoot!.querySelector('.node') as HTMLElement
    expect(first.getAttribute('tabindex')).toBe('0')
    expect(first.getAttribute('role')).toBe('button')
    expect(first.getAttribute('aria-label')).toContain('Alpha')
  })
})
