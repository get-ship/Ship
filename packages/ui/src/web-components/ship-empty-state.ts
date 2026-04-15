/**
 * <ship-empty-state>
 *
 * Centered placeholder for empty lists / missing content. Slot-only per
 * charter (no title/description attrs, no ICON_GLYPHS map).
 *
 * Slots:
 *   - icon
 *   - title
 *   - (default) description / body
 *   - action — action buttons
 *
 * Attributes:
 *   - variant: default | connecting | placeholder
 *     - connecting — compact inline "connecting..." affordance used while
 *       SSE streams initialize; no border, just muted text.
 *     - placeholder — no dashed border, just centered message; used for
 *       "nothing selected" states.
 */

import { LitElement, css, html } from 'lit'
export class ShipEmptyState extends LitElement {
  static styles = css`
    :host { display: block; }
    .box {
      display: flex; flex-direction: column;
      align-items: center; justify-content: center;
      gap: 10px;
      padding: 64px 16px;
      text-align: center;
      font-family: var(--font-body);
      color: var(--muted-foreground);
      background: var(--card);
      border: 1px dashed var(--border);
      border-radius: var(--radius);
    }
    :host([variant='connecting']) .box {
      padding: 48px 0;
      background: transparent;
      border: 0;
      font-size: 13px;
    }
    :host([variant='placeholder']) .box {
      padding: 24px 16px;
      background: transparent;
      border: 0;
      font-size: 13px;
    }
    .icon {
      font-size: 36px; opacity: 0.35;
      display: inline-flex; align-items: center; justify-content: center;
    }
    .title {
      font-size: 14px; font-weight: 600;
      color: var(--foreground);
    }
    .desc {
      font-size: 12px;
      max-width: 420px;
      margin: 0 auto;
      line-height: 1.55;
    }
    .actions {
      display: flex; gap: 8px; justify-content: center;
      margin-top: 4px;
    }
    ::slotted([slot='icon']) { display: inline-flex; }
  `

  static properties = {
    variant: { type: String, reflect: true },
  }

  variant: 'default' | 'connecting' | 'placeholder' = 'default'
  render() {
    const live = this.variant === 'connecting' ? 'polite' : 'off'
    return html`
      <div class="box" role="status" aria-live=${live}>
        <div class="icon"><slot name="icon"></slot></div>
        <div class="title"><slot name="title"></slot></div>
        <div class="desc"><slot></slot></div>
        <div class="actions"><slot name="action"></slot></div>
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
