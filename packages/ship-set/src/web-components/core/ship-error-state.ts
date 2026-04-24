/**
 * <ship-error-state>
 *
 * Sibling of <ship-empty-state> for the "something broke" path.
 * Every view's error branch routes through this so failures read
 * deliberately (destructive halo, clear title, actionable body,
 * optional technical details, primary retry) rather than a bespoke
 * inline div per view.
 *
 * Attributes:
 *   - title  — one-sentence summary (foreground bold).
 *   - body   — what the user should do next (muted, wraps ~60ch).
 *   - size   — sm | md | lg (default md). Controls padding/type scale.
 *   - variant — default | inline (default: default). Inline is a
 *               single-row treatment for "this section failed" within
 *               a larger view.
 *   - icon   — ship-icon glyph name for the halo. Default "alert-circle".
 *
 * Slots:
 *   - title     — override the attr with richer markup
 *   - (default) — body; falls back to `body` attr
 *   - details   — optional technical detail (stack, request id).
 *                 Rendered collapsed behind a "Show details" toggle.
 *   - action    — one or two CTAs. The first button with slot="retry"
 *                 gets auto-focused on mount so keyboard users can
 *                 hit Enter to recover.
 */

import { LitElement, css, html } from 'lit'
import { shipEmptyStateStyles } from './ship-empty-state.styles'
import { ICONS } from './ship-icon'
import type { Size } from './util'

type ErrorVariant = 'default' | 'inline'

const EXTRA = css`
  :host { --tone-fg: var(--destructive); }
  :host([variant='inline']) .box {
    background: color-mix(in oklch, var(--destructive) 4%, var(--card));
    border: 1px solid color-mix(in oklch, var(--destructive) 24%, var(--border));
    border-radius: var(--radius-md);
  }
  .details {
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: color-mix(in oklch, var(--destructive) 6%, var(--muted));
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    white-space: pre-wrap;
    word-break: break-word;
    max-width: 60ch;
    text-align: left;
  }
  .details-toggle {
    appearance: none; background: transparent; border: 0;
    font: inherit; font-size: var(--text-xs);
    color: var(--muted-foreground); cursor: pointer;
    padding: 2px 6px; border-radius: var(--radius-sm);
    margin-top: var(--space-1);
  }
  .details-toggle:hover { color: var(--foreground); background: var(--muted); }
`

export class ShipErrorState extends LitElement {
  static styles = [...shipEmptyStateStyles, EXTRA]

  static properties = {
    title: { type: String },
    body: { type: String },
    size: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    icon: { type: String },
    _open: { state: true },
    _hasDetails: { state: true },
  }

  title = ''
  body = ''
  size: Size = 'md'
  variant: ErrorVariant = 'default'
  icon = 'alert-circle'
  private _open = false
  private _hasDetails = false

  private onDetailsSlotChange = (e: Event) => {
    const slot = e.target as HTMLSlotElement
    this._hasDetails = slot.assignedNodes({ flatten: true }).some(n =>
      n.nodeType === Node.ELEMENT_NODE ||
      (n.nodeType === Node.TEXT_NODE && (n.textContent || '').trim().length > 0)
    )
  }

  private onActionSlotChange = (e: Event) => {
    const slot = e.target as HTMLSlotElement
    const retry = slot.assignedNodes({ flatten: true })
      .find(n => n.nodeType === Node.ELEMENT_NODE && (n as Element).getAttribute('slot') === 'retry') as HTMLElement | undefined
    if (retry) requestAnimationFrame(() => retry.focus?.())
  }

  render() {
    // Destructive tone is implicit — set data-initials-like attr so the
    // shared empty-state styles pick up the destructive tile tint.
    const path = ICONS[this.icon]
    return html`
      <div class="box" part="box" role="alert" tone="destructive">
        <div class="icon" part="icon" aria-hidden="true">
          ${path ? html`<svg class="glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d=${path}></path></svg>` : null}
        </div>

        <div class="copy">
          <div class="title" part="title">
            <slot name="title">${this.title || 'Something went wrong'}</slot>
          </div>
          <div class="desc" part="desc">
            <slot>${this.body}</slot>
          </div>
          ${this._hasDetails
            ? html`
                <button class="details-toggle" type="button"
                        aria-expanded=${this._open}
                        @click=${() => { this._open = !this._open }}>
                  ${this._open ? 'Hide details' : 'Show details'}
                </button>
                <div class="details" ?hidden=${!this._open}>
                  <slot name="details" @slotchange=${this.onDetailsSlotChange}></slot>
                </div>
              `
            : html`<slot name="details" @slotchange=${this.onDetailsSlotChange} hidden></slot>`}
        </div>

        <div class="actions" part="actions">
          <slot name="action" @slotchange=${this.onActionSlotChange}></slot>
        </div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-error-state')) {
  customElements.define('ship-error-state', ShipErrorState)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-error-state': ShipErrorState
  }
}
