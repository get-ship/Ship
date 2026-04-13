/**
 * <ship-toast> + <ship-toast-container>
 *
 * Fixed-position stacking notification container and an individual toast.
 *
 * <ship-toast-container>
 *   Attributes:
 *     - position: top-right | bottom-right | bottom-left | top-left
 *       (default top-right)
 *   Slots:
 *     - (default) ship-toast children, stacked.
 *
 * <ship-toast>
 *   Attributes:
 *     - tone: default | info | success | warning | destructive | accent | muted
 *     - duration: ms before auto-dismiss; 0 = persistent. Default 5000.
 *     - dismissible: boolean, default true.
 *
 *   Slots:
 *     - icon, title, actions, (default) body.
 *
 *   Events:
 *     - ship-dismiss — fires when the toast dismisses itself or the user
 *       clicks the close button.
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from '../util'
import type { Tone } from '../util'

export type ToastPosition = 'top-right' | 'bottom-right' | 'bottom-left' | 'top-left'

export class ShipToastContainer extends LitElement {
  static styles = css`
    :host {
      position: fixed;
      top: 16px; right: 16px;
      display: flex; flex-direction: column; gap: 8px;
      z-index: var(--z-toast);
      pointer-events: none;
      max-width: min(420px, calc(100vw - 32px));
    }
    :host([position='bottom-right']) { top: auto; bottom: 16px; right: 16px; flex-direction: column-reverse; }
    :host([position='bottom-left']) { top: auto; bottom: 16px; left: 16px; right: auto; flex-direction: column-reverse; }
    :host([position='top-left']) { left: 16px; right: auto; }
    ::slotted(ship-toast) { pointer-events: auto; }
  `

  static properties = { position: { type: String, reflect: true } }
  position: ToastPosition = 'top-right'

  render() {
    return html`<slot></slot>`
  }
}

export class ShipToast extends LitElement {
  static styles = [
    toneVars,
    css`
      :host {
        display: block;
        animation: slide-in var(--duration-medium) var(--ease-out);
      }
      .toast {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 10px;
        align-items: start;
        padding: 12px 14px;
        border-radius: var(--radius);
        border: 1px solid var(--tone-border);
        background: var(--tone-bg);
        color: var(--tone-fg);
        box-shadow: var(--shadow-md);
        font-family: var(--font-body);
        font-size: var(--text-base);
        line-height: var(--leading-normal);
      }
      .content { display: flex; flex-direction: column; gap: 4px; min-width: 0; color: var(--foreground); }
      .title { font-weight: var(--font-weight-semibold); color: var(--tone-fg); }
      .close {
        background: transparent; border: 0; color: inherit;
        cursor: pointer; font-size: 14px; padding: 2px 6px;
        border-radius: var(--radius-sm);
      }
      .close:hover { background: color-mix(in oklch, var(--tone-fg) 14%, transparent); }
      .close:focus-visible {
        outline: var(--ring-width) solid var(--ring-color);
        outline-offset: var(--ring-offset);
      }
      @keyframes slide-in {
        from { opacity: 0; transform: translateY(-8px); }
        to   { opacity: 1; transform: translateY(0); }
      }
    `,
  ]

  static properties = {
    tone: { type: String, reflect: true },
    duration: { type: Number },
    dismissible: { type: Boolean, reflect: true },
  }

  tone: Tone = 'info'
  duration = 5000
  dismissible = true

  private _timer: ReturnType<typeof setTimeout> | null = null

  connectedCallback() {
    super.connectedCallback()
    if (this.duration > 0) {
      this._timer = setTimeout(() => this.dismiss(), this.duration)
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    if (this._timer) clearTimeout(this._timer)
  }

  dismiss() {
    this.dispatchEvent(new CustomEvent('ship-dismiss', {
      bubbles: true, composed: true,
    }))
    this.remove()
  }

  render() {
    return html`
      <div class="toast" role="status" part="toast">
        <span><slot name="icon"></slot></span>
        <div class="content">
          <div class="title"><slot name="title"></slot></div>
          <div><slot></slot></div>
          <div><slot name="actions"></slot></div>
        </div>
        ${this.dismissible
          ? html`<button class="close" aria-label="Dismiss" @click=${this.dismiss}>×</button>`
          : html`<span></span>`}
      </div>
    `
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-toast-container'))
    customElements.define('ship-toast-container', ShipToastContainer)
  if (!customElements.get('ship-toast'))
    customElements.define('ship-toast', ShipToast)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-toast': ShipToast
    'ship-toast-container': ShipToastContainer
  }
}
