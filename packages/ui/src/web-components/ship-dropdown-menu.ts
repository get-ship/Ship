/**
 * <ship-dropdown-menu>
 *
 * Popover with a vertical list of actionable items. Light-DOM children
 * with `role="menuitem"` and a `data-value` attr become selectable rows.
 * Use child <button data-value="..."> elements for best semantics.
 *
 * Attributes:
 *   - open: boolean
 *
 * Slots:
 *   - trigger — the triggering button
 *   - (default) menuitem children
 *
 * Events:
 *   - ship-select — CustomEvent { detail: { value } } when an item is
 *     clicked or activated with Enter/Space.
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

@customElement('ship-dropdown-menu')
export class ShipDropdownMenu extends LitElement {
  static styles = css`
    :host { position: relative; display: inline-flex; }
    .menu {
      position: absolute;
      z-index: 30;
      top: calc(100% + 4px);
      left: 0;
      min-width: 160px;
      padding: 4px;
      background: var(--popover);
      color: var(--popover-foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      box-shadow: 0 8px 24px -8px oklch(0 0 0 / 25%);
      display: flex; flex-direction: column;
      font-family: var(--font-body);
      font-size: 13px;
    }
    :host(:not([open])) .menu { display: none; }
    ::slotted(button), ::slotted([role='menuitem']) {
      text-align: left;
      background: transparent; border: 0;
      padding: 6px 10px;
      border-radius: 4px;
      color: inherit; font: inherit;
      cursor: pointer;
    }
    ::slotted(button:hover), ::slotted([role='menuitem']:hover) {
      background: var(--muted);
    }
  `

  @property({ type: Boolean, reflect: true }) open = false

  private onTriggerClick = () => { this.open = !this.open }

  private onItemClick = (e: Event) => {
    const target = (e.target as HTMLElement).closest('[data-value]') as HTMLElement | null
    if (!target) return
    const value = target.dataset.value || ''
    this.dispatchEvent(new CustomEvent('ship-select', {
      detail: { value }, bubbles: true, composed: true,
    }))
    this.open = false
  }

  private onDocClick = (e: MouseEvent) => {
    if (!this.open) return
    const target = e.composedPath()[0] as Node
    if (!this.contains(target) && target !== this) this.open = false
  }

  connectedCallback() { super.connectedCallback(); document.addEventListener('click', this.onDocClick) }
  disconnectedCallback() { super.disconnectedCallback(); document.removeEventListener('click', this.onDocClick) }

  render() {
    return html`
      <span @click=${this.onTriggerClick}><slot name="trigger"></slot></span>
      <div class="menu" role="menu" @click=${this.onItemClick}>
        <slot></slot>
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-dropdown-menu': ShipDropdownMenu
  }
}
