import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-tree-view'
import type { ShipTreeNode } from './ship-tree-view'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  await Promise.resolve()
  return document.body.lastElementChild as HTMLElement
}

describe('ship-tree-view', () => {
  afterEach(cleanupTestDom)

  it('renders nodes with labels', async () => {
    const host = await mount(`
      <ship-tree-view>
        <ship-tree-node label="root" expanded>
          <ship-tree-node label="child" leaf></ship-tree-node>
        </ship-tree-node>
      </ship-tree-view>
    `)
    const nodes = host.querySelectorAll('ship-tree-node') as NodeListOf<ShipTreeNode>
    for (const n of nodes) await n.updateComplete
    expect(nodes[0].shadowRoot!.textContent).toContain('root')
    expect(nodes[1].shadowRoot!.textContent).toContain('child')
  })

  it('emits ship-toggle on caret click', async () => {
    const host = await mount('<ship-tree-view><ship-tree-node label="n"></ship-tree-node></ship-tree-view>')
    const n = host.querySelector('ship-tree-node') as ShipTreeNode
    await n.updateComplete
    const fn = vi.fn()
    n.addEventListener('ship-toggle', fn)
    const caret = n.shadowRoot!.querySelector('.caret') as HTMLElement
    caret.click()
    expect(fn).toHaveBeenCalled()
    expect((fn.mock.calls[0][0] as CustomEvent).detail.expanded).toBe(true)
    expect(n.expanded).toBe(true)
  })

  it('emits ship-select on row click', async () => {
    const host = await mount('<ship-tree-view><ship-tree-node label="n" leaf></ship-tree-node></ship-tree-view>')
    const n = host.querySelector('ship-tree-node') as ShipTreeNode
    await n.updateComplete
    const fn = vi.fn()
    n.addEventListener('ship-select', fn)
    ;(n.shadowRoot!.querySelector('.row') as HTMLElement).click()
    expect(fn).toHaveBeenCalled()
  })

  it('expands on ArrowRight', async () => {
    const host = await mount('<ship-tree-view><ship-tree-node label="n"></ship-tree-node></ship-tree-view>')
    const n = host.querySelector('ship-tree-node') as ShipTreeNode
    await n.updateComplete
    const row = n.shadowRoot!.querySelector('.row') as HTMLElement
    row.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    expect(n.expanded).toBe(true)
  })
})
