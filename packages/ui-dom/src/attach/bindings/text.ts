// Text-interpolation binding.
//
// Marker shape (see SSR walker / marker-contract ADR):
//   <!--sui:text:bN-->INLINED VALUE<!--/sui-->
//
// On wiring we collapse the existing inlined nodes between the open
// and close comments into a single Text node we control, then run an
// effect that writes the stringified expression value to it on every
// signal change.

import { effect } from '@ship/ui-runtime'
import { compileExpr } from '../eval'

/**
 * Replace the SSR'd text content between `open` and `close` with a
 * single managed Text node and subscribe it to `expr`. Returns a
 * disposer that tears down the effect.
 */
export function wireText(
  open: Comment,
  close: Comment,
  scope: Record<string, unknown>,
  expr: string,
): () => void {
  const parent = open.parentNode
  if (!parent) return () => {}
  // Collapse existing siblings between open and close into one Text.
  let s: Node | null = open.nextSibling
  while (s && s !== close) {
    const next: Node | null = s.nextSibling
    parent.removeChild(s)
    s = next
  }
  const text = (parent.ownerDocument ?? document).createTextNode('')
  parent.insertBefore(text, close)

  const ev = compileExpr(expr)
  const dispose = effect(() => {
    let v: unknown
    try {
      v = ev(scope)
    } catch (err) {
      console.error('[ship-set/attach] text expr threw:', err, '\nexpr:', expr)
      v = ''
    }
    text.data = stringify(v)
  })
  return dispose
}

function stringify(v: unknown): string {
  if (v === null || v === undefined) return ''
  return String(v)
}
