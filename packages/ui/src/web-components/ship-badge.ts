/**
 * <ship-badge>
 *
 * Static status label. Small, non-interactive, tone-colored. Reads as
 * inline chrome: "New", "Beta", "3", "Active". If the pill needs to
 * respond to clicks, selection, or removal, reach for <ship-chip>.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     (default: muted)
 *   - variant: solid | soft | outline (default: soft)
 *   - size: sm | md | lg (default: md)
 *   - pill: boolean — flip to fully-rounded (--radius-full). Default is
 *     squared (--radius-sm) for visual parity with the pre-polish badge
 *     used across tables and row metadata.
 *
 * Slots:
 *   - leading — icon / status dot before the label. Rendered at the
 *     size-appropriate icon dimension; tints with currentColor so
 *     <ship-status-dot> / <ship-icon> children pick up the badge tone.
 *   - (default) label text
 *
 * Parts:
 *   - badge — the outer pill element
 *   - leading — the leading-icon slot container
 */

import { LitElement, html } from 'lit'
import { shipBadgeStyles } from './ship-badge.styles'
import type { Size, Tone } from './util'

type BadgeVariant = 'solid' | 'soft' | 'outline'

export class ShipBadge extends LitElement {
  static styles = shipBadgeStyles

  static properties = {
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    pill: { type: Boolean, reflect: true },
  }

  tone: Tone
  variant: BadgeVariant | null
  size: Size
  pill: boolean

  constructor() {
    super()
    this.tone = 'muted'
    this.variant = null
    this.size = 'md'
    this.pill = false
  }

  render() {
    return html`
      <span class="badge" part="badge">
        <span class="leading" part="leading"><slot name="leading"></slot></span>
        <slot></slot>
      </span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-badge')) {
  customElements.define('ship-badge', ShipBadge)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-badge': ShipBadge
  }
}
