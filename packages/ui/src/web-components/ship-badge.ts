/**
 * <ship-badge>
 *
 * Small pill for status / category. Replaces inline `.badge.badge-*` from
 * pipelines + specs views. The tone vocabulary maps the per-view tone
 * names (dispatched/completed/failed/etc.) to semantic tones at the
 * caller — this element only knows tones.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     (default: muted)
 *   - size: sm | md (default: md)
 *
 * Slots:
 *   - (default) label text
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { toneVars } from './util'
import type { Tone } from './util'

@customElement('ship-badge')
export class ShipBadge extends LitElement {
  static styles = [
    toneVars,
    css`
      :host {
        display: inline-flex;
        vertical-align: middle;
      }
      .badge {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 2px 8px;
        font-family: var(--font-body);
        font-size: 10px;
        font-weight: 600;
        line-height: 1.4;
        letter-spacing: 0.04em;
        text-transform: uppercase;
        border-radius: 4px;
        background: var(--tone-bg);
        color: var(--tone-fg);
        border: 1px solid transparent;
        white-space: nowrap;
      }
      :host([size='sm']) .badge {
        padding: 1px 6px;
        font-size: 9px;
      }
    `,
  ]

  @property({ reflect: true }) tone: Tone = 'muted'
  @property({ reflect: true }) size: 'sm' | 'md' = 'md'

  render() {
    return html`<span class="badge" part="badge"><slot></slot></span>`
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-badge': ShipBadge
  }
}
