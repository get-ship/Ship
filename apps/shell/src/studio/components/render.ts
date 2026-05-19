// Imperative DOM helpers for the component-detail live preview. Kept out
// of the controller so the controller stays focused on reactive wiring
// and under the file cap. None of these touch innerHTML — slot content
// is parsed into a DocumentFragment so controller-purity holds.

import type { PropEntry } from './catalog'
import { SLOT_COMPOSITIONS } from './slot-compositions'

const BOOL_TYPES = new Set(['boolean', 'Boolean', 'bool'])

export function isBoolType(t: string | undefined): boolean {
  return t !== undefined && BOOL_TYPES.has(t)
}

// Components whose intrinsic size is < ~64px get scaled up so they're
// legible in the live stage (a status-dot at 1× is a speck).
const ZOOM_TAGS = new Set([
  'sui-status-dot', 'sui-kbd', 'sui-badge', 'sui-chip', 'sui-spinner',
])

// Seed initial knob values. For `size` prefer the largest enum variant so
// small components read clearly; otherwise use the declared default.
export function seedDefaults(props: readonly PropEntry[]): Record<string, unknown> {
  const out: Record<string, unknown> = {}
  for (const p of props) {
    if (p.name === 'size' && p.values && p.values.length > 0) {
      out[p.name] = p.values[p.values.length - 1]
    } else if (p.default !== undefined) {
      out[p.name] = p.default
    }
  }
  return out
}

// Fill a host's default slot — compound components get a typical-use
// composition parsed into a fragment (the parser upgrades nested <sui-*>
// as they adopt into the live document); atomic ones get an "example"
// text node.
export function fillSlot(node: HTMLElement, tag: string): void {
  const compose = SLOT_COMPOSITIONS[tag]
  if (compose) {
    const range = document.createRange()
    range.selectNodeContents(node)
    node.replaceChildren(range.createContextualFragment(compose))
  } else {
    node.replaceChildren(document.createTextNode('example'))
  }
}

// Create the live preview host for a tag, slot-filled and zoom-adjusted.
export function createLiveNode(tag: string): HTMLElement {
  const node = document.createElement(tag)
  node.className = 'cmp-live-node'
  fillSlot(node, tag)
  if (ZOOM_TAGS.has(tag)) {
    node.style.transform = 'scale(2.4)'
    node.style.transformOrigin = 'center'
  }
  return node
}

// Apply knob values to the rendered host. Boolean attrs follow HTML
// semantics (presence ≡ true) so they're stripped when falsy; string
// props are removed when unset so the component falls back to its
// declared default rather than rendering an empty attribute.
export function applyProps(
  node: HTMLElement,
  vals: Record<string, unknown>,
  props: readonly PropEntry[],
): void {
  for (const p of props) {
    const v = vals[p.name]
    if (isBoolType(p.type)) {
      const truthy = v === true || v === 'true'
      if (truthy) node.setAttribute(p.name, '')
      else node.removeAttribute(p.name)
    } else if (v == null || v === '') {
      node.removeAttribute(p.name)
    } else {
      node.setAttribute(p.name, String(v))
    }
  }
}
