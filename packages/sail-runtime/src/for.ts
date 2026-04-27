// Keyed list reconciliation. Renders an array signal as a sequence
// of DOM nodes, preserving identity (focus, scroll, animation state)
// across reorders, inserts, removes, and updates.
//
// Sail's compiler turns `{#for item of list key={item.id}}…{/for}` into
// a call to bindFor at hydrate time. Authors don't import this directly.
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Keyed list reconciliation.

import { effect, type Signal } from './signal'
import type { Disposer } from './bind'

export interface BindForOptions<T> {
  /** Anchor placeholder — siblings are inserted AFTER this node. */
  anchor: Comment
  /** Reactive source of the array. */
  items: () => readonly T[]
  /** Per-item key — must be stable + unique per item identity. */
  key: (item: T, index: number) => unknown
  /** Render one item; returns its top-level nodes + an optional cleanup. */
  render: (item: T, index: number) => RenderedItem
}

export interface RenderedItem {
  nodes: Node[]
  cleanup?: () => void
  /**
   * Optional in-place updater. Called on reuse — when the same key
   * appears in the next list with a different item value or position.
   * Lets the row push the new item/index into its internal signals
   * without tearing down DOM. Without this, reused rows keep their
   * original closure and won't reflect updates.
   */
  update?: (item: unknown, index: number) => void
}

interface ItemRecord<T> {
  key: unknown
  item: T
  nodes: Node[]
  cleanup?: () => void
  update?: (item: unknown, index: number) => void
}

/**
 * Bind a keyed list. The renderer is called once per new item; reused
 * items keep their original DOM nodes across reorders.
 *
 * Reconciliation is O(n+m) — diff by key set, insert new keys, remove
 * gone keys, move existing keys into position.
 */
export function bindFor<T>(opts: BindForOptions<T>): Disposer {
  const { anchor, items, key, render } = opts
  let prev: ItemRecord<T>[] = []
  let active = true

  const dispose = effect(() => {
    if (!active) return
    const next = items()
    const nextKeys = new Map<unknown, { item: T; index: number }>()
    for (let i = 0; i < next.length; i++) {
      const item = next[i]!
      const k = key(item, i)
      nextKeys.set(k, { item, index: i })
    }

    const prevByKey = new Map<unknown, ItemRecord<T>>()
    for (const r of prev) prevByKey.set(r.key, r)

    // 1. Remove items that are gone in the next array.
    for (const r of prev) {
      if (!nextKeys.has(r.key)) {
        for (const n of r.nodes) {
          try {
            n.parentNode?.removeChild(n)
          } catch {
            // Already detached — ignore.
          }
        }
        r.cleanup?.()
      }
    }

    // 2. Walk the new array; reuse where keys match, render where new.
    //    Insert each item's nodes immediately after the running cursor.
    let cursor: Node = anchor
    const newRecords: ItemRecord<T>[] = []
    for (let i = 0; i < next.length; i++) {
      const item = next[i]!
      const k = key(item, i)
      const existing = prevByKey.get(k)
      let record: ItemRecord<T>
      if (existing) {
        existing.item = item
        existing.update?.(item, i)
        record = existing
      } else {
        const r = render(item, i)
        record = {
          key: k,
          item,
          nodes: r.nodes,
          ...(r.cleanup && { cleanup: r.cleanup }),
          ...(r.update && { update: r.update }),
        }
      }
      // Move/insert nodes into position right after `cursor`.
      const parent = anchor.parentNode
      if (!parent) {
        // Anchor isn't in a tree — defer (this can happen mid-disconnect).
        continue
      }
      for (const n of record.nodes) {
        const desired = cursor.nextSibling
        if (n !== desired) {
          parent.insertBefore(n, desired)
        }
        cursor = n
      }
      newRecords.push(record)
    }

    prev = newRecords
  })

  return () => {
    active = false
    dispose()
    for (const r of prev) {
      for (const n of r.nodes) {
        try {
          n.parentNode?.removeChild(n)
        } catch {
          // Already detached.
        }
      }
      r.cleanup?.()
    }
    prev = []
  }
}

/**
 * Convenience: bind a list using a signal directly. Equivalent to
 * `bindFor({ items: () => sig(), ... })`.
 */
export function bindForSignal<T>(
  anchor: Comment,
  sig: Signal<readonly T[]>,
  key: (item: T, index: number) => unknown,
  render: (item: T, index: number) => RenderedItem,
): Disposer {
  return bindFor({ anchor, items: () => sig(), key, render })
}
