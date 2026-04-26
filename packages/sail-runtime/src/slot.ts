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
 * Resolve slot fills against the host's pre-existing children. Walks
 * `host.children`, finds elements with `slot="X"`, distributes them
 * to matching anchors. Default slot collects children with no slot
 * attribute.
 *
 * Should be called AFTER the component's template has been rendered
 * (so anchors exist) but before lifecycle hooks fire (so authors see
 * a fully-distributed tree).
 */
export function resolveSlots(host: Element, slots: SlotDecl[], originalChildren: Element[]): void {
  // Group originals by slot name.
  const fills = new Map<string, Element[]>()
  for (const child of originalChildren) {
    const name = child.getAttribute('slot') ?? 'default'
    const list = fills.get(name) ?? []
    list.push(child)
    fills.set(name, list)
    // Strip the slot attribute now — it served its routing purpose.
    if (child.hasAttribute('slot')) child.removeAttribute('slot')
  }

  for (const slot of slots) {
    const filled = fills.get(slot.name) ?? []
    const parent = slot.anchor.parentNode
    if (!parent) continue
    if (filled.length > 0) {
      for (const el of filled) parent.insertBefore(el, slot.anchor)
    } else if (slot.fallback) {
      const nodes = slot.fallback()
      for (const n of nodes) parent.insertBefore(n, slot.anchor)
    }
  }
  // Remove the anchor placeholders themselves.
  for (const slot of slots) slot.anchor.remove()
}

/**
 * Snapshot the host's current children before innerHTML is replaced.
 * Returns an array the caller passes to `resolveSlots` after rendering.
 *
 * The component wrapper calls this in connectedCallback BEFORE setting
 * innerHTML so the original light-DOM children can be distributed
 * after the new template is in place.
 */
export function snapshotChildren(host: Element): Element[] {
  return Array.from(host.children)
}
