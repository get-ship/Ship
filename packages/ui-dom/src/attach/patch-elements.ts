// patch-elements — runtime applier for surgical DOM patches received
// over the per-view SSE channel.
//
// # Trust boundary
//
// Patches are scoped to the view root (`[data-sui-root][data-view=...]`)
// passed in by `attach.ts`. Selectors are evaluated via
// `root.querySelector`, never `document.querySelector` — so a patch
// targeting `body > .ship-shell` finds no match inside the view and
// is dropped with a console warning. This boundary is the runtime's
// half of the security contract; the daemon's half is the bearer-auth
// gate on `POST /_sui/views/{name}/patch-elements`.
//
// HTML is applied verbatim. The daemon is the trust boundary; per the
// phase 0.5 spec, the runtime does NOT sanitize incoming HTML.
// (See `apps/shipd/src/views/sui_agent_push.rs` for the rationale.)
//
// # Wire format
//
// ```json
// { "selector":"#main .item[data-id='42']",
//   "mode":"morph",
//   "html":"<div class='item' data-id='42'>...</div>",
//   "settle": 150 }
// ```
//
// `mode` ∈ `replace | append | prepend | before | after | morph`.
// `settle` (optional, ms) toggles a `data-sui-settling` attribute on
// the patched root for the duration so authors can hook CSS transitions.

import { morph } from '../morph'

export type PatchElementsMode =
  | 'replace'
  | 'append'
  | 'prepend'
  | 'before'
  | 'after'
  | 'morph'

export interface PatchElementsPayload {
  selector: string
  mode: PatchElementsMode
  html: string
  /** Optional settle duration in ms; toggles `data-sui-settling`. */
  settle?: number
}

const VALID_MODES: ReadonlySet<PatchElementsMode> = new Set([
  'replace',
  'append',
  'prepend',
  'before',
  'after',
  'morph',
])

/**
 * Reject selectors that look like HTML, are empty, or contain
 * characters outside the CSS-selector grammar. We don't try to be a
 * full CSS parser — `querySelector` itself rejects malformed
 * selectors and throws a `SyntaxError` we catch below — this is a
 * cheap pathological-case guard.
 */
export function isValidSelector(s: unknown): s is string {
  if (typeof s !== 'string') return false
  if (s.length === 0 || s.trim().length === 0) return false
  // `<` and `>` aren't legal anywhere in a CSS selector. Quotes are
  // legal inside attribute values like `[data-x="y"]` but nowhere else,
  // so we allow them; querySelector will reject the rest.
  if (/[<>]/.test(s)) return false
  return true
}

/**
 * Apply a `patch-elements` payload to the DOM under `root`.
 * Returns true when the patch was applied, false when it was rejected
 * or the selector matched nothing.
 */
export function applyPatchElements(
  root: HTMLElement,
  patch: PatchElementsPayload,
): boolean {
  if (!isValidSelector(patch.selector)) {
    console.warn('[ship-set/patch-elements] rejected invalid selector', patch.selector)
    return false
  }
  if (!VALID_MODES.has(patch.mode)) {
    console.warn('[ship-set/patch-elements] rejected unknown mode', patch.mode)
    return false
  }
  if (typeof patch.html !== 'string') {
    console.warn('[ship-set/patch-elements] html must be a string')
    return false
  }

  let target: Element | null
  try {
    target = root.querySelector(patch.selector)
  } catch (err) {
    console.warn('[ship-set/patch-elements] selector parse error', patch.selector, err)
    return false
  }
  if (!target) {
    console.warn(
      '[ship-set/patch-elements] no element matched selector under view root',
      patch.selector,
    )
    return false
  }

  applyByMode(target, patch.mode, patch.html)

  if (typeof patch.settle === 'number' && patch.settle > 0) {
    markSettling(target, patch.settle)
  }
  return true
}

function applyByMode(
  target: Element,
  mode: PatchElementsMode,
  html: string,
): void {
  switch (mode) {
    case 'replace':
      target.outerHTML = html
      return
    case 'append':
      target.insertAdjacentHTML('beforeend', html)
      return
    case 'prepend':
      target.insertAdjacentHTML('afterbegin', html)
      return
    case 'before':
      target.insertAdjacentHTML('beforebegin', html)
      return
    case 'after':
      target.insertAdjacentHTML('afterend', html)
      return
    case 'morph':
      morph(target, html)
      return
  }
}

function markSettling(el: Element, ms: number): void {
  const node = el.isConnected ? el : null
  if (!node) return
  node.setAttribute('data-sui-settling', '')
  const win = node.ownerDocument?.defaultView
  if (!win) {
    node.removeAttribute('data-sui-settling')
    return
  }
  win.setTimeout(() => {
    node.removeAttribute('data-sui-settling')
  }, ms)
}
