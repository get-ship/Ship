// Marker walker.
//
// Walks SSR'd DOM under `root` and yields one `MarkerHit` per
// `data-set-*` attribute or `<!--set:text:bN-->` comment pair. Each
// hit carries the resolved Binding from the manifest and the host
// (Element for attribute markers, comment-pair + parent for text).
//
// `data-set-key` (the {#for} iteration key) is intentionally skipped —
// it's data-bearing, not a binding marker.

import type { Binding } from '../attach'

const ATTR_PREFIX = 'data-set-'
const TEXT_OPEN_PREFIX = 'set:text:'
const TEXT_CLOSE = '/set'

/** A resolved marker. */
export type MarkerHit =
  | {
      kind: 'element'
      binding: Binding
      el: Element
      /** Raw attribute name (e.g. `data-set-on:click`). */
      rawAttr: string
    }
  | {
      kind: 'text'
      binding: Binding
      open: Comment
      close: Comment
      parent: Element
    }

/** Build a fast lookup keyed by `bN`. */
export function indexBindings(bindings: Binding[]): Map<string, Binding> {
  const map = new Map<string, Binding>()
  for (const b of bindings) map.set(b.key, b)
  return map
}

/**
 * Yield every marker under `root`, in document order.
 *
 * Two passes:
 *   1. SHOW_ELEMENT — every element including `root`. For each
 *      `data-set-*` attribute (other than `data-set-key`), look up
 *      the binding by attribute value and yield an `element` hit.
 *   2. SHOW_COMMENT — every `<!--set:text:bN-->` opener gets paired
 *      with its sibling `<!--/set-->` closer; yields a `text` hit.
 *
 * Bindings whose key isn't in the manifest are silently skipped —
 * the runtime is tolerant of stale HTML, and lookups fail loudly
 * elsewhere.
 */
export function* walkMarkers(
  root: HTMLElement,
  byKey: Map<string, Binding>,
): Generator<MarkerHit> {
  const owner = root.ownerDocument ?? document
  // SHOW_ELEMENT: include root itself.
  const elWalker = owner.createTreeWalker(root, NodeFilter.SHOW_ELEMENT)
  let el: Element | null = elWalker.currentNode as Element
  while (el !== null) {
    for (const attr of Array.from(el.attributes)) {
      if (!attr.name.startsWith(ATTR_PREFIX)) continue
      if (attr.name === 'data-set-key') continue
      const binding = byKey.get(attr.value)
      if (!binding) continue
      yield { kind: 'element', binding, el, rawAttr: attr.name }
    }
    el = elWalker.nextNode() as Element | null
  }

  // SHOW_COMMENT: snapshot the opens first so DOM mutations during
  // wiring (none today, but future-proof) don't perturb the walk.
  const cWalker = owner.createTreeWalker(root, NodeFilter.SHOW_COMMENT)
  const opens: Comment[] = []
  let n: Node | null = cWalker.currentNode
  // currentNode at the start is `root`, which is an element, so
  // SHOW_COMMENT skips it on the first call to nextNode(). Use that.
  while ((n = cWalker.nextNode())) {
    if (!isComment(n)) continue
    if (n.data.startsWith(TEXT_OPEN_PREFIX)) opens.push(n)
  }

  for (const open of opens) {
    const key = open.data.slice(TEXT_OPEN_PREFIX.length)
    const binding = byKey.get(key)
    if (!binding) continue
    let close: Comment | null = null
    for (let s: Node | null = open.nextSibling; s; s = s.nextSibling) {
      if (isComment(s) && s.data === TEXT_CLOSE) {
        close = s
        break
      }
    }
    if (!close) continue
    const parent = open.parentElement
    if (!parent) continue
    yield { kind: 'text', binding, open, close, parent }
  }
}

function isComment(n: Node): n is Comment {
  return n.nodeType === 8 /* COMMENT_NODE */
}
