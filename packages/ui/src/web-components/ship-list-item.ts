/**
 * <ship-list-item>
 *
 * Generic selectable row. Replaces ship-file-row (deleted — too
 * domain-specific). Matches specs `.spec-row` button-row pattern.
 *
 * Attributes:
 *   - selected: boolean — reflects aria-selected + highlight
 *   - disabled: boolean
 *
 * Slots:
 *   - leading — icon/status prefix
 *   - (default) — primary content
 *   - trailing — right-side content
 *   - meta — secondary line below primary
 *
 * Events:
 *   - ship-select — CustomEvent { detail: {} } on click / Enter / Space
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

@customElement('ship-list-item')
export class ShipListItem extends LitElement {
  static styles = css`
    :host { display: block; }
    button {
      width: 100%;
      text-align: left;
      background: transparent;
      border: 1px solid transparent;
      border-radius: 6px;
      padding: 8px 10px;
      cursor: pointer;
      display: flex;
      flex-direction: column;
      gap: 4px;
      color: inherit;
      font: inherit;
    }
    button:hover:not(:disabled) { background: var(--muted); border-color: var(--border); }
    button:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }
    :host([selected]) button {
      background: var(--muted);
      border-color: var(--primary);
    }
    button:disabled { opacity: 0.6; cursor: not-allowed; }

    .row { display: flex; align-items: center; gap: 8px; }
    .primary { flex: 1; min-width: 0; font-weight: 500; font-size: 13px; }
    .meta {
      display: flex; gap: 6px; align-items: center;
      font-size: 10px; color: var(--muted-foreground);
    }
    ::slotted([slot='leading']), ::slotted([slot='trailing']) {
      display: inline-flex; align-items: center;
    }
  `

  @property({ type: Boolean, reflect: true }) selected = false
  @property({ type: Boolean, reflect: true }) disabled = false

  private onClick(e: Event) {
    if (this.disabled) return
    this.dispatchEvent(new CustomEvent('ship-select', {
      detail: {}, bubbles: true, composed: true,
    }))
    // Don't consume the native click — let outer listeners also react.
    void e
  }

  render() {
    return html`
      <button
        part="button"
        role="option"
        aria-selected=${this.selected ? 'true' : 'false'}
        ?disabled=${this.disabled}
        @click=${this.onClick}
      >
        <div class="row">
          <slot name="leading"></slot>
          <span class="primary"><slot></slot></span>
          <slot name="trailing"></slot>
        </div>
        <div class="meta"><slot name="meta"></slot></div>
      </button>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-list-item': ShipListItem
  }
}
