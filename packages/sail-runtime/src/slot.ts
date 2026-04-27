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

  // Track which slots received caller-provided content (vs. fell back
  // to defaults). Read by `slotFilled(host, name)` and the compiler-
  // injected local of the same name. Non-empty meaningful nodes only —
  // pure-whitespace text fills count as "not filled" (they would
  // otherwise mark every slot with leading whitespace as filled).
  const filledNames = new Set<string>()
  for (const [name, nodes] of fills) {
    const hasMeaning = nodes.some((n) => {
      if (n.nodeType === 1) return true
      if (n.nodeType === 3) return (n.textContent || '').trim().length > 0
      return false
    })
    if (hasMeaning) filledNames.add(name)
  }
  ;(host as unknown as { __sailSlotFilled: Set<string> }).__sailSlotFilled = filledNames

  for (const slot of slots) {
    // Slot anchors inside conditional branches (e.g. `{#if x}<slot
    // name="y"/>{/if}`) aren't in the DOM at mount time. Tolerate
    // undefined anchors here so the rest of the slot wiring still
    // resolves; the missing slot becomes effectively hidden until
    // its branch renders.
    if (!slot || !slot.anchor) continue
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
  for (const slot of slots) {
    if (slot && slot.anchor) slot.anchor.remove()
  }
}

/**
 * `slotFilled(host, 'icon')` → was the named slot filled with author-
 * provided content at mount time, or did it fall back to the default?
 *
 * Lets template authors gate fallback content on slot presence without
 * the manual `host.querySelector(':scope > .X > [part="Y"]')` +
 * childNodes walker pattern. Static — slot fills are evaluated once at
 * mount; subsequent fills don't update this. The compiler injects a
 * `slotFilled` local into the component setup so script-block code can
 * call `slotFilled('icon')` without passing the host explicitly.
 */
export function slotFilled(host: Element, name: string): boolean {
  const set = (host as unknown as { __sailSlotFilled?: Set<string> }).__sailSlotFilled
  return set?.has(name) ?? false
}

/**
 * Snapshot the host's current child *nodes* (not just elements)
 * before innerHTML is replaced. Text and comment children survive
 * the round-trip — native slots distribute them, and so should we.
 */
export function snapshotChildren(host: Element): Node[] {
  return Array.from(host.childNodes)
}
