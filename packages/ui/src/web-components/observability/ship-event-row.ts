/**
 * <ship-event-row>
 *
 * Structured event row — actor + verb + target + timestamp + optional payload.
 *
 * Attributes:
 *   - actor: subject of the event
 *   - verb: action description
 *   - target: object of the action
 *   - at: ISO timestamp or relative label
 *
 * Slots:
 *   - (default) nested payload block rendered below the summary
 */

import { LitElement, css, html } from 'lit'

export class ShipEventRow extends LitElement {
  static styles = css`
    :host {
      display: block;
      padding: 8px 12px;
      border-bottom: 1px solid var(--border);
      font-family: var(--font-body);
      font-size: var(--text-sm);
      line-height: var(--leading-normal);
    }
    .head { display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px; }
    .actor { font-weight: var(--font-weight-semibold); color: var(--foreground); }
    .verb { color: var(--muted-foreground); }
    .target { font-weight: var(--font-weight-medium); color: var(--accent); }
    .at {
      margin-left: auto;
      color: var(--muted-foreground);
      font-size: var(--text-xs);
      white-space: nowrap;
    }
    .payload { margin-top: 6px; }
    .payload:empty { display: none; }
  `

  static properties = {
    actor: { type: String, reflect: true },
    verb: { type: String, reflect: true },
    target: { type: String, reflect: true },
    at: { type: String, reflect: true },
  }

  actor?: string
  verb?: string
  target?: string
  at?: string

  render() {
    return html`
      <div class="head" part="head">
        ${this.actor ? html`<span class="actor" part="actor">${this.actor}</span>` : null}
        ${this.verb ? html`<span class="verb" part="verb">${this.verb}</span>` : null}
        ${this.target ? html`<span class="target" part="target">${this.target}</span>` : null}
        ${this.at ? html`<span class="at" part="at">${this.at}</span>` : null}
      </div>
      <div class="payload" part="payload"><slot></slot></div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-event-row')) {
  customElements.define('ship-event-row', ShipEventRow)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-event-row': ShipEventRow
  }
}
