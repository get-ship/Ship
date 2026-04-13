/**
 * <ship-popover>
 *
 * Lightweight anchored popover. Toggle via the `open` attribute or
 * `el.toggle()` / `el.show()` / `el.hide()`. Uses the native top-layer
 * popover API when available, falls back to absolute positioning.
 *
 * Attributes:
 *   - open: boolean
 *   - placement: top | bottom | left | right (default bottom)
 *
 * Slots:
 *   - trigger — the anchoring element (button, etc.)
 *   - (default) popover body
 *
 * Events:
 *   - ship-open, ship-close
 */

import { LitElement, css, html } from 'lit'
export class ShipPopover extends LitElement {
  static styles = css`
    :host { position: relative; display: inline-flex; }
    .panel {
      position: absolute;
      z-index: var(--z-popover);
      min-width: 180px;
      padding: 6px;
      background: var(--popover);
      color: var(--popover-foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      box-shadow: var(--shadow-lg);
      font-family: var(--font-body);
      font-size: var(--text-base);
      opacity: 0;
      transform: translateY(-4px);
      pointer-events: none;
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    :host([open]) .panel {
      opacity: 1;
      transform: translateY(0);
      pointer-events: auto;
    }
    :host([placement='bottom']) .panel,
    :host(:not([placement])) .panel { top: calc(100% + 6px); left: 0; }
    :host([placement='top']) .panel { bottom: calc(100% + 6px); left: 0; }
    :host([placement='right']) .panel { left: calc(100% + 6px); top: 0; }
    :host([placement='left']) .panel { right: calc(100% + 6px); top: 0; }
  `

  static properties = {
    open: { type: Boolean, reflect: true },
    placement: { type: String, reflect: true },
  }

  open = false
  placement: 'top' | 'bottom' | 'left' | 'right' = 'bottom'
  show() { this.open = true; this.emit('ship-open') }
  hide() { this.open = false; this.emit('ship-close') }
  toggle() { this.open ? this.hide() : this.show() }

  private emit(name: string) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
  }

  private onDocClick = (e: MouseEvent) => {
    if (!this.open) return
    const target = e.composedPath()[0] as Node
    if (!this.contains(target) && target !== this) this.hide()
  }

  connectedCallback() { super.connectedCallback(); document.addEventListener('click', this.onDocClick) }
  disconnectedCallback() { super.disconnectedCallback(); document.removeEventListener('click', this.onDocClick) }

  render() {
    return html`
      <span @click=${() => this.toggle()}><slot name="trigger"></slot></span>
      <div class="panel" role="dialog"><slot></slot></div>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-popover')) {
  customElements.define('ship-popover', ShipPopover)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-popover': ShipPopover
  }
}
