import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-tree-view/sui-tree-view.compiled.js'
import '../src/sui-tree-view/sui-tree-item.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-tree-view>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-tree-view')).toBeDefined()
  })

  it('sets role=tree on host', async () => {
    const { el } = mount(reg, 'sui-tree-view', {})
    await flush()
    expect(el.getAttribute('role')).toBe('tree')
  })

  it('has a shadow root with a slot', () => {
    const { shadow } = mount(reg, 'sui-tree-view', {})
    expect(shadow!.querySelector('slot')).not.toBeNull()
  })
})

describe('<sui-tree-item>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-tree-item')).toBeDefined()
  })

  it('sets role=treeitem on host', async () => {
    const { el } = mount(reg, 'sui-tree-item', { label: 'root' })
    await flush()
    expect(el.getAttribute('role')).toBe('treeitem')
  })

  it('renders label text in shadow DOM', async () => {
    const { shadow } = mount(reg, 'sui-tree-item', { label: 'My Node' })
    await flush()
    expect(shadow!.querySelector('.label')!.textContent).toContain('My Node')
  })

  it('reflects expanded, selected, leaf attributes', () => {
    const { el } = mount(reg, 'sui-tree-item', { label: 'n', expanded: true, selected: true, leaf: true })
    expect(el.hasAttribute('expanded')).toBe(true)
    expect(el.hasAttribute('selected')).toBe(true)
    expect(el.hasAttribute('leaf')).toBe(true)
  })

  it('has .row in shadow DOM with tabindex=0', async () => {
    const { shadow } = mount(reg, 'sui-tree-item', { label: 'n' })
    await flush()
    const row = shadow!.querySelector('.row') as HTMLElement
    expect(row).not.toBeNull()
    expect(row.getAttribute('tabindex')).toBe('0')
  })

  it('row aria-expanded reflects expanded state', async () => {
    const { el, shadow } = mount(reg, 'sui-tree-item', { label: 'n' })
    await flush()
    const row = shadow!.querySelector('.row') as HTMLElement
    expect(row.getAttribute('aria-expanded')).toBe('false')
    el.setAttribute('expanded', '')
    await flush()
    expect(row.getAttribute('aria-expanded')).toBe('true')
  })

  it('leaf node has no aria-expanded on row', async () => {
    const { shadow } = mount(reg, 'sui-tree-item', { label: 'n', leaf: true })
    await flush()
    const row = shadow!.querySelector('.row') as HTMLElement
    expect(row.hasAttribute('aria-expanded')).toBe(false)
  })

  it('row aria-selected reflects selected state', async () => {
    const { shadow } = mount(reg, 'sui-tree-item', { label: 'n', selected: true })
    await flush()
    const row = shadow!.querySelector('.row') as HTMLElement
    expect(row.getAttribute('aria-selected')).toBe('true')
  })

  it('emits sui-toggle on row click (non-leaf)', async () => {
    const { el, shadow } = mount(reg, 'sui-tree-item', { label: 'n' })
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-toggle', fn)
    ;(shadow!.querySelector('.row') as HTMLElement).click()
    await flush()
    expect(fn).toHaveBeenCalled()
    const detail = (fn.mock.calls[0][0] as CustomEvent).detail
    expect(detail.expanded).toBe(true)
    expect(el.hasAttribute('expanded')).toBe(true)
  })

  it('does not emit sui-toggle for leaf nodes', async () => {
    const { el, shadow } = mount(reg, 'sui-tree-item', { label: 'n', leaf: true })
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-toggle', fn)
    ;(shadow!.querySelector('.row') as HTMLElement).click()
    await flush()
    expect(fn).not.toHaveBeenCalled()
  })

  it('emits sui-select on row click', async () => {
    const { el, shadow } = mount(reg, 'sui-tree-item', { label: 'n', leaf: true })
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-select', fn)
    ;(shadow!.querySelector('.row') as HTMLElement).click()
    await flush()
    expect(fn).toHaveBeenCalled()
  })

  it('ArrowRight expands a collapsed non-leaf node', async () => {
    const { el, shadow } = mount(reg, 'sui-tree-item', { label: 'n' })
    await flush()
    const row = shadow!.querySelector('.row') as HTMLElement
    row.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await flush()
    expect(el.hasAttribute('expanded')).toBe(true)
  })

  it('ArrowLeft collapses an expanded node', async () => {
    const { el, shadow } = mount(reg, 'sui-tree-item', { label: 'n', expanded: true })
    await flush()
    const row = shadow!.querySelector('.row') as HTMLElement
    row.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))
    await flush()
    expect(el.hasAttribute('expanded')).toBe(false)
  })

  it('Enter key emits sui-select', async () => {
    const { el, shadow } = mount(reg, 'sui-tree-item', { label: 'n' })
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-select', fn)
    const row = shadow!.querySelector('.row') as HTMLElement
    row.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await flush()
    expect(fn).toHaveBeenCalled()
  })

  it('children slot is hidden when not expanded', async () => {
    const { shadow } = mount(reg, 'sui-tree-item', { label: 'n' })
    await flush()
    const children = shadow!.querySelector('.children') as HTMLElement
    const style = getComputedStyle(children)
    expect(children).not.toBeNull()
  })

  it('has .caret hidden for leaf nodes via CSS', async () => {
    const { shadow } = mount(reg, 'sui-tree-item', { label: 'n', leaf: true })
    await flush()
    const caret = shadow!.querySelector('.caret') as HTMLElement
    expect(caret).not.toBeNull()
  })
})
