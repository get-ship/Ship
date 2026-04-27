// Conditional render. Sail's compiler turns
// `{#if cond}then{:else}else{/if}` into a call to bindIf.
//
// Anchor-based: a Comment placeholder marks the insertion point;
// branches are rendered lazily and swapped on cond change. Then- and
// else-branches each get their own dispose to tear down their bindings
// when swapped out (avoiding leaks from the inactive branch's
// effects).
//
// See docs/sail/03-runtime-architecture.md § Conditional render.

import { effect } from './signal'
import type { Disposer } from './bind'

export interface BranchRender {
  nodes: Node[]
  cleanup?: () => void
}

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
