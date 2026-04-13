/**
 * <ship-kbd>
 *
 * Displays a keyboard shortcut. Splits the `keys` string on `+` and renders
 * each token as a styled <kbd> with `+` separators.
 *
 * Attributes:
 *   - keys: plus-separated shortcut string, e.g. "Cmd+K"
 */

import { LitElement, css, html } from 'lit'
import { MONO } from '../util'

export class ShipKbd extends LitElement {
  static styles = css`
    :host { display: inline-flex; align-items: center; gap: 3px; vertical-align: middle; }
    kbd {
      display: inline-flex; align-items: center; justify-content: center;
      min-width: 18px;
      padding: 1px 5px;
      font-family: ${MONO};
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      line-height: 1.4;
      color: var(--foreground);
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: var(--radius-sm);
      box-shadow: var(--shadow-xs);
    }
    .plus { color: var(--muted-foreground); font-size: var(--text-xs); }
  `

  static properties = {
    keys: { type: String },
  }

  keys = ''

  render() {
    const tokens = this.keys.split('+').map(s => s.trim()).filter(Boolean)
    return html`${tokens.map((t, i) => html`
      ${i > 0 ? html`<span class="plus" aria-hidden="true">+</span>` : null}
      <kbd part="key">${t}</kbd>
    `)}`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-kbd')) {
  customElements.define('ship-kbd', ShipKbd)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-kbd': ShipKbd
  }
}
