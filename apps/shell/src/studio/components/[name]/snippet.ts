// Build a paste-able `<sui-foo ...>example</sui-foo>` snippet from the
// studio's current prop values. Only props whose value differs from the
// declared default are emitted; booleans use bare-attribute presence;
// compound components use their slot composition as inner content.

import type { PropEntry } from '../catalog'
import { slotLabel } from './slot-compositions'

const BOOL_TYPES = new Set(['boolean', 'Boolean', 'bool'])

function isBool(type: string | undefined): boolean {
  return type !== undefined && BOOL_TYPES.has(type)
}

function escapeAttr(v: string): string {
  return v.replace(/&/g, '&amp;').replace(/"/g, '&quot;')
}

export function buildSnippet(
  tag: string,
  values: Record<string, unknown>,
  props: readonly PropEntry[],
  slotHtml: string | undefined,
): string {
  const attrs: string[] = []
  for (const p of props) {
    const v = values[p.name]
    if (isBool(p.type)) {
      const truthy = v === true || v === 'true'
      const defaultTrue = p.default === 'true'
      if (truthy && !defaultTrue) attrs.push(p.name)
      // omit when false or matches default
      continue
    }
    if (v == null || v === '') continue
    const str = String(v)
    if (p.default !== undefined && str === String(p.default)) continue
    attrs.push(`${p.name}="${escapeAttr(str)}"`)
  }
  const attrStr = attrs.length ? ' ' + attrs.join(' ') : ''
  const inner = slotHtml !== undefined ? slotHtml.trim() : slotLabel(tag)
  return `<${tag}${attrStr}>${inner}</${tag}>`
}
