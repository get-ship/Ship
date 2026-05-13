// Conditional render. The ship-set compiler turns
// `{#if cond}then{:elif …}{:else}else{/if}` (and `{#match}`) into a
// call to bindIf.
//
// Anchor-based: a Comment placeholder marks the insertion point;
// branches are rendered lazily and swapped on selector change. Each
// branch gets its own dispose to tear down its bindings when swapped
// out (avoiding leaks from inactive branches' effects).
//
// Two input shapes are accepted, both ending in the same swap engine:
//
//   - Two-branch (the original API): `{ anchor, cond, then_, else_ }`.
//     `cond()` returns a boolean. Lives on for hand-rolled callers.
//
//   - Multi-branch (compiler emit): `{ anchor, test, branches }`.
//     `test()` returns the integer index of the branch to render
//     (`-1` selects nothing). The compiler emits this for `{#if}` —
//     including elif chains — and for `{#match}`. The else-branch (or
//     match's `default`) is appended to `branches` so the test
//     function can return its index instead of a sentinel.
//
// See docs/ship-set/03-runtime-architecture.md § Conditional render.

import { effect } from './signal'
import type { Disposer } from './bind'
import { runProtected } from './error-boundary'

/**
 * One branch's rendered nodes plus an optional teardown. Returned by
 * the `then_` / `else_` / `loading` / branch-render callbacks of every
 * branching binding (`bindIf`, `bindAwait`).
 */
export interface BranchRender {
  /** Top-level nodes the branch contributes to the tree. */
  nodes: Node[]
  /** Optional teardown — called when the branch is swapped out. */
  cleanup?: () => void
}

/** Two-branch input shape — the original hand-author API. */
export interface BindIfTwoBranchOptions {
  anchor: Comment
  cond: () => boolean
  then_: () => BranchRender
  else_?: () => BranchRender
}

/**
 * Multi-branch input shape — the compiler emit for `{#if}` /
 * `{#match}`. `test()` returns the index of the branch in `branches`
 * to render, or `-1` for "render nothing".
 */
export interface BindIfMultiBranchOptions {
  anchor: Comment
  test: () => number
  branches: Array<() => BranchRender>
}

/** Public union — `bindIf` accepts either shape. */
export type BindIfOptions = BindIfTwoBranchOptions | BindIfMultiBranchOptions

/**
 * Conditionally render one of N branches at `opts.anchor`. Branches
 * are rendered lazily — only the selected branch's renderer is
 * invoked. The previous branch's `cleanup` runs on swap, freeing its
 * bindings.
 *
 * Compiler target: `{#if cond}then{:elif …}{:else}else{/if}` and
 * `{#match}`. The compiler always passes the multi-branch shape; the
 * two-branch shape is preserved for hand-rolled callers.
 *
 * @returns Disposer that tears down the active branch and stops the effect.
 *
 * @example two-branch (hand-rolled):
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
 * @example multi-branch (compiler emit):
 * ```ts
 * bindIf({
 *   anchor,
 *   test: () => kind() === 'ok' ? 0 : kind() === 'loading' ? 1 : -1,
 *   branches: [() => renderOk(), () => renderLoading()],
 * })
 * ```
 *
 * @see bindShow when the element should remain mounted (just hidden);
 *   bindAwait for promise-driven branching.
 */
export function bindIf(opts: BindIfOptions): Disposer {
  const { anchor } = opts
  // Normalize both shapes into a single (test, branches) pair so the
  // swap engine has one path. The two-branch shape collapses to
  // branches: [then_, else_?]; cond() → 0 picks then, 1 (when else_
  // exists) picks else, otherwise -1.
  const { test, branches } = isMulti(opts)
    ? { test: opts.test, branches: opts.branches }
    : twoBranchToMulti(opts)

  let active: BranchRender | null = null
  let lastIdx = -2 // sentinel: not -1 (which is a real "no match")

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
    const idx = test()
    if (idx === lastIdx) return
    tearDown()
    lastIdx = idx
    if (idx < 0 || idx >= branches.length) {
      active = null
      return
    }
    const rendered = runProtected(anchor, 'render', () => branches[idx]!())
    if (!rendered) {
      active = null
      return
    }
    active = rendered
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

function isMulti(o: BindIfOptions): o is BindIfMultiBranchOptions {
  return typeof (o as BindIfMultiBranchOptions).test === 'function'
}

function twoBranchToMulti(
  o: BindIfTwoBranchOptions,
): { test: () => number; branches: Array<() => BranchRender> } {
  const { cond, then_, else_ } = o
  const branches: Array<() => BranchRender> = else_ ? [then_, else_] : [then_]
  return {
    test: () => (cond() ? 0 : else_ ? 1 : -1),
    branches,
  }
}
