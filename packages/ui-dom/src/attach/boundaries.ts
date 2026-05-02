// Control-flow boundaries — `if` / `for` / `match` / `await`.
//
// **Phase 2** — pure client-side swap. The compiler emits a hoisted
// `__wireBoundary_b<N>` helper inside `setup(ctx)` per boundary plus a
// `__b<N>_attach: (target) => disposer` named property on the view's
// setup return. This module wires SSR'd `<sui-{kind} key="b<N>">`
// elements by dispatching to the matching handler — no server
// round-trip, no scope serialization. The handler swaps the SSR'd
// wrapper for a Comment anchor and runs `bindIf` / `bindFor` against
// the live signal layer; the first effect-run mounts the active branch
// synchronously, replacing the SSR'd content client-side.

const BOUNDARY_TAGS = new Set(['sui-if', 'sui-for', 'sui-match', 'sui-await'])

/**
 * Wire every `<sui-{if|for|match|await}>` boundary under `root` by
 * looking up `__b<N>_attach` on the setup return. Returns a single
 * disposer that tears down every wired boundary in reverse order.
 *
 * Order: outer→inner in document order. After wiring an outer
 * boundary, its inner SSR'd `<sui-{kind}>` children are detached
 * (replaced by client-rendered content with their own freshly-wired
 * inner boundaries via the hoisted helpers). The snapshot list still
 * holds references to detached elements; we skip those by checking
 * `Node.isConnected` before dispatching.
 */
export function wireBoundaries(
  root: Element,
  setupRet: Record<string, unknown>,
): () => void {
  const owner = root.ownerDocument ?? document
  const all: Element[] = []
  // Include the root itself if it's a boundary; querySelectorAll
  // omits the root by spec.
  if (BOUNDARY_TAGS.has(root.tagName.toLowerCase())) {
    all.push(root)
  }
  const list = root.querySelectorAll('sui-if, sui-for, sui-match, sui-await')
  for (const el of Array.from(list)) all.push(el)

  const cleanups: Array<() => void> = []
  for (const el of all) {
    if (!el.isConnected) continue
    const key = el.getAttribute('key')
    if (!key) continue
    const handlerName = '__' + key + '_attach'
    const handler = setupRet[handlerName] as
      | ((target: Element) => () => void)
      | undefined
    if (typeof handler !== 'function') {
      console.error(
        '[ship-set/attach] no boundary handler for ' +
          handlerName +
          ' on view; SSR\'d boundary left in place',
      )
      continue
    }
    try {
      const dispose = handler(el)
      if (typeof dispose === 'function') cleanups.push(dispose)
    } catch (err) {
      console.error(
        '[ship-set/attach] boundary handler threw for',
        key,
        err,
      )
    }
  }

  void owner

  return () => {
    for (let i = cleanups.length - 1; i >= 0; i--) {
      try {
        cleanups[i]!()
      } catch (err) {
        console.error('[ship-set/attach] boundary dispose threw:', err)
      }
    }
  }
}
