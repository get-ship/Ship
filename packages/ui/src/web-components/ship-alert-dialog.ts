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
import { customElement, property } from 'lit/decorators.js'
import './ship-dialog'
import './ship-button'
import type { Tone } from './util'

@customElement('ship-alert-dialog')
export class ShipAlertDialog extends LitElement {
  static styles = css`
    :host { display: contents; }
  `

  @property({ type: Boolean, reflect: true }) open = false
  @property({ reflect: true }) tone: Extract<Tone, 'default' | 'destructive'> = 'default'
  @property({ attribute: 'confirm-label' }) confirmLabel = 'Confirm'
  @property({ attribute: 'cancel-label' }) cancelLabel = 'Cancel'

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

declare global {
  interface HTMLElementTagNameMap {
    'ship-alert-dialog': ShipAlertDialog
  }
}
