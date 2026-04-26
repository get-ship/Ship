import { describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { bindFor, bindForSignal } from '../src/for'

interface Item { id: string; label: string }

function setup(initial: Item[]) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const anchor = document.createComment('list')
  root.appendChild(anchor)

  const $list = signal<Item[]>(initial)
  const renderCalls: { id: string; index: number }[] = []
  const cleanupCalls: string[] = []

  const dispose = bindForSignal(
    anchor,
    $list,
    (item) => item.id,
    (item, index) => {
      renderCalls.push({ id: item.id, index })
      const li = document.createElement('li')
      li.id = `item-${item.id}`
      li.textContent = item.label
      return {
        nodes: [li],
        cleanup: () => cleanupCalls.push(item.id),
      }
    },
  )

  const ids = () =>
    Array.from(root.querySelectorAll('li')).map((el) => el.id.replace('item-', ''))

  return { root, $list, ids, renderCalls, cleanupCalls, dispose }
}

describe('bindFor — initial render', () => {
  it('renders an empty list as no DOM nodes', () => {
    const { root, ids, renderCalls } = setup([])
    expect(ids()).toEqual([])
    expect(renderCalls).toEqual([])
    expect(root.querySelectorAll('li').length).toBe(0)
  })

  it('renders each item once', () => {
    const { ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    expect(ids()).toEqual(['a', 'b', 'c'])
    expect(renderCalls.map((r) => r.id)).toEqual(['a', 'b', 'c'])
  })

  it('renders inserted in correct order', () => {
    const items = [
      { id: 'z', label: 'Z' },
      { id: 'm', label: 'M' },
      { id: 'a', label: 'A' },
    ]
    const { ids } = setup(items)
    expect(ids()).toEqual(['z', 'm', 'a'])
  })
})

describe('bindFor — append', () => {
  it('appends one item without re-rendering existing', () => {
    const { $list, ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(renderCalls.length).toBe(2)
    $list.set([...$list(), { id: 'c', label: 'C' }])
    expect(ids()).toEqual(['a', 'b', 'c'])
    expect(renderCalls.length).toBe(3) // only `c` rendered
    expect(renderCalls[2]!.id).toBe('c')
  })

  it('appends many items', () => {
    const { $list, ids } = setup([{ id: 'a', label: 'A' }])
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
      { id: 'd', label: 'D' },
    ])
    expect(ids()).toEqual(['a', 'b', 'c', 'd'])
  })
})

describe('bindFor — prepend', () => {
  it('prepends one item without re-rendering existing', () => {
    const { $list, ids, renderCalls } = setup([
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([{ id: 'a', label: 'A' }, ...$list()])
    expect(ids()).toEqual(['a', 'b', 'c'])
    expect(renderCalls.length).toBe(3) // only `a` is new
    expect(renderCalls[2]!.id).toBe('a')
  })
})

describe('bindFor — remove', () => {
  it('removes an item from the middle, calls its cleanup', () => {
    const { $list, ids, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'c', label: 'C' },
    ])
    expect(ids()).toEqual(['a', 'c'])
    expect(cleanupCalls).toEqual(['b'])
  })

  it('removes the first item', () => {
    const { $list, ids } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    $list.set([{ id: 'b', label: 'B' }])
    expect(ids()).toEqual(['b'])
  })

  it('removes the last item', () => {
    const { $list, ids } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    $list.set([{ id: 'a', label: 'A' }])
    expect(ids()).toEqual(['a'])
  })

  it('removes all items', () => {
    const { $list, ids, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    $list.set([])
    expect(ids()).toEqual([])
    expect(cleanupCalls.sort()).toEqual(['a', 'b'])
  })
})

describe('bindFor — reorder', () => {
  it('swaps two adjacent items, preserving DOM identity', () => {
    const { root, $list, ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    const aBefore = root.querySelector('#item-a')!
    const bBefore = root.querySelector('#item-b')!
    $list.set([
      { id: 'b', label: 'B' },
      { id: 'a', label: 'A' },
    ])
    expect(ids()).toEqual(['b', 'a'])
    expect(renderCalls.length).toBe(2) // no re-render
    // Same DOM node identity — focus / scroll / animation preserved.
    expect(root.querySelector('#item-a')).toBe(aBefore)
    expect(root.querySelector('#item-b')).toBe(bBefore)
  })

  it('reverses a longer list', () => {
    const { $list, ids } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
      { id: 'd', label: 'D' },
    ])
    $list.set([...$list()].reverse())
    expect(ids()).toEqual(['d', 'c', 'b', 'a'])
  })

  it('moves an item from end to beginning', () => {
    const { $list, ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([
      { id: 'c', label: 'C' },
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(ids()).toEqual(['c', 'a', 'b'])
    expect(renderCalls.length).toBe(3)
  })
})

describe('bindFor — mixed mutations', () => {
  it('add + remove + reorder in one update', () => {
    const { $list, ids, renderCalls, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([
      { id: 'c', label: 'C' },         // moved
      { id: 'd', label: 'D' },         // new
      { id: 'a', label: 'A' },         // moved
      // b removed
    ])
    expect(ids()).toEqual(['c', 'd', 'a'])
    expect(renderCalls.length).toBe(4) // a, b, c, d
    expect(cleanupCalls).toEqual(['b'])
  })
})

describe('bindFor — preserves form state across reorder', () => {
  it('checkbox state survives a reorder', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const $list = signal<{ id: string }[]>([
      { id: 'a' },
      { id: 'b' },
      { id: 'c' },
    ])
    bindForSignal(
      anchor,
      $list,
      (item) => item.id,
      (item) => {
        const li = document.createElement('li')
        const cb = document.createElement('input')
        cb.type = 'checkbox'
        cb.id = `cb-${item.id}`
        li.appendChild(cb)
        return { nodes: [li] }
      },
    )
    const cbB = root.querySelector('#cb-b') as HTMLInputElement
    cbB.checked = true
    $list.set([{ id: 'c' }, { id: 'b' }, { id: 'a' }])
    expect((root.querySelector('#cb-b') as HTMLInputElement).checked).toBe(true)
  })
})

describe('bindFor — disposer', () => {
  it('removes all rendered nodes + calls cleanups', () => {
    const { root, dispose, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(root.querySelectorAll('li').length).toBe(2)
    dispose()
    expect(root.querySelectorAll('li').length).toBe(0)
    expect(cleanupCalls.sort()).toEqual(['a', 'b'])
  })

  it('subsequent signal changes are ignored after dispose', () => {
    const { $list, root, dispose } = setup([{ id: 'a', label: 'A' }])
    dispose()
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(root.querySelectorAll('li').length).toBe(0)
  })
})

describe('bindFor — large lists (perf sanity)', () => {
  it('renders 1000 items + reorders one without rendering others', () => {
    const items = Array.from({ length: 1000 }, (_, i) => ({
      id: String(i),
      label: `item ${i}`,
    }))
    const { $list, renderCalls, ids } = setup(items)
    expect(renderCalls.length).toBe(1000)

    // Move last to first
    const swapped = [items[999]!, ...items.slice(0, 999)]
    $list.set(swapped)
    expect(ids()[0]).toBe('999')
    expect(renderCalls.length).toBe(1000) // no new renders, just moves
  })
})
