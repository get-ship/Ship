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
      box-shadow: var(--shadow-md);
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
    :host([placement='bottom-end']) .panel { top: calc(100% + 6px); right: 0; }
    :host([placement='top']) .panel { bottom: calc(100% + 6px); left: 0; }
    :host([placement='top-end']) .panel { bottom: calc(100% + 6px); right: 0; }
    :host([placement='right']) .panel { left: calc(100% + 6px); top: 0; }
    :host([placement='left']) .panel { right: calc(100% + 6px); top: 0; }
  `

  static properties = {
    open: { type: Boolean, reflect: true },
    placement: { type: String, reflect: true },
  }

  open = false
  placement: 'top' | 'bottom' | 'left' | 'right' = 'bottom'
  private static _idSeq = 0
  private panelId = `ship-popover-panel-${++ShipPopover._idSeq}`
  show() { this.open = true; this.emit('ship-open') }
  hide() { this.open = false; this.emit('ship-close') }
  toggle() { this.open ? this.hide() : this.show() }

  private emit(name: string) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
  }

  private triggerEl(): HTMLElement | null {
    const slot = this.renderRoot.querySelector('slot[name="trigger"]') as HTMLSlotElement | null
    const assigned = slot?.assignedElements({ flatten: true }) as HTMLElement[] | undefined
    return assigned?.[0] ?? null
  }

  private setupTrigger = () => {
    const t = this.triggerEl()
    if (!t) return
    t.setAttribute('aria-haspopup', 'dialog')
    t.setAttribute('aria-expanded', this.open ? 'true' : 'false')
    t.setAttribute('aria-controls', this.panelId)
  }

  updated() {
    this.setAttribute('data-state', this.open ? 'open' : 'closed')
    this.setupTrigger()
  }

  private onDocClick = (e: MouseEvent) => {
    if (!this.open) return
    // contains() doesn't cross shadow boundaries — use composedPath so a
    // click on a web-component trigger (ship-button) doesn't register as
    // "outside" and close the popover immediately after opening.
    if (!e.composedPath().includes(this)) this.hide()
  }

  private onKeydown = (e: KeyboardEvent) => {
    if (this.open && e.key === 'Escape') {
      e.preventDefault()
      this.hide()
      this.triggerEl()?.focus()
    }
  }

  connectedCallback() {
    super.connectedCallback()
    document.addEventListener('click', this.onDocClick)
    this.addEventListener('keydown', this.onKeydown)
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener('click', this.onDocClick)
    this.removeEventListener('keydown', this.onKeydown)
  }

  render() {
    return html`
      <span @click=${() => this.toggle()}><slot name="trigger" @slotchange=${this.setupTrigger}></slot></span>
      <div
        id=${this.panelId}
        class="panel"
        role="dialog"
        aria-modal="false"
        aria-hidden=${this.open ? 'false' : 'true'}
      ><slot></slot></div>
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
