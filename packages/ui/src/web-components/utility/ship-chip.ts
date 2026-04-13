/**
 * <ship-chip>
 *
 * Small removable / selectable tag. Filter-chip capable via `selected`.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - size: sm | md (default md)
 *   - removable: boolean — renders an × button
 *   - selected: boolean — filter-chip active style
 *
 * Slots:
 *   - leading: icon / avatar before label
 *   - (default) label
 *
 * Events:
 *   - ship-remove — fires when × is clicked. { detail: {} }
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from '../util'
import type { Tone } from '../util'

export class ShipChip extends LitElement {
  static styles = [
    toneVars,
    css`
      :host { display: inline-flex; vertical-align: middle; }
      .chip {
        display: inline-flex; align-items: center; gap: 6px;
        padding: 4px 10px;
        font-family: var(--font-body);
        font-size: var(--text-sm);
        font-weight: var(--font-weight-medium);
        line-height: var(--leading-tight);
        border-radius: var(--radius-full);
        background: var(--tone-bg);
        color: var(--tone-fg);
        border: 1px solid var(--tone-border);
        white-space: nowrap;
      }
      :host([size='sm']) .chip { padding: 2px 8px; font-size: var(--text-xs); gap: 4px; }
      :host([selected]) .chip {
        background: color-mix(in oklch, var(--tone-fg) 22%, transparent);
        border-color: var(--tone-fg);
      }
      .remove {
        display: inline-flex; align-items: center; justify-content: center;
        width: 16px; height: 16px;
        border: 0; padding: 0; margin-right: -4px;
        background: transparent; color: inherit; cursor: pointer;
        border-radius: var(--radius-full);
        font-size: 14px; line-height: 1;
      }
      .remove:hover { background: color-mix(in oklch, var(--tone-fg) 18%, transparent); }
      .remove:focus-visible {
        outline: var(--ring-width) solid var(--ring-color);
        outline-offset: var(--ring-offset);
      }
      ::slotted([slot='leading']) { display: inline-flex; }
    `,
  ]

  static properties = {
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    removable: { type: Boolean, reflect: true },
    selected: { type: Boolean, reflect: true },
  }

  tone: Tone = 'muted'
  size: 'sm' | 'md' = 'md'
  removable = false
  selected = false

  private onRemove(e: Event) {
    e.stopPropagation()
    this.dispatchEvent(new CustomEvent('ship-remove', {
      detail: {}, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <span class="chip" part="chip">
        <slot name="leading"></slot>
        <slot></slot>
        ${this.removable
          ? html`<button class="remove" part="remove" aria-label="Remove" @click=${this.onRemove}>×</button>`
          : null}
      </span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-chip')) {
  customElements.define('ship-chip', ShipChip)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-chip': ShipChip
  }
}
