/**
 * <ship-dialog>
 *
 * Wraps native <dialog>. Modal dialog with backdrop. Open via `open=true`
 * attr or `el.show()` / `el.close()`. Closed via the ESC key (native
 * behavior), backdrop click, or `el.close()`.
 *
 * Attributes:
 *   - open: boolean — when set, the dialog is visible (modal)
 *   - size: sm | md | lg (default md — constrains max-width)
 *   - dismissible: boolean — clicking the backdrop closes (default true)
 *
 * Slots:
 *   - header, (default) body, footer
 *
 * Events:
 *   - ship-open, ship-close — CustomEvents bubbling from the host
 */

import { LitElement, css, html } from 'lit'
export class ShipDialog extends LitElement {
  static styles = css`
    :host { display: contents; }
    dialog {
      padding: 0;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      background: var(--card);
      color: var(--card-foreground);
      max-width: 560px;
      width: 100%;
      max-height: calc(100vh - 64px);
      box-shadow: 0 24px 64px -16px oklch(0 0 0 / 30%);
      font-family: var(--font-body);
    }
    dialog::backdrop {
      background: oklch(0 0 0 / 40%);
      backdrop-filter: blur(2px);
    }
    :host([size='sm']) dialog { max-width: 400px; }
    :host([size='lg']) dialog { max-width: 800px; }

    .wrap { display: flex; flex-direction: column; max-height: inherit; }
    .header {
      padding: 14px 20px;
      border-bottom: 1px solid var(--border);
      font-size: 14px; font-weight: 600;
    }
    .body {
      padding: 16px 20px;
      overflow-y: auto;
      font-size: 13px;
      line-height: 1.5;
    }
    .footer {
      padding: 12px 20px;
      border-top: 1px solid var(--border);
      display: flex; gap: 8px; justify-content: flex-end;
    }
  `

  static properties = {
    open: { type: Boolean, reflect: true },
    size: { type: String, reflect: true },
    dismissible: { type: Boolean },
  }

  open = false
  size: 'sm' | 'md' | 'lg' = 'md'
  dismissible = true
  get dialogEl(): HTMLDialogElement { return this.renderRoot.querySelector('dialog')! as HTMLDialogElement }

  updated(changed: Map<string, unknown>) {
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

  render() {
    return html`
      <dialog @close=${this.onClose} @click=${this.onBackdropClick}>
        <div class="wrap">
          <div class="header"><slot name="header"></slot></div>
          <div class="body"><slot></slot></div>
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
