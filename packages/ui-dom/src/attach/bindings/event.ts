// Event-handler binding (`on:<event>`).
//
// Resolved at attach time: the expression evaluates to a function in
// scope (e.g. an `on:` handler the controller exposed). We wrap it in
// a stable listener so subsequent re-evaluations (none in v1) don't
// break addEventListener identity. v1 evaluates the expression once
// at wire time — `on:` handlers don't reactively re-bind.

import { compileExpr } from '../eval'

/**
 * Attach a listener for `eventName` on `el`. Each fire re-evaluates
 * the expression to fetch the current handler and invokes it. This
 * lets handlers swap when the underlying signal changes (rare, but
 * preserves the property authors expect when an `on:` value is
 * conditional).
 */
export function wireOn(
  el: Element,
  eventName: string,
  scope: Record<string, unknown>,
  expr: string,
): () => void {
  const ev = compileExpr(expr)
  const listener = (e: Event): void => {
    let handler: unknown
    try {
      handler = ev(scope)
    } catch (err) {
      console.error('[ship-set/attach] on:' + eventName + ' expr threw:', err, '\nexpr:', expr)
      return
    }
    if (typeof handler !== 'function') return
    try {
      ;(handler as (e: Event) => unknown)(e)
    } catch (err) {
      console.error('[ship-set/attach] on:' + eventName + ' handler threw:', err)
    }
  }
  el.addEventListener(eventName, listener)
  return () => el.removeEventListener(eventName, listener)
}
