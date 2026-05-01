// `style:NAME={value}` — sets a single inline-style property.
//
// Falsy values clear the property (mirrors how the SSR walker omits
// the style fragment when its expression is falsy).

import { effect } from '@ship/ui-runtime'
import { compileExpr } from '../eval'

export function wireStyle(
  el: HTMLElement,
  propName: string,
  scope: Record<string, unknown>,
  expr: string,
): () => void {
  const ev = compileExpr(expr)
  return effect(() => {
    let v: unknown
    try {
      v = ev(scope)
    } catch (err) {
      console.error('[ship-set/attach] style:' + propName + ' expr threw:', err)
      return
    }
    if (v === null || v === undefined || v === false || v === '') {
      el.style.removeProperty(propName)
      return
    }
    el.style.setProperty(propName, String(v))
  })
}
