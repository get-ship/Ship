/**
 * <ship-card>
 *
 * Generic container surface — matches shadcn/ui's Card visual weight:
 * soft border, --radius-lg corners, --shadow-xs at rest, --shadow-sm on
 * hover for interactive cards, clean header/content/footer composition.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     Tints the border and (when `interactive`) the focus ring. Pairs with
 *     `fill` or `status-rail` for alert/callout-style cards.
 *   - variant: default | outline (default: default)
 *     `outline` drops the solid fill + shadow; useful for cards nested
 *     inside another panel.
 *   - padding: none | sm | md | lg (default: md)
 *     Density channel. `md` matches the pre-polish 16px/20px padding so
 *     existing composites (pipeline-definition, pipeline-run, settings
 *     cards) keep their weight. `none` hands spacing back to the caller.
 *   - interactive: boolean
 *     Turns the card into a button-like affordance — cursor, hover lift,
 *     active press, focus-visible ring, Enter/Space key activation, and
 *     role="button" / tabindex="0" set automatically.
 *   - accent-bar: boolean
 *     Renders a 2-stop gradient top-bar that fades in on hover (matches
 *     `.def-card::before`).
 *   - status-rail: boolean
 *     Tone-driven 3px left rail (matches `.run-card.is-{status}`).
 *   - fill: boolean
 *     Tone-tinted background wash. Pairs with `status-rail` or `tone`
 *     for an alert-style card.
 *
 * Slots:
 *   - header — optional; gets a border-bottom separator when filled
 *   - (default) — primary content
 *   - footer — optional; gets a border-top separator when filled
 */

import { LitElement, html } from 'lit'
import { shipCardStyles } from './ship-card.styles'
import type { Tone } from './util'

type CardPadding = 'none' | 'sm' | 'md' | 'lg'
type CardVariant = 'default' | 'outline'

export class ShipCard extends LitElement {
  static styles = shipCardStyles

  static properties = {
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    padding: { type: String, reflect: true },
    interactive: { type: Boolean, reflect: true },
    accentBar: { type: Boolean, reflect: true, attribute: 'accent-bar' },
    statusRail: { type: Boolean, reflect: true, attribute: 'status-rail' },
    fill: { type: Boolean, reflect: true },
  }

  tone?: Tone
  variant: CardVariant = 'default'
  padding: CardPadding = 'md'
  interactive = false
  accentBar = false
  statusRail = false
  fill = false

  connectedCallback() {
    super.connectedCallback()
    if (this.interactive) this.activateInteractiveRole()
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this.onKey)
  }
  updated(changed: Map<string, unknown>) {
    if (changed.has('interactive')) {
      if (this.interactive) this.activateInteractiveRole()
      else this.deactivateInteractiveRole()
    }
  }
  private activateInteractiveRole() {
    if (!this.hasAttribute('role')) this.setAttribute('role', 'button')
    if (!this.hasAttribute('tabindex')) this.setAttribute('tabindex', '0')
    this.addEventListener('keydown', this.onKey)
  }
  private deactivateInteractiveRole() {
    if (this.getAttribute('role') === 'button') this.removeAttribute('role')
    if (this.getAttribute('tabindex') === '0') this.removeAttribute('tabindex')
    this.removeEventListener('keydown', this.onKey)
  }
  private onKey = (e: KeyboardEvent) => {
    if (!this.interactive) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this.click()
    }
  }

  /** Reflect projected content onto the wrapper so CSS can render a
   * separator only when the header/footer slot actually has content. */
  private syncFilled = (e: Event) => {
    const slot = e.target as HTMLSlotElement
    const wrapper = slot.parentElement as HTMLElement | null
    if (!wrapper) return
    const filled = slot.assignedNodes({ flatten: true }).some((n) => {
      if (n.nodeType === Node.TEXT_NODE) return (n.textContent || '').trim().length > 0
      return n.nodeType === Node.ELEMENT_NODE
    })
    if (filled) wrapper.setAttribute('data-filled', '')
    else wrapper.removeAttribute('data-filled')
  }

  render() {
    return html`
      <div class="header" part="header">
        <slot name="header" @slotchange=${this.syncFilled}></slot>
      </div>
      <div class="content" part="content">
        <slot></slot>
      </div>
      <div class="footer" part="footer">
        <slot name="footer" @slotchange=${this.syncFilled}></slot>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-card')) {
  customElements.define('ship-card', ShipCard)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-card': ShipCard
  }
}
