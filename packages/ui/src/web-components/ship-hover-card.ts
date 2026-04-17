/**
 * <ship-hover-card>
 *
 * Hover-triggered card overlay. Richer than a tooltip: supports arbitrary
 * slotted content. Shows after a configurable delay, hides after a shorter
 * delay. Anchored to the trigger element.
 *
 * Attributes:
 *   - open: boolean — reflects the visible state.
 *   - placement: top | bottom | left | right (default: bottom)
 *   - show-delay: number (ms, default 300)
 *   - hide-delay: number (ms, default 100)
 *
 * Slots:
 *   - trigger — the element that activates the hover card.
 *   - (default) — card body content.
 *
 * Parts:
 *   - card — the positioned card panel.
 *
 * Events:
 *   - ship-open  — dispatched when the card becomes visible.
 *   - ship-close — dispatched when the card hides.
 */

import { LitElement, css, html } from 'lit'
import { reducedMotion } from './util'

export class ShipHoverCard extends LitElement {
  static styles = [reducedMotion, css`
    :host { position: relative; display: inline-flex; }
    .card {
      position: absolute;
      z-index: var(--z-popover);
      min-width: 240px;
      max-width: 340px;
      padding: 16px;
      background: var(--popover);
      color: var(--popover-foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius-lg);
      box-shadow: var(--shadow-lg);
      font-family: var(--font-body);
      font-size: var(--text-base);
      line-height: var(--leading-normal);
      opacity: 0;
      transform: translateY(-4px);
      pointer-events: none;
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    :host([open]) .card {
      opacity: 1;
      transform: translateY(0);
      pointer-events: auto;
    }
    /* Arrow */
    .arrow {
      position: absolute;
      width: 10px;
      height: 10px;
      background: var(--popover);
      border: 1px solid var(--border);
      transform: rotate(45deg);
      z-index: -1;
    }
    /* Placement: bottom (default) */
    :host([placement='bottom']) .card,
    :host(:not([placement])) .card { top: calc(100% + 8px); left: 50%; transform: translateX(-50%); }
    :host([placement='bottom'][open]) .card,
    :host(:not([placement])[open]) .card { transform: translateX(-50%) translateY(0); }
    :host([placement='bottom']) .arrow,
    :host(:not([placement])) .arrow {
      top: -6px; left: 50%; transform: translateX(-50%) rotate(45deg);
      border-bottom: none; border-right: none;
    }
    /* Placement: top */
    :host([placement='top']) .card { bottom: calc(100% + 8px); left: 50%; transform: translateX(-50%); }
    :host([placement='top'][open]) .card { transform: translateX(-50%) translateY(0); }
    :host([placement='top']) .arrow {
      bottom: -6px; left: 50%; transform: translateX(-50%) rotate(45deg);
      border-top: none; border-left: none;
    }
    /* Placement: right */
    :host([placement='right']) .card { left: calc(100% + 8px); top: 50%; transform: translateY(-50%); }
    :host([placement='right'][open]) .card { transform: translateY(-50%); }
    :host([placement='right']) .arrow {
      left: -6px; top: 50%; transform: translateY(-50%) rotate(45deg);
      border-top: none; border-right: none;
    }
    /* Placement: left */
    :host([placement='left']) .card { right: calc(100% + 8px); top: 50%; transform: translateY(-50%); }
    :host([placement='left'][open]) .card { transform: translateY(-50%); }
    :host([placement='left']) .arrow {
      right: -6px; top: 50%; transform: translateY(-50%) rotate(45deg);
      border-bottom: none; border-left: none;
    }
    @media (forced-colors: active) {
      .card { border: 1px solid CanvasText; }
    }
  `]

  static properties = {
    open: { type: Boolean, reflect: true },
    placement: { type: String, reflect: true },
    showDelay: { type: Number, attribute: 'show-delay' },
    hideDelay: { type: Number, attribute: 'hide-delay' },
  }

  open = false
  placement: 'top' | 'bottom' | 'left' | 'right' = 'bottom'
  showDelay = 300
  hideDelay = 100

  private static _idSeq = 0
  private cardId = `ship-hover-card-${++ShipHoverCard._idSeq}`
  private showTimer: ReturnType<typeof setTimeout> | null = null
  private hideTimer: ReturnType<typeof setTimeout> | null = null

  show() {
    this.clearTimers()
    this.open = true
    this.emit('ship-open')
  }

  hide() {
    this.clearTimers()
    this.open = false
    this.emit('ship-close')
  }

  private emit(name: string) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
  }

  private clearTimers() {
    if (this.showTimer) { clearTimeout(this.showTimer); this.showTimer = null }
    if (this.hideTimer) { clearTimeout(this.hideTimer); this.hideTimer = null }
  }

  private onMouseEnter = () => {
    this.clearTimers()
    this.showTimer = setTimeout(() => this.show(), this.showDelay)
  }

  private onMouseLeave = () => {
    this.clearTimers()
    this.hideTimer = setTimeout(() => this.hide(), this.hideDelay)
  }

  private onFocusIn = () => {
    this.clearTimers()
    this.show()
  }

  private onFocusOut = () => {
    this.clearTimers()
    this.hideTimer = setTimeout(() => this.hide(), this.hideDelay)
  }

  private wireTrigger = () => {
    const slot = this.renderRoot.querySelector('slot[name="trigger"]') as HTMLSlotElement | null
    const first = slot?.assignedElements({ flatten: true })[0] as HTMLElement | undefined
    if (!first) return
    first.setAttribute('aria-describedby', this.cardId)
  }

  updated() {
    this.setAttribute('data-state', this.open ? 'open' : 'closed')
  }

  connectedCallback() {
    super.connectedCallback()
    this.addEventListener('mouseenter', this.onMouseEnter)
    this.addEventListener('mouseleave', this.onMouseLeave)
    this.addEventListener('focusin', this.onFocusIn)
    this.addEventListener('focusout', this.onFocusOut)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.clearTimers()
    this.removeEventListener('mouseenter', this.onMouseEnter)
    this.removeEventListener('mouseleave', this.onMouseLeave)
    this.removeEventListener('focusin', this.onFocusIn)
    this.removeEventListener('focusout', this.onFocusOut)
  }

  render() {
    return html`
      <slot name="trigger" @slotchange=${this.wireTrigger}></slot>
      <div id=${this.cardId} class="card" part="card" role="tooltip"
        aria-hidden=${this.open ? 'false' : 'true'}>
        <div class="arrow"></div>
        <slot></slot>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-hover-card')) {
  customElements.define('ship-hover-card', ShipHoverCard)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-hover-card': ShipHoverCard
  }
}
