// Text-interpolation hydrator.
//
// Direct-emit shape: the compiler walks to a `<!--sui:text-->` open
// comment and passes it in. We collapse the SSR'd nodes between this
// open comment and its sibling `<!--/sui-->` close into a single Text
// node we own, then run an effect that writes the stringified getter
// value to it on every signal change.
//
// `getter` is a closure the compiler emitted from the IR expression
// against the captured scope — no string-eval at runtime.

import { effect } from '@ship/ui-runtime'

const TEXT_CLOSE = '/sui'

export function hydrateText(open: Comment, getter: () => unknown): () => void {
  const parent = open.parentNode
  if (!parent) return () => {}
  let close: Comment | null = null
  for (let s: Node | null = open.nextSibling; s; s = s.nextSibling) {
    if (s.nodeType === 8 && (s as Comment).data === TEXT_CLOSE) {
      close = s as Comment
      break
    }
  }
  if (!close) return () => {}
  let s: Node | null = open.nextSibling
  while (s && s !== close) {
    const next: Node | null = s.nextSibling
    parent.removeChild(s)
    s = next
  }
  const text = (parent.ownerDocument ?? document).createTextNode('')
  parent.insertBefore(text, close)
  return effect(() => {
    const v = getter()
    text.data = v == null ? '' : String(v)
  })
}
