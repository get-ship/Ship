// Single-pass scan for every author-syntax binding marker attribute
// (`on:*`, `attr:*`, `attr-dynamic`, `class:*`, `style:*`, `bind:*`)
// under `root`. Returns a map from binding key (`bN`) → the SSR'd
// element carrying the marker.
//
// The compiler-emitted `hydrate(root, ctx)` calls this once per attach
// and dispatches each element-bound leaf binding by key — one tree walk
// for the whole view rather than one per binding. Mirrors the
// `collectTextOpens` pattern used for text bindings.

export function collectElementMarkers(root: Node): Record<string, Element> {
  const owner = (root as Element).ownerDocument ?? document
  const walker = owner.createTreeWalker(root, 1 /* SHOW_ELEMENT */)
  const out: Record<string, Element> = Object.create(null)
  let n: Node | null = walker.currentNode
  while ((n = walker.nextNode())) {
    const el = n as Element
    for (const attr of Array.from(el.attributes)) {
      const name = attr.name
      if (
        name.startsWith('on:') ||
        name.startsWith('attr:') ||
        name === 'attr-dynamic' ||
        name.startsWith('class:') ||
        name.startsWith('style:') ||
        name.startsWith('bind:')
      ) {
        out[attr.value] = el
      }
    }
  }
  return out
}
