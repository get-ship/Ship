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

/**
 * Inputs to `bindFor` — what to iterate, how to identify items, and
 * how to render them.
 */
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

/**
 * The shape returned by a `BindForOptions.render` call — DOM nodes for
 * one row plus optional cleanup and update hooks.
 */
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
 * Bind a keyed list to the DOM after `opts.anchor`. The renderer is
 * called once per new key; rows whose keys persist across renders keep
 * their original nodes (and any focus / scroll / animation state).
 *
 * Reconciliation is O(n+m): diff by key set, remove gone keys, insert
 * new keys, move existing keys into their new positions.
 *
 * Compiler target: `{#for item of list key={item.id}}…{/for}`.
 *
 * @param opts Anchor, reactive items source, key fn, and render fn.
 * @returns Disposer that detaches every row and stops updates.
 *
 * @example
 * ```ts
 * const items = signal<Row[]>([])
 * bindFor({
 *   anchor,
 *   items: () => items(),
 *   key: (row) => row.id,
 *   render: (row) => {
 *     const li = document.createElement('li')
 *     li.textContent = row.label
 *     return { nodes: [li] }
 *   },
 * })
 * ```
 *
 * @see bindForSignal for the common case where `items` is a signal.
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
 * Convenience wrapper around `bindFor` for the common case where the
 * items source is a signal. Equivalent to
 * `bindFor({ anchor, items: () => sig(), key, render })`.
 *
 * @param anchor Comment placeholder; rows are inserted after it.
 * @param sig Signal whose value is the current array.
 * @param key Stable, unique key per item identity.
 * @param render Per-row renderer.
 * @returns Disposer.
 *
 * @example
 * ```ts
 * const rows = signal<Row[]>([])
 * bindForSignal(anchor, rows, (r) => r.id, renderRow)
 * ```
 *
 * @see bindFor for the full options form.
 */
export function bindForSignal<T>(
  anchor: Comment,
  sig: Signal<readonly T[]>,
  key: (item: T, index: number) => unknown,
  render: (item: T, index: number) => RenderedItem,
): Disposer {
  return bindFor({ anchor, items: () => sig(), key, render })
}
