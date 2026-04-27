// Light-DOM slot resolution.
//
// Sail components use light DOM (no shadow root). Native <slot> elements
// don't distribute children in light DOM, so the runtime walks the
// host's children and places them at slot anchors at hydrate time.
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Slots.

export interface SlotDecl {
  /** Slot name; default slot is `'default'`. */
  name: string
  /** Anchor element (or comment) where slot content goes. */
  anchor: Comment
  /** Render fallback if no fill provided. Returns nodes to insert. */
  fallback?: () => Node[]
}

/**
 * Resolve slot fills against the host's pre-existing nodes. Walks
 * the snapshot, routes Elements with `slot="X"` to the matching
 * named anchor, and routes everything else (Elements without a slot
 * attribute, Text, Comment) to the default anchor. Mirrors native
 * `<slot>` distribution semantics so text children like
 * `<my-component>Active</my-component>` aren't dropped.
 *
 * Should be called AFTER the component's template has been rendered
 * (so anchors exist) but before lifecycle hooks fire (so authors see
 * a fully-distributed tree).
 */
export function resolveSlots(host: Element, slots: SlotDecl[], originalChildren: Node[]): void {
  const fills = new Map<string, Node[]>()
  for (const child of originalChildren) {
    let name = 'default'
    if (child instanceof Element) {
      name = child.getAttribute('slot') ?? 'default'
      if (child.hasAttribute('slot')) child.removeAttribute('slot')
    }
    const list = fills.get(name) ?? []
    list.push(child)
    fills.set(name, list)
  }

  for (const slot of slots) {
    const filled = fills.get(slot.name) ?? []
    const parent = slot.anchor.parentNode
    if (!parent) continue
    if (filled.length > 0) {
      for (const n of filled) parent.insertBefore(n, slot.anchor)
    } else if (slot.fallback) {
      const nodes = slot.fallback()
      for (const n of nodes) parent.insertBefore(n, slot.anchor)
    }
  }
  for (const slot of slots) slot.anchor.remove()
}

/**
 * Snapshot the host's current child *nodes* (not just elements)
 * before innerHTML is replaced. Text and comment children survive
 * the round-trip — native slots distribute them, and so should we.
 */
export function snapshotChildren(host: Element): Node[] {
  return Array.from(host.childNodes)
}
