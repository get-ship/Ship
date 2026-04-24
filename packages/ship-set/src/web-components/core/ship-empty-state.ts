/**
 * <ship-empty-state>
 *
 * Centered placeholder for empty lists / missing content. Every view's
 * empty state should feel deliberate, not bare — this primitive supplies
 * the icon tile, typography rhythm, size channel, and variant chrome so
 * callers only write the copy + CTA.
 *
 * Attributes:
 *   - variant: default | inline | placeholder | connecting (default: default)
 *     - default     dashed-border card; "zero items" hero empty state
 *     - inline      compact horizontal row; list-within-list empty state
 *     - placeholder subtle no-border; "nothing selected" vs "no items"
 *     - connecting  backward-compat alias for placeholder + aria-live=polite,
 *                   used while SSE streams initialize
 *   - size: sm | md | lg (default: md)
 *     Controls padding, icon size, and typography scale together.
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     Tints the icon tile; bare states keep muted neutrals.
 *   - icon: string — ship-icon glyph name used when the `icon` slot is empty.
 *     Slot always wins; the attr is a convenience fallback so callers
 *     don't have to wire <ship-icon> manually for the common path.
 *
 * Slots:
 *   - icon          — optional. Takes precedence over the `icon` attribute.
 *   - title         — bold foreground headline.
 *   - (default)     — description / body. Wraps at ~40ch for readability.
 *   - action        — one or two CTAs; rendered centered below the copy
 *                     (or on the trailing edge in variant=inline).
 *
 * Parts:
 *   - box     — outer container (surface + border + padding)
 *   - icon    — icon tile wrapper
 *   - title   — title row
 *   - desc    — description row
 *   - actions — actions row
 */

import { LitElement, html, nothing } from 'lit'
import { shipEmptyStateStyles } from './ship-empty-state.styles'
// Inline-render the glyph from the shared registry so we control the SVG
// size directly (ship-icon locks its internal SVG to 12/16/20px which
// would leave the icon tile looking sparse at md/lg).
import { ICONS } from './ship-icon'
import type { Size, Tone } from './util'

type EmptyVariant = 'default' | 'inline' | 'placeholder' | 'connecting'

export class ShipEmptyState extends LitElement {
  static styles = shipEmptyStateStyles

  static properties = {
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    icon: { type: String, reflect: true },
  }

  variant: EmptyVariant = 'default'
  size: Size = 'md'
  tone?: Tone
  icon = ''

  /** Track whether the icon slot has projected content so the `icon` attr
   * fallback only renders when the slot is genuinely empty. Slot always
   * wins — matches the charter rule "slots over attrs for content". */
  private iconSlotFilled = false

  private onIconSlotChange = (e: Event) => {
    const slot = e.target as HTMLSlotElement
    const filled = slot.assignedNodes({ flatten: true }).some((n) => {
      if (n.nodeType === Node.TEXT_NODE) {
        return (n.textContent || '').trim().length > 0
      }
      return n.nodeType === Node.ELEMENT_NODE
    })
    if (filled !== this.iconSlotFilled) {
      this.iconSlotFilled = filled
      this.requestUpdate()
    }
  }

  render() {
    // Connecting gets a polite live region so SSE-driven views announce
    // the state transition to screen readers without stealing focus.
    const live = this.variant === 'connecting' ? 'polite' : 'off'
    const role = this.variant === 'connecting' ? 'status' : 'note'

    // The icon slot always wins. When empty, fall back to the `icon` attr
    // via an inline SVG pulled from the shared ICONS registry so the
    // glyph scales with the tile. Either way the tile owns tone + sizing
    // so callers don't restyle their glyph per size. `data-empty` collapses
    // the tile when neither path produces a glyph.
    const fallbackPath = this.icon ? ICONS[this.icon] : undefined
    const hasFallback = !this.iconSlotFilled && !!fallbackPath
    const tileHasContent = this.iconSlotFilled || hasFallback

    return html`
      <div class="box" part="box" role=${role} aria-live=${live}>
        <div
          class="icon"
          part="icon"
          aria-hidden="true"
          ?data-empty=${!tileHasContent}
        >
          <slot name="icon" @slotchange=${this.onIconSlotChange}>
            ${hasFallback
              ? html`<svg
                  class="glyph"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.75"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path d=${fallbackPath}></path>
                </svg>`
              : nothing}
          </slot>
        </div>

        <div class="copy">
          <div class="title" part="title">
            <slot name="title"></slot>
          </div>
          <div class="desc" part="desc">
            <slot></slot>
          </div>
        </div>

        <div class="actions" part="actions">
          <slot name="action"></slot>
        </div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-empty-state')) {
  customElements.define('ship-empty-state', ShipEmptyState)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-empty-state': ShipEmptyState
  }
}
