/**
 * <ship-status-dot>
 *
 * Colored dot for live/online state indicators. Matches pipelines view's
 * `.live-dot` (2.5s pulse, 4px glow ring). The animation runs only when
 * `pulse` is set.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - pulse: boolean — animated pulse
 *   - size: sm | md (default: md = 8px)
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { toneVars } from './util'
import type { Tone } from './util'

@customElement('ship-status-dot')
export class ShipStatusDot extends LitElement {
  static styles = [
    toneVars,
    css`
      :host { display: inline-flex; vertical-align: middle; }
      .dot {
        display: inline-block;
        width: 8px; height: 8px;
        border-radius: 50%;
        background: var(--tone-fg);
        box-shadow: 0 0 0 4px color-mix(in oklch, var(--tone-fg) 20%, transparent);
      }
      :host([size='sm']) .dot { width: 6px; height: 6px; box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-fg) 20%, transparent); }
      :host([pulse]) .dot { animation: pulse 2.5s infinite; }
      @keyframes pulse {
        0%, 100% { opacity: 1; transform: scale(1); }
        50% { opacity: 0.75; transform: scale(0.9); }
      }
    `,
  ]

  @property({ reflect: true }) tone: Tone = 'success'
  @property({ type: Boolean, reflect: true }) pulse = false
  @property({ reflect: true }) size: 'sm' | 'md' = 'md'

  render() {
    return html`<span class="dot" part="dot" aria-hidden="true"></span>`
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-status-dot': ShipStatusDot
  }
}
