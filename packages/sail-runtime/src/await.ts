// Promise-driven branch render. Sail's compiler turns
// `{#await PROMISE}loading{:then VAL}then{:catch ERR}catch{/await}`
// into a call to bindAwait.
//
// Loading branch shows immediately; resolution swaps to then/catch.
// If the promise expression returns a NEW promise (e.g., the dep
// changes), the previous active branch is torn down and the loading
// branch renders again.

import { effect } from './signal'
import type { Disposer } from './bind'
import type { BranchRender } from './if'

export interface BindAwaitOptions {
  anchor: Comment
  /** Re-evaluated reactively; each new Promise triggers a fresh cycle. */
  promise: () => Promise<unknown>
  loading: () => BranchRender
  then_?: (value: unknown) => BranchRender
  catch_?: (err: unknown) => BranchRender
}

export function bindAwait(opts: BindAwaitOptions): Disposer {
  const { anchor, promise, loading, then_, catch_ } = opts
  let active: BranchRender | null = null
  let token = 0

  const tearDown = () => {
    if (!active) return
    for (const n of active.nodes) {
      try { n.parentNode?.removeChild(n) } catch {}
    }
    active.cleanup?.()
    active = null
  }

  const insert = (br: BranchRender) => {
    active = br
    const parent = anchor.parentNode
    if (!parent) return
    for (const n of br.nodes) parent.insertBefore(n, anchor)
  }

  const dispose = effect(() => {
    const myToken = ++token
    const p = promise()
    tearDown()
    insert(loading())
    p.then(
      (v) => {
        if (myToken !== token) return
        if (then_) {
          tearDown()
          insert(then_(v))
        }
      },
      (e) => {
        if (myToken !== token) return
        if (catch_) {
          tearDown()
          insert(catch_(e))
        }
      },
    )
  })

  return () => {
    token++ // invalidate any in-flight resolutions
    dispose()
    tearDown()
  }
}
