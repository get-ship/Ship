// `attr-dynamic` — attributes whose **name** is reactive.
//
// The IR shape is `{name_expr, value_expr}` but the binding manifest
// only carries one `expr` (the value). v1 cannot reconstruct the
// name expression from the manifest alone, so this wirer subscribes
// to the deps for parity with the contract and warns once per node.
// Phase E-2 will land manifest-side support (name expr + value expr
// emitted as a tuple) and finish the swap-on-name-change path.

import { effect } from '@ship-set/runtime'
import { compileExpr } from '../eval'

export function wireAttrDynamic(
  el: Element,
  scope: Record<string, unknown>,
  expr: string,
): () => void {
  const ev = compileExpr(expr)
  let warned = false
  return effect(() => {
    try {
      ev(scope)
    } catch {
      // ignore — the value expression is informational in v1
    }
    if (!warned) {
      warned = true
      console.warn(
        '[ship-set/attach] attr-dynamic on <' +
          el.tagName.toLowerCase() +
          '> recognised but not yet wired (Phase E-2). The SSR-rendered attribute is preserved.',
      )
    }
  })
}
