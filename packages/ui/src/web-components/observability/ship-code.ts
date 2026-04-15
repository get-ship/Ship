/**
 * <ship-code>
 *
 * Inline or block code. Monospace only — no syntax highlighting this pass.
 *
 * Attributes:
 *   - lang: optional language label, shown in block variant header
 *   - block: boolean — block variant (multi-line pre) instead of inline
 *
 * Slots:
 *   - (default) code text
 */

import { LitElement, css, html } from 'lit'
import { MONO } from '../util'

export class ShipCode extends LitElement {
  static styles = css`
    :host {
      display: inline;
      font-family: ${MONO};
      font-size: var(--text-sm);
    }
    :host([block]) { display: block; }

    .inline {
      padding: 1px 5px;
      border-radius: var(--radius-sm);
      background: var(--muted);
      color: var(--foreground);
      border: 1px solid var(--border);
      font-size: 0.92em;
    }
    .block {
      display: block;
      background: var(--muted);
      color: var(--foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      overflow: hidden;
    }
    .block header {
      display: flex;
      justify-content: space-between;
      padding: 4px 10px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      background: color-mix(in oklch, var(--muted) 60%, var(--background));
      border-bottom: 1px solid var(--border);
      text-transform: uppercase;
      letter-spacing: 0.04em;
      font-weight: var(--font-weight-medium);
    }
    pre {
      margin: 0;
      padding: 10px 12px;
      overflow-x: auto;
      line-height: var(--leading-normal);
      font-family: inherit;
    }
  `

  static properties = {
    language: { type: String, reflect: true, attribute: 'lang' },
    block: { type: Boolean, reflect: true },
  }

  language?: string
  block = false

  render() {
    if (this.block) {
      return html`
        <div class="block" part="block" role="region" aria-label=${this.language ? `${this.language} code block` : 'code block'}>
          ${this.language ? html`<header part="label">${this.language}</header>` : null}
          <pre part="pre"><slot></slot></pre>
        </div>
      `
    }
    return html`<code class="inline" part="inline"><slot></slot></code>`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-code')) {
  customElements.define('ship-code', ShipCode)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-code': ShipCode
  }
}
