/**
 * <ship-icon>
 *
 * Canonical inline SVG icon. Resolves `name` against the curated
 * Lucide registry generated from `lucide-static` (ISC). Names not
 * present in the curated allowlist fall through to the default slot
 * — provide raw SVG there for one-off glyphs.
 *
 * Add a new icon: append its name to
 * `packages/ship-set/scripts/lucide-allowlist.json`, then
 * `bun run --filter @ship/set build`.
 *
 * Attributes:
 *   - name: Lucide icon id, kebab-case (e.g. "chevron-right",
 *     "messages-square"). A small alias map handles legacy names
 *     used by older templates (`stop` → `stop-circle`,
 *     `refresh` → `refresh-cw`, `git` → `git-branch`).
 *   - size: sm | md | lg (default md). 12 / 16 / 20px.
 *   - tone: default | info | success | warning | destructive | accent | muted | primary
 *   - label: when provided, the host gets role=img + aria-label;
 *     otherwise aria-hidden is set.
 *
 * Slots:
 *   - (default) — raw SVG passthrough. Used as a fallback when
 *     `name` is not in the registry.
 */

import { LitElement, css, html, nothing } from 'lit'
import { unsafeSVG } from 'lit/directives/unsafe-svg.js'
import { LUCIDE_ICONS } from './lucide-icons.generated'
import { toneVars } from './util'
import type { Size, Tone } from './util'

/**
 * Aliases mapping legacy / shorthand names that templates currently
 * use to the canonical Lucide name. Keep this list short and only
 * for backward compatibility — prefer authoring with the canonical
 * Lucide name in new templates.
 */
const ALIASES: Record<string, string> = {
  // Legacy short names
  git: 'git-branch',
  refresh: 'refresh-cw',
  stop: 'stop-circle',
  // shadcn-style alias
  warning: 'triangle-alert',
}

function resolveName(name: string): string | undefined {
  if (!name) return undefined
  if (LUCIDE_ICONS[name]) return name
  const aliased = ALIASES[name]
  if (aliased && LUCIDE_ICONS[aliased]) return aliased
  return undefined
}

export class ShipIcon extends LitElement {
  static styles = [
    toneVars,
    css`
      :host { display: inline-flex; align-items: center; justify-content: center; color: var(--tone-fg); }
      svg { width: 16px; height: 16px; display: block; }
      :host([size='sm']) svg { width: 12px; height: 12px; }
      :host([size='lg']) svg { width: 20px; height: 20px; }
      ::slotted(svg) { width: 100%; height: 100%; }
    `,
  ]

  static properties = {
    name: { type: String, reflect: true },
    size: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    label: { type: String, reflect: true },
  }

  name = ''
  size: Size = 'md'
  tone: Tone = 'default'
  label = ''

  updated() {
    if (this.label) {
      this.setAttribute('role', 'img')
      this.setAttribute('aria-label', this.label)
      this.removeAttribute('aria-hidden')
    } else {
      this.setAttribute('aria-hidden', 'true')
      this.removeAttribute('role')
      this.removeAttribute('aria-label')
    }
  }

  render() {
    const resolved = resolveName(this.name)
    const inner = resolved ? LUCIDE_ICONS[resolved] : undefined
    const hidden = this.label ? nothing : 'true'
    return html`
      <slot>
        ${inner
          ? html`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
              stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden=${hidden} part="svg">${unsafeSVG(inner)}</svg>`
          : null}
      </slot>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-icon')) {
  customElements.define('ship-icon', ShipIcon)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-icon': ShipIcon
  }
}
