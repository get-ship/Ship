/**
 * <ship-dialog>
 *
 * Wraps native <dialog>. Modal dialog with backdrop. Open via `open=true`
 * attr or `el.show()` / `el.close()`. Closed via the ESC key (native
 * behavior), backdrop click, the × close button, or `el.close()`.
 *
 * Attributes:
 *   - open: boolean — when set, the dialog is visible (modal)
 *   - size: sm | md | lg (default md — constrains max-width)
 *   - dismissible: boolean — clicking the backdrop closes (default true)
 *   - no-close-button: boolean — hides the × close button in the top-right
 *
 * Slots:
 *   - header, (default) body, footer
 *
 * Events:
 *   - ship-open, ship-close — CustomEvents bubbling from the host
 *   - ship-dismiss — fired when the × close button is clicked
 */

import { LitElement, css, html } from 'lit'
export class ShipDialog extends LitElement {
  static styles = css`
    :host { display: contents; }
    dialog {
      padding: 0;
      border: 1px solid var(--border);
      border-radius: var(--radius-lg);
      background: var(--card);
      color: var(--card-foreground);
      max-width: 560px;
      width: 100%;
      max-height: calc(100vh - 64px);
      box-shadow: var(--shadow-xl);
      font-family: var(--font-body);
      opacity: 0;
      transform: translateY(4px);
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    dialog[open] {
      opacity: 1;
      transform: translateY(0);
    }
    dialog::backdrop {
      background: oklch(0 0 0 / 40%);
      backdrop-filter: blur(2px);
      -webkit-backdrop-filter: blur(2px);
    }
    :host([size='sm']) dialog { max-width: 400px; }
    :host([size='lg']) dialog { max-width: 800px; }

    .wrap { display: flex; flex-direction: column; max-height: inherit; position: relative; }
    .header {
      padding: 14px 20px;
      padding-right: 44px;
      border-bottom: 1px solid var(--border);
      font-size: var(--text-md);
      font-weight: var(--font-weight-semibold);
    }
    .body {
      padding: 16px 20px;
      overflow-y: auto;
      font-size: var(--text-base);
      line-height: var(--leading-normal);
    }
    .footer {
      padding: 12px 20px;
      border-top: 1px solid var(--border);
      display: flex; gap: 8px; justify-content: flex-end;
    }
    .close {
      position: absolute;
      top: 10px; right: 10px;
      width: 28px; height: 28px;
      display: inline-flex; align-items: center; justify-content: center;
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--radius-md);
      color: var(--muted-foreground);
      cursor: pointer;
      font: inherit;
      font-size: 18px; line-height: 1;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out);
    }
    .close:hover { background: var(--muted); color: var(--foreground); }
    .close:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--card),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([no-close-button]) .close { display: none; }
  `

  static properties = {
    open: { type: Boolean, reflect: true },
    size: { type: String, reflect: true },
    dismissible: { type: Boolean },
    noCloseButton: { type: Boolean, reflect: true, attribute: 'no-close-button' },
    dialogRole: { type: String, attribute: 'role-override' },
  }

  open = false
  size: 'sm' | 'md' | 'lg' = 'md'
  dismissible = true
  noCloseButton = false
  dialogRole: 'dialog' | 'alertdialog' = 'dialog'
  private static _idSeq = 0
  private uid = ++ShipDialog._idSeq
  private get headerId() { return `ship-dialog-header-${this.uid}` }
  private get bodyId() { return `ship-dialog-body-${this.uid}` }
  get dialogEl(): HTMLDialogElement { return this.renderRoot.querySelector('dialog')! as HTMLDialogElement }

  updated(changed: Map<string, unknown>) {
    this.setAttribute('data-state', this.open ? 'open' : 'closed')
    if (changed.has('open') && this.dialogEl) {
      if (this.open && !this.dialogEl.open) {
        // jsdom / older browsers without <dialog>: gracefully skip.
        if (typeof this.dialogEl.showModal === 'function') {
          try { this.dialogEl.showModal() } catch { /* already-open or env quirk */ }
        }
        this.dispatchEvent(new CustomEvent('ship-open', { bubbles: true, composed: true }))
      } else if (!this.open && this.dialogEl.open) {
        this.dialogEl.close()
      }
    }
  }

  show() { this.open = true }
  close() { this.open = false }

  private onClose = () => {
    this.open = false
    this.dispatchEvent(new CustomEvent('ship-close', { bubbles: true, composed: true }))
  }

  private onBackdropClick = (e: MouseEvent) => {
    if (!this.dismissible) return
    // Click landed directly on <dialog> (outside any child) = backdrop click.
    if (e.target === this.dialogEl) this.close()
  }

  private onDismiss = () => {
    this.dispatchEvent(new CustomEvent('ship-dismiss', { bubbles: true, composed: true }))
    this.close()
  }

  render() {
    return html`
      <dialog
        role=${this.dialogRole}
        aria-modal="true"
        aria-labelledby=${this.headerId}
        aria-describedby=${this.bodyId}
        @close=${this.onClose}
        @click=${this.onBackdropClick}
      >
        <div class="wrap">
          <button
            class="close"
            type="button"
            aria-label="Close dialog"
            @click=${this.onDismiss}
          >×</button>
          <div class="header" id=${this.headerId}><slot name="header"></slot></div>
          <div class="body" id=${this.bodyId}><slot></slot></div>
          <div class="footer"><slot name="footer"></slot></div>
        </div>
      </dialog>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-dialog')) {
  customElements.define('ship-dialog', ShipDialog)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-dialog': ShipDialog
  }
}
