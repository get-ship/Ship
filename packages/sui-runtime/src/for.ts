// Keyed list reconciliation. Renders an array signal as a sequence
// of DOM nodes, preserving identity (focus, scroll, animation state)
// across reorders, inserts, removes, and updates.
//
// The ship-set compiler turns `{#for item of list key={item.id}}…{/for}` into
// a call to bindFor at hydrate time. Authors don't import this directly.
//
// See .artifacts/specs/ship-set/03-runtime-architecture.md § Keyed list reconciliation.

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

// Returns a boolean mask where mask[i] is true iff seq[i] participates in
// a longest increasing subsequence. O(n log n) via patience sorting.
// seq must contain non-negative integers with no duplicates.
function lisMask(seq: number[]): boolean[] {
  const n = seq.length
  const mask = new Array<boolean>(n).fill(false)
  if (n === 0) return mask

  const pred = new Array<number>(n).fill(-1) // predecessor index in seq
  const tails: number[] = [] // tails[k] = index into seq of smallest-tail IS of length k+1

  for (let i = 0; i < n; i++) {
    const v = seq[i]!
    let lo = 0
    let hi = tails.length
    while (lo < hi) {
      const mid = (lo + hi) >>> 1
      if (seq[tails[mid]!]! < v) lo = mid + 1
      else hi = mid
    }
    if (lo > 0) pred[i] = tails[lo - 1]!
    tails[lo] = i
  }

  let idx = tails[tails.length - 1]!
  while (idx >= 0) {
    mask[idx] = true
    idx = pred[idx]!
  }
  return mask
}

/**
 * Bind a keyed list to the DOM after `opts.anchor`. The renderer is
 * called once per new key; rows whose keys persist across renders keep
 * their original nodes (and any focus / scroll / animation state).
 *
 * Reconciliation uses a longest-increasing-subsequence algorithm to
 * minimise DOM moves. Matched keys already in a stable relative order
 * are never moved; only out-of-order and new keys are inserted.
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
      if (nextKeys.has(k)) {
        // Duplicate key: silent overwrite would drop the earlier item
        // from the rendered output. Warn once per render — the user's
        // key function is broken (or the list contains duplicate
        // identity). Reconciliation continues using last-wins to keep
        // the DOM in a consistent state.
        console.warn(
          '[sui] {#for}: duplicate key',
          k,
          'at index',
          i,
          '(prior at',
          nextKeys.get(k)!.index,
          '); the earlier item will be dropped from the rendered output.',
          'Fix the key function so each item maps to a unique value.',
        )
      }
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

    // 2. Build new records list — reuse matched keys, render new ones.
    //    No DOM mutations in this pass.
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
      newRecords.push(record)
    }

    // 3. Find matched records already in a stable relative order (LIS of
    //    their previous-array positions). These records don't need DOM
    //    moves — only non-LIS and new records are repositioned.
    const prevPos = new Map<unknown, number>()
    for (let i = 0; i < prev.length; i++) prevPos.set(prev[i]!.key, i)

    const matchedAt: number[] = []   // new-array positions of matched records
    const matchedPrev: number[] = [] // their prev-array positions
    for (let i = 0; i < newRecords.length; i++) {
      const p = prevPos.get(newRecords[i]!.key)
      if (p !== undefined) {
        matchedAt.push(i)
        matchedPrev.push(p)
      }
    }

    const lisMasked = lisMask(matchedPrev)
    const stable = new Set<number>()
    for (let j = 0; j < matchedAt.length; j++) {
      if (lisMasked[j]) stable.add(matchedAt[j]!)
    }

    // 4. Place all records. Stable records advance the cursor without any
    //    DOM mutation. Unstable / new records are moved via insertBefore.
    const parent = anchor.parentNode
    let cursor: Node = anchor
    for (let i = 0; i < newRecords.length; i++) {
      if (!parent) break
      const record = newRecords[i]!
      if (stable.has(i)) {
        cursor = record.nodes[record.nodes.length - 1] ?? cursor
      } else {
        for (const n of record.nodes) {
          parent.insertBefore(n, cursor.nextSibling)
          cursor = n
        }
      }
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
