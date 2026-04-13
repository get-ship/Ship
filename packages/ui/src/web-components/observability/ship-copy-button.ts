/**
 * <ship-copy-button>
 *
 * Copies `value` to the clipboard on click. Shows a checkmark flash for
 * 1.2s after a successful copy.
 *
 * Attributes:
 *   - value: string to copy (required)
 *   - label: accessible label (default: "Copy")
 *
 * Events:
 *   - ship-copy { value: string } — dispatched after a successful copy
 *
 * Slots:
 *   - (default) button label content (falls back to icon + label text)
 */

import { LitElement, css, html } from 'lit'

export class ShipCopyButton extends LitElement {
  static styles = css`
    :host { display: inline-flex; }
    button {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      font-family: var(--font-body);
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      padding: 4px 8px;
      color: var(--muted-foreground);
      background: transparent;
      border: 1px solid var(--border);
      border-radius: var(--radius-sm);
      cursor: pointer;
      transition:
        color var(--duration-fast) var(--ease-out),
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out);
    }
    button:hover { color: var(--foreground); background: var(--muted); }
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([copied]) button { color: var(--success); border-color: var(--success); }
    svg { width: 12px; height: 12px; }
  `

  static properties = {
    value: { type: String, reflect: true },
    label: { type: String, reflect: true },
    copied: { type: Boolean, reflect: true },
  }

  value = ''
  label = 'Copy'
  copied = false
  private _resetTimer?: ReturnType<typeof setTimeout>

  disconnectedCallback(): void {
    super.disconnectedCallback()
    if (this._resetTimer) clearTimeout(this._resetTimer)
  }

  private async _copy() {
    try {
      await navigator.clipboard.writeText(this.value)
    } catch {
      const ta = document.createElement('textarea')
      ta.value = this.value
      document.body.appendChild(ta)
      ta.select()
      try { document.execCommand('copy') } catch { /* ignore */ }
      ta.remove()
    }
    this.copied = true
    this.dispatchEvent(
      new CustomEvent('ship-copy', {
        detail: { value: this.value },
        bubbles: true,
        composed: true,
      }),
    )
    if (this._resetTimer) clearTimeout(this._resetTimer)
    this._resetTimer = setTimeout(() => { this.copied = false }, 1200)
  }

  render() {
    const icon = this.copied
      ? html`<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><polyline points="3 8 7 12 13 4"></polyline></svg>`
      : html`<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><rect x="4" y="4" width="9" height="9" rx="1"></rect><path d="M11 4V3a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v7a1 1 0 0 0 1 1h1"></path></svg>`
    return html`
      <button
        type="button"
        part="button"
        aria-label=${this.label}
        @click=${this._copy}
      >
        ${icon}
        <slot>${this.copied ? 'Copied' : this.label}</slot>
      </button>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-copy-button')) {
  customElements.define('ship-copy-button', ShipCopyButton)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-copy-button': ShipCopyButton
  }
}
