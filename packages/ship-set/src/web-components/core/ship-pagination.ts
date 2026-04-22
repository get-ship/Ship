/**
 * <ship-pagination>
 *
 * Pagination bar with prev/next buttons and page info text.
 *
 * Attributes:
 *   - page: current page (1-based, default 1)
 *   - total: total number of items
 *   - per-page: items per page (default 25)
 *   - tone: default | info | success | warning | destructive | accent
 *
 * Events:
 *   - `ship-change` — CustomEvent { detail: { page: number } }
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from './util'
import type { Tone } from './util'

export class ShipPagination extends LitElement {
  static styles = [
    toneVars,
    css`
      :host { display: block; }
      .bar {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 0.8125rem;
        font-family: var(--font-body);
      }
      .info {
        color: var(--muted-foreground);
        user-select: none;
        white-space: nowrap;
      }
      .spacer { flex: 1; }
    `,
  ]

  static properties = {
    page: { type: Number, reflect: true },
    total: { type: Number, reflect: true },
    perPage: { type: Number, attribute: 'per-page', reflect: true },
    tone: { type: String, reflect: true },
  }

  page = 1
  total = 0
  perPage = 25
  tone?: Tone

  private get _totalPages(): number {
    return this.total > 0 ? Math.ceil(this.total / this.perPage) : 1
  }

  private _emitChange(newPage: number) {
    this.page = newPage
    this.dispatchEvent(
      new CustomEvent('ship-change', {
        detail: { page: newPage },
        bubbles: true,
        composed: true,
      }),
    )
  }

  private _prev() {
    if (this.page > 1) this._emitChange(this.page - 1)
  }

  private _next() {
    if (this.page < this._totalPages) this._emitChange(this.page + 1)
  }

  render() {
    const tp = this._totalPages
    const start = (this.page - 1) * this.perPage + 1
    const end = Math.min(this.page * this.perPage, this.total)
    const toneAttr = this.tone && this.tone !== 'default' ? this.tone : undefined

    return html`
      <nav class="bar" aria-label="Pagination">
        <ship-button
          variant="outline"
          size="sm"
          tone=${toneAttr ?? ''}
          ?disabled=${this.page <= 1}
          @click=${this._prev}
          aria-label="Previous page"
        >Prev</ship-button>
        <span class="info">
          Page ${this.page} of ${tp}${this.total > 0
            ? html` <span>(${start}\u2013${end} of ${this.total})</span>`
            : ''}
        </span>
        <span class="spacer"></span>
        <ship-button
          variant="outline"
          size="sm"
          tone=${toneAttr ?? ''}
          ?disabled=${this.page >= tp}
          @click=${this._next}
          aria-label="Next page"
        >Next</ship-button>
      </nav>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-pagination')) {
  customElements.define('ship-pagination', ShipPagination)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-pagination': ShipPagination
  }
}
