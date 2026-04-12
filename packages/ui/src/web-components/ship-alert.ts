/**
 * <ship-alert>
 *
 * Inline status banner. Tone-driven surface.
 *
 * Attributes:
 *   - tone: info | success | warning | destructive | muted (default: info)
 *   - dismissible: boolean — shows a close button; dismissing emits
 *     ship-dismiss and removes the host from the DOM.
 *
 * Slots:
 *   - icon
 *   - title
 *   - (default) body
 *   - actions
 *
 * Events:
 *   - ship-dismiss — fires when the dismiss button is clicked
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { toneVars } from './util'
import type { Tone } from './util'

@customElement('ship-alert')
export class ShipAlert extends LitElement {
  static styles = [
    toneVars,
    css`
      :host { display: block; }
      .box {
        display: grid;
        grid-template-columns: auto 1fr auto;
        align-items: start;
        gap: 10px;
        padding: 12px 14px;
        border-radius: var(--radius);
        border: 1px solid var(--tone-border);
        background: var(--tone-bg);
        color: var(--tone-fg);
        font-family: var(--font-body);
        font-size: 13px;
        line-height: 1.5;
      }
      .content { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
      .title { font-weight: 600; }
      .body { color: var(--foreground); }
      .close {
        background: transparent; border: 0; color: inherit;
        cursor: pointer; font-size: 14px; padding: 2px 6px;
        border-radius: 4px;
      }
      .close:hover { background: color-mix(in oklch, var(--tone-fg) 12%, transparent); }
      .actions { display: flex; gap: 6px; margin-top: 6px; }
      ::slotted([slot='icon']) { display: inline-flex; }
    `,
  ]

  @property({ reflect: true }) tone: Tone = 'info'
  @property({ type: Boolean, reflect: true }) dismissible = false

  private onDismiss() {
    this.dispatchEvent(new CustomEvent('ship-dismiss', {
      bubbles: true, composed: true,
    }))
    this.remove()
  }

  render() {
    return html`
      <div class="box" role="status">
        <span><slot name="icon"></slot></span>
        <div class="content">
          <div class="title"><slot name="title"></slot></div>
          <div class="body"><slot></slot></div>
          <div class="actions"><slot name="actions"></slot></div>
        </div>
        ${this.dismissible
          ? html`<button class="close" aria-label="Dismiss" @click=${this.onDismiss}>×</button>`
          : html`<span></span>`}
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-alert': ShipAlert
  }
}
