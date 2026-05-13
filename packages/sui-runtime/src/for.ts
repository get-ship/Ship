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
import { runProtected } from './error-boundary'
import { runEnter, runLeave } from './transition'

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

// `matched` packs each matched record's (newIndex, prevIndex) pair into
// adjacent Int32 slots: matched[2*i]=new-array index, matched[2*i+1]=prev-array
// index. mask/pred/tails are caller-owned scratch (length >= count). On
// return, mask[i]=1 iff matched record i is in the LIS of its prev positions.
// O(n log n) patience sort.
function computeLisMaskInto(
  matched: Int32Array, count: number,
  mask: Uint8Array, pred: Int32Array, tails: Int32Array,
): void {
  mask.fill(0, 0, count)
  if (count === 0) return
  let tailsLen = 0
  for (let i = 0; i < count; i++) {
    const v = matched[2 * i + 1]!
    let lo = 0, hi = tailsLen
    while (lo < hi) {
      const mid = (lo + hi) >>> 1
      if (matched[2 * tails[mid]! + 1]! < v) lo = mid + 1
      else hi = mid
    }
    pred[i] = lo > 0 ? tails[lo - 1]! : -1
    tails[lo] = i
    if (lo === tailsLen) tailsLen++
  }
  let idx = tails[tailsLen - 1]!
  while (idx >= 0) { mask[idx] = 1; idx = pred[idx]! }
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
  // Rows whose key disappeared from the items array and whose
  // out-transition is in flight. Tracked so the disposer can flush
  // them synchronously without waiting for animations.
  const departing = new Set<ItemRecord<T>>()

  // Scratch — allocated once at bindFor setup, .clear()'d each render so
  // reconciliation reuses a constant working set instead of allocating
  // O(N) transient collections per effect run.
  const nextKeys = new Map<unknown, number>() // key → first-seen index
  const prevByKey = new Map<unknown, ItemRecord<T>>()
  const prevPos = new Map<unknown, number>()
  const stable = new Set<number>()
  const freshNodes: Node[] = []
  let nextRecords: ItemRecord<T>[] = []
  let matched = new Int32Array(0)        // [newIdx, prevIdx] pairs
  let lisMaskBuf = new Uint8Array(0), lisPred = new Int32Array(0), lisTails = new Int32Array(0)

  const flush = (r: ItemRecord<T>) => {
    for (const n of r.nodes) {
      try {
        n.parentNode?.removeChild(n)
      } catch {
        // Already detached.
      }
    }
    r.cleanup?.()
  }

  const dispose = effect(() => {
    if (!active) return
    const next = items()
    const n = next.length

    // Grow scratch typed arrays to the new high-water mark; never shrink.
    if (matched.length < n * 2) matched = new Int32Array(Math.max(matched.length * 2, n * 2, 16))
    if (lisMaskBuf.length < n) {
      const cap = Math.max(lisMaskBuf.length * 2, n, 8)
      lisMaskBuf = new Uint8Array(cap); lisPred = new Int32Array(cap); lisTails = new Int32Array(cap)
    }

    for (let i = 0; i < n; i++) {
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
          nextKeys.get(k),
          '); the earlier item will be dropped from the rendered output.',
          'Fix the key function so each item maps to a unique value.',
        )
      }
      nextKeys.set(k, i)
    }

    for (const r of prev) prevByKey.set(r.key, r)

    // 1. Remove items that are gone in the next array. Rows with an
    //    out-transition stay in DOM until the animation resolves; the
    //    LIS reposition below skips them naturally since they're
    //    absent from `nextRecords`. No transition → synchronous flush
    //    (preserves the historical non-animated behaviour).
    for (const r of prev) {
      if (nextKeys.has(r.key)) continue
      const leaving = runLeave(r.nodes)
      if (leaving === null) {
        flush(r)
        continue
      }
      departing.add(r)
      leaving.then(() => {
        if (!departing.delete(r)) return
        flush(r)
      })
    }

    // 2. Build new records list — reuse matched keys, render new ones.
    //    No DOM mutations in this pass.
    nextRecords.length = 0
    freshNodes.length = 0
    for (let i = 0; i < n; i++) {
      const item = next[i]!
      const k = key(item, i)
      const existing = prevByKey.get(k)
      let record: ItemRecord<T>
      if (existing) {
        existing.item = item
        runProtected(anchor, 'render', () => existing.update?.(item, i))
        record = existing
      } else {
        const r = runProtected(anchor, 'render', () => render(item, i))
        if (!r) {
          // Render threw and was caught by a boundary (or logged when
          // no boundary is in scope). Skip this item: the boundary
          // will unmount the subtree shortly.
          continue
        }
        record = {
          key: k,
          item,
          nodes: r.nodes,
          ...(r.cleanup && { cleanup: r.cleanup }),
          ...(r.update && { update: r.update }),
        }
        freshNodes.push(...r.nodes)
      }
      nextRecords.push(record)
    }

    // 3. Find matched records already in a stable relative order (LIS of
    //    their previous-array positions). These records don't need DOM
    //    moves — only non-LIS and new records are repositioned.
    for (let i = 0; i < prev.length; i++) prevPos.set(prev[i]!.key, i)

    let matchedCount = 0
    for (let i = 0; i < nextRecords.length; i++) {
      const p = prevPos.get(nextRecords[i]!.key)
      if (p !== undefined) {
        matched[2 * matchedCount] = i
        matched[2 * matchedCount + 1] = p
        matchedCount++
      }
    }

    computeLisMaskInto(matched, matchedCount, lisMaskBuf, lisPred, lisTails)
    for (let j = 0; j < matchedCount; j++) {
      if (lisMaskBuf[j]) stable.add(matched[2 * j]!)
    }

    // 4. Place all records. Stable records advance the cursor without any
    //    DOM mutation. Unstable / new records are moved via insertBefore.
    const parent = anchor.parentNode
    let cursor: Node = anchor
    for (let i = 0; i < nextRecords.length; i++) {
      if (!parent) break
      const record = nextRecords[i]!
      if (stable.has(i)) {
        cursor = record.nodes[record.nodes.length - 1] ?? cursor
      } else {
        for (const node of record.nodes) {
          parent.insertBefore(node, cursor.nextSibling)
          cursor = node
        }
      }
    }

    // Swap buffers: `nextRecords` becomes the new prev; the old prev
    // (now stale) becomes the empty buffer for the next render.
    const tmp = prev
    prev = nextRecords
    nextRecords = tmp

    if (freshNodes.length > 0) runEnter(freshNodes)

    // Drop scratch contents so we don't pin references (records, items,
    // keys) between renders. The Maps/Sets themselves are reused.
    nextKeys.clear()
    prevByKey.clear()
    prevPos.clear()
    stable.clear()
    freshNodes.length = 0
  })

  return () => {
    active = false
    dispose()
    for (const r of prev) flush(r)
    // Departing rows: hard teardown skips out-animations.
    for (const d of departing) flush(d)
    departing.clear()
    prev = []
    nextRecords.length = 0
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
