// Conditional render. The ship-set compiler turns
// `{#if cond}then{:else}else{/if}` into a call to bindIf.
//
// Anchor-based: a Comment placeholder marks the insertion point;
// branches are rendered lazily and swapped on cond change. Then- and
// else-branches each get their own dispose to tear down their bindings
// when swapped out (avoiding leaks from the inactive branch's
// effects).
//
// See docs/ship-set/03-runtime-architecture.md § Conditional render.

import { effect } from './signal'
import type { Disposer } from './bind'

/**
 * One branch's rendered nodes plus an optional teardown. Returned by
 * the `then_` / `else_` / `loading` / etc. callbacks of every
 * branching binding (`bindIf`, `bindAwait`).
 */
export interface BranchRender {
  /** Top-level nodes the branch contributes to the tree. */
  nodes: Node[]
  /** Optional teardown — called when the branch is swapped out. */
  cleanup?: () => void
}

/** Inputs to `bindIf`. */
export interface BindIfOptions {
  /** Anchor placeholder — the active branch's nodes go BEFORE it. */
  anchor: Comment
  /** Reactive predicate. */
  cond: () => boolean
  /** Render the then-branch on demand. */
  then_: () => BranchRender
  /** Render the else-branch on demand (omit for then-only). */
  else_?: () => BranchRender
}

/**
 * Conditionally render a then-branch (and optional else-branch) at
 * `opts.anchor`. Branches are rendered lazily — `then_()` / `else_()`
 * are called only when their branch becomes active. The previous
 * branch's `cleanup` runs on swap, freeing its bindings.
 *
 * Compiler target: `{#if cond}then{:else}else{/if}`.
 *
 * @param opts Anchor, predicate, and branch renderers.
 * @returns Disposer that tears down the active branch and stops the effect.
 *
 * @example
 * ```ts
 * const open = signal(false)
 * bindIf({
 *   anchor,
 *   cond: () => open(),
 *   then_: () => ({ nodes: [renderDialog()] }),
 *   else_: () => ({ nodes: [] }),
 * })
 * ```
 *
 * @see bindShow when the element should remain mounted (just hidden);
 *   bindAwait for promise-driven branching.
 */
export function bindIf(opts: BindIfOptions): Disposer {
  const { anchor, cond, then_, else_ } = opts
  let active: BranchRender | null = null
  let lastBranch: 'then' | 'else' | null = null

  const tearDown = () => {
    if (!active) return
    for (const n of active.nodes) {
      try {
        n.parentNode?.removeChild(n)
      } catch {
        // already detached
      }
    }
    active.cleanup?.()
    active = null
  }

  const dispose = effect(() => {
    const want: 'then' | 'else' = cond() ? 'then' : 'else'
    if (want === lastBranch) return
    tearDown()
    lastBranch = want
    if (want === 'then') {
      active = then_()
    } else if (else_) {
      active = else_()
    } else {
      active = null
      return
    }
    const parent = anchor.parentNode
    if (!parent) return
    for (const n of active.nodes) {
      parent.insertBefore(n, anchor)
    }
  })

  return () => {
    dispose()
    tearDown()
  }
}
