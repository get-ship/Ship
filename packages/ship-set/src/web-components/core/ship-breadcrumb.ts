/**
 * <ship-breadcrumb> + <ship-breadcrumb-item>
 *
 * Path navigation. Chevron separators are inserted between items.
 *
 * <ship-breadcrumb>
 *   Slots:
 *     - (default) one or more <ship-breadcrumb-item> children.
 *
 * <ship-breadcrumb-item>
 *   Attributes:
 *     - href: optional; when present, renders an <a>.
 *     - active: boolean; marks last / current item.
 *   Slots:
 *     - (default) label text.
 */

import { LitElement, css, html } from 'lit'

export class ShipBreadcrumbItem extends LitElement {
  static styles = css`
    :host {
      display: inline-flex; align-items: center;
      font-family: var(--font-body);
      font-size: var(--text-base);
      line-height: var(--leading-normal);
      color: var(--muted-foreground);
    }
    :host([active]) { color: var(--foreground); font-weight: var(--font-weight-medium); }
    a, .label {
      display: inline-flex; align-items: center;
      color: inherit;
      text-decoration: none;
      padding: 2px 4px;
      border-radius: var(--radius-sm);
      transition: color var(--duration-fast);
    }
    a:hover { color: var(--foreground); }
    a:focus-visible {
      outline: var(--ring-width) solid var(--ring-color);
      outline-offset: var(--ring-offset);
    }
  `

  static properties = {
    href: { type: String },
    active: { type: Boolean, reflect: true },
  }

  href = ''
  active = false

  render() {
    if (this.href && !this.active) {
      return html`<a href=${this.href} part="link"><slot></slot></a>`
    }
    return html`<span class="label" part="label" aria-current=${this.active ? 'page' : 'false'}><slot></slot></span>`
  }
}

export class ShipBreadcrumb extends LitElement {
  static styles = css`
    :host { display: flex; align-items: center; }
    nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
    .sep {
      display: inline-flex; align-items: center; justify-content: center;
      width: 12px; height: 12px;
      color: var(--muted-foreground);
      opacity: 0.7;
    }
    .sep svg { width: 12px; height: 12px; }
  `

  private get items(): ShipBreadcrumbItem[] {
    return Array.from(this.querySelectorAll(':scope > ship-breadcrumb-item')) as ShipBreadcrumbItem[]
  }

  connectedCallback() {
    super.connectedCallback()
    this._observer = new MutationObserver(() => this.requestUpdate())
    this._observer.observe(this, { childList: true })
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this._observer?.disconnect()
  }

  private _observer: MutationObserver | null = null

  render() {
    const count = this.items.length
    const slots: unknown[] = []
    for (let i = 0; i < count; i++) {
      slots.push(html`<slot name="i${i}"></slot>`)
      if (i < count - 1) {
        slots.push(html`<span class="sep" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
               stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6-6 6"/></svg>
        </span>`)
      }
    }
    // Assign light-DOM items to named slots.
    this.items.forEach((el, i) => el.setAttribute('slot', `i${i}`))
    return html`<nav aria-label="Breadcrumb" part="nav">${slots}</nav>`
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-breadcrumb-item'))
    customElements.define('ship-breadcrumb-item', ShipBreadcrumbItem)
  if (!customElements.get('ship-breadcrumb'))
    customElements.define('ship-breadcrumb', ShipBreadcrumb)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-breadcrumb': ShipBreadcrumb
    'ship-breadcrumb-item': ShipBreadcrumbItem
  }
}
