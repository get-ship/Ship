// `class:NAME={cond}` — toggles a class on truthy / off on falsy.

import { effect } from '@ship/ui-runtime'
import { compileExpr } from '../eval'

export function wireClass(
  el: Element,
  name: string,
  scope: Record<string, unknown>,
  expr: string,
): () => void {
  const ev = compileExpr(expr)
  return effect(() => {
    let v: unknown
    try {
      v = ev(scope)
    } catch (err) {
      console.error('[ship-set/attach] class:' + name + ' expr threw:', err)
      return
    }
    el.classList.toggle(name, !!v)
  })
}
