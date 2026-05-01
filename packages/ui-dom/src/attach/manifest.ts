// Marker walker.
//
// Walks SSR'd DOM under `root` and yields one `MarkerHit` per marker.
// Three marker shapes (post SSR re-cutover):
//
//   1. Author-syntax attribute markers: `on:click="bN"`, `bind:value="bN"`,
//      `class:active="bN"`, `style:color="bN"`, `attr:href="bN"`,
//      `attr-dynamic="bN"` — on the author's own element.
//   2. Control-flow boundary elements: `<sui-if key="bN">`,
//      `<sui-for key="bN">`, `<sui-match key="bN">` — read the `key` attr.
//   3. Text comment markers: `<!--sui:text:bN-->value<!--/sui-->`.

import type { Binding } from '../attach'

const TEXT_OPEN_PREFIX = 'sui:text:'
const TEXT_CLOSE = '/sui'

const BOUNDARY_TAGS = new Set(['sui-if', 'sui-for', 'sui-match'])

/** A resolved marker. */
export type MarkerHit =
  | {
      kind: 'element'
      binding: Binding
      el: Element
      /** Raw attribute name (e.g. `on:click`, `bind:value`, or `key` for boundaries). */
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

/** Options for `walkMarkers`. */
export interface WalkMarkersOptions {
  /** When true, do not yield markers on `root` itself — only descendants.
   *  Used by Phase E-2's `wireBoundary` so re-entering wireSubtree from
   *  inside a boundary doesn't double-wire the boundary's own marker. */
  skipRoot?: boolean
}

/**
 * Yield every marker under `root`, in document order.
 *
 * Two passes:
 *   1. SHOW_ELEMENT — every element including `root` (unless
 *      `opts.skipRoot` is set). For each element:
 *      - If tag is `sui-if`, `sui-for`, or `sui-match`: read `key` attr,
 *        look up in manifest, yield an `element` hit.
 *      - Otherwise: scan attributes for author-syntax binding names
 *        (`on:*`, `bind:*`, `class:*`, `style:*`, `attr:*`, `attr-dynamic`),
 *        look up the attribute value in the manifest, yield an `element` hit.
 *   2. SHOW_COMMENT — every `<!--sui:text:bN-->` opener gets paired
 *      with its sibling `<!--/sui-->` closer; yields a `text` hit.
 *
 * Bindings whose key isn't in the manifest are silently skipped —
 * the runtime is tolerant of stale HTML.
 */
export function* walkMarkers(
  root: HTMLElement,
  byKey: Map<string, Binding>,
  opts: WalkMarkersOptions = {},
): Generator<MarkerHit> {
  const owner = root.ownerDocument ?? document
  const elWalker = owner.createTreeWalker(root, NodeFilter.SHOW_ELEMENT)
  let el: Element | null = elWalker.currentNode as Element
  if (opts.skipRoot && el === root) {
    el = elWalker.nextNode() as Element | null
  }
  while (el !== null) {
    const tag = el.tagName.toLowerCase()
    if (BOUNDARY_TAGS.has(tag)) {
      const keyVal = el.getAttribute('key')
      if (keyVal) {
        const binding = byKey.get(keyVal)
        if (binding) {
          yield { kind: 'element', binding, el, rawAttr: 'key' }
        }
      }
    } else {
      for (const attr of Array.from(el.attributes)) {
        if (!isBindingAttr(attr.name)) continue
        const binding = byKey.get(attr.value)
        if (!binding) continue
        yield { kind: 'element', binding, el, rawAttr: attr.name }
      }
    }
    el = elWalker.nextNode() as Element | null
  }

  // SHOW_COMMENT: snapshot the opens first so DOM mutations during
  // wiring don't perturb the walk.
  const cWalker = owner.createTreeWalker(root, NodeFilter.SHOW_COMMENT)
  const opens: Comment[] = []
  let n: Node | null = cWalker.currentNode
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

/** True for attribute names that the SSR walker uses as binding markers. */
function isBindingAttr(name: string): boolean {
  return (
    name.startsWith('on:') ||
    name.startsWith('bind:') ||
    name.startsWith('class:') ||
    name.startsWith('style:') ||
    name.startsWith('attr:') ||
    name === 'attr-dynamic' ||
    name === 'mutation'
  )
}
