/**
 * <ship-card>
 *
 * Generic container matching pipelines `.def-card` / `.run-card` base
 * surface (bg: var(--card), 1px border, var(--radius)). Layout is handled
 * via slots; callers can add their own accent treatments via `tone`.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - variant: default | outline (default: default)
 *     - accent-bar: boolean — renders a 2-stop gradient top-bar on hover
 *       (matches .def-card::before behavior)
 *   - interactive: boolean — hover lift + cursor:pointer (matches .def-card)
 *   - status-rail: boolean — left colored rail (3px), tone-driven (matches
 *     .run-card.is-{status})
 *
 * Slots:
 *   - header
 *   - (default)
 *   - footer
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { toneVars } from './util'
import type { Tone } from './util'

@customElement('ship-card')
export class ShipCard extends LitElement {
  static styles = [
    toneVars,
    css`
      :host {
        display: block;
        background: var(--card);
        color: var(--card-foreground);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        padding: 16px 20px;
        position: relative;
        transition: border-color 140ms, transform 140ms, box-shadow 140ms;
      }
      :host([interactive]) { cursor: pointer; }
      :host([interactive]:hover) {
        border-color: color-mix(in oklch, var(--border) 180%, transparent);
        transform: translateY(-2px);
        box-shadow: 0 8px 24px -8px oklch(0.147 0.004 49.25 / 10%);
      }
      :host([accent-bar])::before {
        content: '';
        position: absolute; top: 0; left: 0; right: 0; height: 2px;
        background: linear-gradient(90deg, var(--primary), var(--accent));
        opacity: 0;
        transition: opacity 160ms;
        border-radius: var(--radius) var(--radius) 0 0;
      }
      :host([accent-bar]:hover)::before { opacity: 1; }

      /* status-rail — 3px left border tinted by tone */
      :host([status-rail]) {
        border-left: 3px solid var(--tone-fg);
        padding-left: 17px;
      }

      :host([variant='outline']) { background: transparent; }

      .header { margin-bottom: 10px; }
      .footer { margin-top: 10px; }
      ::slotted([slot='header']) { display: block; }
    `,
  ]

  @property({ reflect: true }) tone?: Tone
  @property({ reflect: true }) variant: 'default' | 'outline' = 'default'
  @property({ type: Boolean, reflect: true }) interactive = false
  @property({ type: Boolean, reflect: true, attribute: 'accent-bar' }) accentBar = false
  @property({ type: Boolean, reflect: true, attribute: 'status-rail' }) statusRail = false

  render() {
    return html`
      <div class="header"><slot name="header"></slot></div>
      <slot></slot>
      <div class="footer"><slot name="footer"></slot></div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-card': ShipCard
  }
}
