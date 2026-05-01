// `attr:NAME={expr}` — reactive attribute value.
//
// Effect re-evaluates the expression on dep change and writes the
// stringified result via `setAttribute`. A nullish result removes
// the attribute (so authors can flip an attribute on/off via the
// signal value).

import { effect } from '@ship/ui-runtime'
import { compileExpr } from '../eval'

export function wireAttr(
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
      console.error('[ship-set/attach] attr:' + name + ' expr threw:', err)
      return
    }
    if (v === null || v === undefined || v === false) {
      el.removeAttribute(name)
      return
    }
    el.setAttribute(name, String(v))
  })
}
