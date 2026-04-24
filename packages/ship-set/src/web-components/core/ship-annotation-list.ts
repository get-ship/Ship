/**
 * <ship-annotation-list>
 *
 * Compact list of staged `AnnotationEvent`s with a per-item remove
 * control. Dispatches `ship-annotation-remove` (detail: { id }) when
 * the user clicks remove. Visual only — the parent layer owns the
 * authoritative staged array.
 *
 * Properties:
 *   - staged: AnnotationEvent[] (JS property, or JSON via `staged` attr)
 */

import { LitElement, css, html } from 'lit'
import type { AnnotationEvent } from '../../utils/annotation-helpers'

export class ShipAnnotationList extends LitElement {
  static styles = css`
    :host {
      display: block;
      font-family: var(--font-body);
      font-size: var(--text-sm);
      color: var(--foreground);
    }
    ol {
      list-style: none;
      margin: 0;
      padding: 0;
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
    li {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 4px 8px;
      border: 1px solid var(--border);
      border-radius: var(--radius-sm);
      background: var(--background);
    }
    .kind {
      font-size: var(--text-xs);
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--muted-foreground);
      min-width: 48px;
    }
    .note {
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .empty {
      color: var(--muted-foreground);
      padding: 4px 8px;
    }
    button {
      background: transparent;
      border: 1px solid var(--border);
      border-radius: var(--radius-sm);
      color: var(--muted-foreground);
      cursor: pointer;
      font-size: var(--text-xs);
      padding: 2px 6px;
    }
    button:hover { color: var(--destructive); border-color: var(--destructive); }
  `

  static properties = {
    staged: { type: Array },
  }

  staged: AnnotationEvent[] = []

  /**
   * When the `staged` attribute is set directly (rather than the
   * property), parse it as JSON so consumers can hydrate from markup.
   */
  attributeChangedCallback(name: string, old: string | null, value: string | null): void {
    if (name === 'staged' && value !== null) {
      try {
        const parsed = JSON.parse(value)
        if (Array.isArray(parsed)) {
          this.staged = parsed as AnnotationEvent[]
          return
        }
      } catch {
        /* fall through to default behavior */
      }
    }
    super.attributeChangedCallback(name, old, value)
  }

  private emitRemove(id: string): void {
    this.dispatchEvent(
      new CustomEvent('ship-annotation-remove', {
        detail: { id },
        bubbles: true,
        composed: true,
      }),
    )
  }

  render() {
    if (!this.staged || this.staged.length === 0) {
      return html`<div class="empty" part="empty">No staged annotations.</div>`
    }
    return html`
      <ol part="list">
        ${this.staged.map(
          (a) => html`
            <li part="item" data-id=${a.id}>
              <span class="kind">${a.kind}</span>
              <span class="note" title=${a.note ?? ''}>${a.note ?? '(no note)'}</span>
              <button
                type="button"
                aria-label="Remove annotation"
                @click=${() => this.emitRemove(a.id)}
              >Remove</button>
            </li>
          `,
        )}
      </ol>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-annotation-list')) {
  customElements.define('ship-annotation-list', ShipAnnotationList)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-annotation-list': ShipAnnotationList
  }
}
