/**
 * <ship-link>
 *
 * Styled anchor wrapper. Adds external-link affordances when `external`.
 *
 * Attributes:
 *   - href: target URL.
 *   - external: boolean. Adds target="_blank" rel="noopener noreferrer" and
 *     an external-link icon suffix.
 *   - tone: default | muted (default). `default` uses primary accent.
 *
 * Slots:
 *   - (default) link label.
 */

import { LitElement, css, html } from 'lit'

export type LinkTone = 'default' | 'muted'

export class ShipLink extends LitElement {
  static styles = css`
    :host {
      display: inline;
      font-family: var(--font-body);
      color: var(--primary);
    }
    :host([tone='muted']) { color: var(--muted-foreground); }
    a {
      color: inherit;
      text-decoration: none;
      border-radius: var(--radius-sm);
      display: inline;
      transition: color var(--duration-fast);
    }
    a:hover { text-decoration: underline; text-underline-offset: 2px; }
    a:focus-visible {
      outline: var(--ring-width) solid var(--ring-color);
      outline-offset: var(--ring-offset);
    }
    .ext {
      display: inline-block;
      margin-left: 2px;
      font-size: 0.9em;
      opacity: 0.8;
    }
    .ext svg { width: 0.9em; height: 0.9em; vertical-align: -0.1em; }
  `

  static properties = {
    href: { type: String },
    external: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
  }

  href = ''
  external = false
  tone: LinkTone = 'default'

  render() {
    const target = this.external ? '_blank' : undefined
    const rel = this.external ? 'noopener noreferrer' : undefined
    return html`
      <a href=${this.href}
         part="anchor"
         target=${target ?? ''}
         rel=${rel ?? ''}>
        <slot></slot>
        ${this.external ? html`<span class="ext" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
               stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"/>
          </svg>
        </span>` : null}
      </a>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-link')) {
  customElements.define('ship-link', ShipLink)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-link': ShipLink
  }
}
