// Stub controller for the sui-counter reference. The buttons are wired
// via `on:` handlers that emit the canonical `change` event so external
// listeners get a typed payload. Writes happen via `state.value = …`
// (proxy routes through signal.set) — never `state.value.set(v)` after
// the #179 unification.

import { controller } from '@ship/sui-runtime'
import type { Scope } from './stub-artifact.sui'

export default controller<Scope>({
  on: {
    decrement(state, ev: Event) {
      const previous = state.value
      const next = Math.max(state.min, state.value - state.step)
      state.value = next
      ;(ev.currentTarget as HTMLElement)
        .getRootNode({ composed: true })
        ?.dispatchEvent(new CustomEvent('change', { detail: { value: next, previous }, bubbles: true, composed: true }))
    },
    increment(state, ev: Event) {
      const previous = state.value
      const next = Math.min(state.max, state.value + state.step)
      state.value = next
      ;(ev.currentTarget as HTMLElement)
        .getRootNode({ composed: true })
        ?.dispatchEvent(new CustomEvent('change', { detail: { value: next, previous }, bubbles: true, composed: true }))
    },
  },
})
