/**
 * <ship-icon>
 *
 * Canonical inline SVG icon. Renders one of a bundled set of named glyphs
 * (lucide-equivalent paths), or an arbitrary SVG via the default slot.
 *
 * Attributes:
 *   - name: key into the bundled glyph registry (see ICONS below)
 *   - size: sm | md | lg (default md). Maps to 12/16/20px.
 *   - tone: default | info | success | warning | destructive | accent | muted
 *
 * Slots:
 *   - (default) raw SVG passthrough; if present, takes precedence over `name`.
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from '../util'
import type { Size, Tone } from '../util'

// Lucide-equivalent 24x24 viewBox path data. Single `<path d="...">` each.
export const ICONS: Record<string, string> = {
  'chevron-down': 'M6 9l6 6 6-6',
  'chevron-right': 'M9 6l6 6-6 6',
  'chevron-left': 'M15 6l-6 6 6 6',
  'chevron-up': 'M6 15l6-6 6 6',
  check: 'M20 6L9 17l-5-5',
  x: 'M18 6L6 18M6 6l12 12',
  plus: 'M12 5v14M5 12h14',
  minus: 'M5 12h14',
  search: 'M11 4a7 7 0 100 14 7 7 0 000-14zM21 21l-4.3-4.3',
  'alert-circle': 'M12 2a10 10 0 100 20 10 10 0 000-20zM12 8v4M12 16h.01',
  info: 'M12 2a10 10 0 100 20 10 10 0 000-20zM12 16v-4M12 8h.01',
  star: 'M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z',
  folder: 'M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z',
  file: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6zM14 2v6h6',
  trash: 'M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M6 6l1 14a2 2 0 002 2h6a2 2 0 002-2l1-14',
  edit: 'M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7M18.5 2.5a2.12 2.12 0 113 3L12 15l-4 1 1-4 9.5-9.5z',
  copy: 'M9 9h10a2 2 0 012 2v10a2 2 0 01-2 2H9a2 2 0 01-2-2V11a2 2 0 012-2zM5 15H4a2 2 0 01-2-2V3a2 2 0 012-2h10a2 2 0 012 2v1',
  'external-link': 'M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3',
  'arrow-up': 'M12 19V5M5 12l7-7 7 7',
  'arrow-down': 'M12 5v14M19 12l-7 7-7-7',
  'arrow-left': 'M19 12H5M12 19l-7-7 7-7',
  'arrow-right': 'M5 12h14M12 5l7 7-7 7',
  sparkles: 'M12 3l2 5 5 2-5 2-2 5-2-5-5-2 5-2 2-5zM19 14l1 2 2 1-2 1-1 2-1-2-2-1 2-1 1-2z',
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
  }

  name = ''
  size: Size = 'md'
  tone: Tone = 'default'

  render() {
    const d = this.name ? ICONS[this.name] : undefined
    return html`
      <slot>
        ${d
          ? html`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
              stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden="true" part="svg">
              <path d=${d}></path>
            </svg>`
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
