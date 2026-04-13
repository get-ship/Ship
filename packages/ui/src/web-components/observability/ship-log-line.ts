/**
 * <ship-log-line>
 *
 * Single log row in a monospace stream. Level maps to tone.
 *
 * Attributes:
 *   - level: debug | info | warn | error (default: info)
 *   - timestamp: ISO string or pre-formatted relative time (optional)
 *   - source: optional source label (e.g., stdout, daemon)
 *
 * Slots:
 *   - (default) message body
 */

import { LitElement, css, html } from 'lit'
import { MONO } from '../util'

type Level = 'debug' | 'info' | 'warn' | 'error'

export class ShipLogLine extends LitElement {
  static styles = css`
    :host {
      display: grid;
      grid-template-columns: auto auto 1fr;
      gap: 8px;
      align-items: baseline;
      font-family: ${MONO};
      font-size: var(--text-sm);
      line-height: var(--leading-normal);
      padding: 2px 8px;
      border-left: 2px solid transparent;
      color: var(--foreground);
    }
    :host([level='debug']) { border-left-color: var(--muted-foreground); color: var(--muted-foreground); }
    :host([level='info']) { border-left-color: var(--info); }
    :host([level='warn']) { border-left-color: var(--warning); }
    :host([level='error']) {
      border-left-color: var(--destructive);
      background: color-mix(in oklch, var(--destructive) 6%, transparent);
    }
    .ts { color: var(--muted-foreground); font-size: var(--text-xs); white-space: nowrap; }
    .src {
      color: var(--muted-foreground);
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      text-transform: uppercase;
      letter-spacing: 0.04em;
      white-space: nowrap;
    }
    .msg { white-space: pre-wrap; word-break: break-word; min-width: 0; }
  `

  static properties = {
    level: { type: String, reflect: true },
    timestamp: { type: String, reflect: true },
    source: { type: String, reflect: true },
  }

  level: Level = 'info'
  timestamp?: string
  source?: string

  render() {
    return html`
      ${this.timestamp ? html`<span class="ts" part="timestamp">${this.timestamp}</span>` : html`<span></span>`}
      ${this.source ? html`<span class="src" part="source">${this.source}</span>` : html`<span></span>`}
      <span class="msg" part="message"><slot></slot></span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-log-line')) {
  customElements.define('ship-log-line', ShipLogLine)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-log-line': ShipLogLine
  }
}
