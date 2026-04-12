/**
 * <ship-alert-dialog>
 *
 * A modal confirmation dialog. Replaces window.confirm() with a styled,
 * a11y-friendly primitive. Composes <ship-dialog>.
 *
 * Attributes:
 *   - open: boolean
 *   - tone: default | destructive — tints the confirm button
 *   - confirm-label, cancel-label — button text (defaults: Confirm / Cancel)
 *
 * Slots:
 *   - title
 *   - (default) body
 *
 * Events:
 *   - ship-confirm, ship-cancel — fired by the footer buttons.
 */

import { LitElement, css, html } from 'lit'
import './ship-dialog'
import './ship-button'
import type { Tone } from './util'

export class ShipAlertDialog extends LitElement {
  static styles = css`
    :host { display: contents; }
  `

  static properties = {
    open: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    confirmLabel: { type: String, attribute: 'confirm-label' },
    cancelLabel: { type: String, attribute: 'cancel-label' },
  }

  open = false
  tone: Extract<Tone, 'default' | 'destructive'> = 'default'
  confirmLabel = 'Confirm'
  cancelLabel = 'Cancel'
  private emit(name: 'ship-confirm' | 'ship-cancel') {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
    this.open = false
  }

  render() {
    return html`
      <ship-dialog size="sm" ?open=${this.open}
        @ship-close=${() => { this.open = false }}
      >
        <span slot="header"><slot name="title"></slot></span>
        <slot></slot>
        <span slot="footer">
          <ship-button variant="outline" @click=${() => this.emit('ship-cancel')}>
            ${this.cancelLabel}
          </ship-button>
          <ship-button
            variant="solid"
            tone=${this.tone === 'destructive' ? 'destructive' : 'default'}
            @click=${() => this.emit('ship-confirm')}
          >
            ${this.confirmLabel}
          </ship-button>
        </span>
      </ship-dialog>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-alert-dialog')) {
  customElements.define('ship-alert-dialog', ShipAlertDialog)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-alert-dialog': ShipAlertDialog
  }
}
