// DOM morph. Reconcile the contents of `target` to match `source`
// in-place, preserving DOM identity where possible (focus, scroll,
// selection, custom-element state, animation).
//
// This is a focused implementation — not a full idiomorph port. It
// handles the cases Sail's HMR + SSR-hydration use:
// - Same-tag elements: morph attributes + recurse children
// - Different-tag: replace
// - Text nodes: update nodeValue
// - Children align by index (Sail's anchors give us key-stable order)
//
// Key-aware list reconciliation lives in `bindFor`; this morph is for
// non-keyed structural alignment of templates.

/** Options that control which subtrees `morph` will descend into. */
export interface MorphOptions {
  /** Return `true` to leave a subtree untouched (e.g. third-party widgets). */
  skip?: (el: Element) => boolean
}

/**
 * Reconcile the contents of `target` to match `source` in-place,
 * preserving DOM identity where possible (focus, scroll, selection,
 * custom-element state, animations).
 *
 * Same-tag elements have their attributes morphed and children walked
 * by index. Different-tag elements are replaced. Text and Comment
 * nodes have their `nodeValue` patched. Children align by index — Sail
 * relies on `bindFor` for keyed list reconciliation; `morph` is the
 * non-keyed fallback used for HMR + SSR-hydration.
 *
 * @param target DOM element to mutate in place.
 * @param source Element whose shape `target` should converge to.
 * @param opts Optional skip predicate.
 *
 * @example
 * ```ts
 * const next = parseHtml(serverHtml).body.firstElementChild!
 * morph(rootEl, next, { skip: (el) => el.hasAttribute('data-no-morph') })
 * ```
 *
 * @see bindFor for keyed list reconciliation.
 */
export function morph(target: Element, source: Element, opts: MorphOptions = {}): void {
  morphElement(target, source, opts)
}

function morphElement(target: Element, source: Element, opts: MorphOptions): void {
  if (opts.skip?.(target)) return
  if (target.tagName !== source.tagName) {
    target.replaceWith(source.cloneNode(true))
    return
  }
  morphAttributes(target, source)
  morphChildren(target, source, opts)
}

function morphAttributes(target: Element, source: Element): void {
  // Remove attrs gone in source.
  const tAttrs = Array.from(target.attributes)
  for (const a of tAttrs) {
    if (!source.hasAttribute(a.name)) target.removeAttribute(a.name)
  }
  // Set/update attrs from source.
  const sAttrs = Array.from(source.attributes)
  for (const a of sAttrs) {
    if (target.getAttribute(a.name) !== a.value) {
      target.setAttribute(a.name, a.value)
    }
  }
  // Inputs need explicit value/checked sync; attribute alone is the
  // *default*, not the *current* value.
  if (target instanceof HTMLInputElement && source instanceof HTMLInputElement) {
    if (source.hasAttribute('value') && target.value !== source.value) {
      target.value = source.value
    }
    if (target.checked !== source.checked) {
      target.checked = source.checked
    }
  }
}

function morphChildren(target: Element, source: Element, opts: MorphOptions): void {
  const tChildren = Array.from(target.childNodes)
  const sChildren = Array.from(source.childNodes)
  const max = Math.max(tChildren.length, sChildren.length)
  for (let i = 0; i < max; i++) {
    const t = tChildren[i]
    const s = sChildren[i]
    if (!s && t) {
      // Surplus on target — remove.
      target.removeChild(t)
      continue
    }
    if (!t && s) {
      // Missing on target — append clone.
      target.appendChild(s.cloneNode(true))
      continue
    }
    if (!t || !s) continue
    morphNode(t, s, target, opts)
  }
}

function morphNode(t: Node, s: Node, parent: Element, opts: MorphOptions): void {
  if (t.nodeType !== s.nodeType) {
    parent.replaceChild(s.cloneNode(true), t)
    return
  }
  if (t.nodeType === Node.TEXT_NODE || t.nodeType === Node.COMMENT_NODE) {
    if (t.nodeValue !== s.nodeValue) t.nodeValue = s.nodeValue
    return
  }
  if (t.nodeType === Node.ELEMENT_NODE) {
    morphElement(t as Element, s as Element, opts)
  }
}
