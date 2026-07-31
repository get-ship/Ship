import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-file-tree/ship-file-tree.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// <ship-file-tree> is RUNTIME-FREE and purely presentational: a titled panel
// that composes the generic <sui-tree> primitive. It owns NO hierarchy
// behaviour — expand/collapse/keyboard/selection all live in <sui-tree>, whose
// events bubble through this wrapper to the host. It emits nothing and imports
// nothing. The composed <sui-tree> renders as an unregistered element here;
// these tests assert the panel chrome + that the tree is composed and fed the
// forwarded `nodes` / `selectable` contract.

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const tree = (el: HTMLElement) =>
  shadow(el).querySelector('sui-tree') as HTMLElement | null

describe('<ship-file-tree>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-file-tree')).toBeDefined()
  })

  it('renders the context label in the header', async () => {
    const { el } = mount(reg, 'ship-file-tree', { label: 'Views' })
    await flush()
    expect(shadow(el).querySelector('.head .t')?.textContent?.trim()).toBe('Views')
  })

  it('composes <sui-tree> in the body', () => {
    const { el } = mount(reg, 'ship-file-tree')
    expect(tree(el)).not.toBeNull()
  })

  it('forwards nodes + selectable to the composed tree', async () => {
    const nodes = JSON.stringify([{ id: 'a', label: 'a.ts', icon: 'file' }])
    const { el } = mount(reg, 'ship-file-tree', { nodes, selectable: 'single' })
    await flush()
    const t = tree(el)!
    expect(t.getAttribute('nodes')).toBe(nodes)
    expect(t.getAttribute('selectable')).toBe('single')
  })

  it('exposes a host-filled header actions slot (no baked-in buttons)', () => {
    const { el } = mount(reg, 'ship-file-tree')
    expect(shadow(el).querySelector('.head slot[name="actions"]')).not.toBeNull()
    // The wrapper owns no New / Collapse buttons — those are the host's job.
    expect(shadow(el).querySelector('.head button')).toBeNull()
  })
})
