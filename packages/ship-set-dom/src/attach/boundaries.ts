// Control-flow boundaries — `if` / `for` / `match` / `await`.
//
// Phase E v1 scope: recognise the boundary, subscribe to the
// condition signal so changes don't go silent (warn the user that
// re-rendering the active branch isn't wired yet), and recurse into
// the active branch so its leaf bindings get wired. Active wrappers
// are `<div data-set-{kind}="bN" style="display:contents">…</div>`;
// placeholders are `<template data-set-{kind}="bN"></template>`.
//
// **Phase E-2** lands swap-on-signal-change: a placeholder activates
// by rendering the matching branch from the compiled module, an
// active branch deactivates by tearing the wrapper down. The renderer
// for the un-rendered branches needs a fragment factory the compiler
// will emit — see `Notes for the agent` in the Phase E job spec.
//
// Until then: the warning is the user-visible signal.

import { effect } from '@ship-set/runtime'
import type { Binding } from '../attach'
import { compileExpr } from './eval'

/**
 * Wire a control-flow boundary marker.
 *
 * `recurse` is the per-element wirer the caller uses to recurse into
 * the active branch (`<div>` only — placeholders' content lives in
 * `template.content` and isn't part of the live DOM walk).
 */
export function wireBoundary(
  el: Element,
  binding: Binding,
  scope: Record<string, unknown>,
  recurse: (el: Element) => () => void,
): () => void {
  const cleanups: Array<() => void> = []
  const isActive = el.tagName.toLowerCase() === 'div'

  // Subscribe to the condition signal so a change fires the warning.
  if (binding.expr.length > 0) {
    const ev = compileExpr(binding.expr)
    let firstRun = true
    const dispose = effect(() => {
      try {
        ev(scope)
      } catch {
        // expr might not be evaluatable on first run if the scope
        // hasn't fully resolved — silently swallow; deps still register.
      }
      if (firstRun) {
        firstRun = false
        return
      }
      console.warn(
        '[ship-set/attach] control-flow signal changed for binding ' +
          binding.key +
          ' (kind=' +
          binding.kind +
          '). Re-rendering active control-flow boundaries is Phase E-2; for now reload the page to see the new branch. (expr: ' +
          binding.expr +
          ')',
      )
    })
    cleanups.push(dispose)
  }

  if (isActive) {
    cleanups.push(recurse(el))
  }
  // Placeholder (`<template>`) has no live children to wire.

  return () => {
    for (const c of cleanups) {
      try {
        c()
      } catch (err) {
        console.error('[ship-set/attach] boundary cleanup threw:', err)
      }
    }
  }
}
