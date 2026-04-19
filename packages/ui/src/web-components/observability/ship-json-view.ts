/**
 * <ship-json-view>
 *
 * Collapsible JSON tree. Each object/array level is togglable.
 *
 * Attributes:
 *   - data: JSON string (parsed at render)
 *   - expanded: boolean — force-expand all levels
 *   - depth: number — initial expansion depth (default 1)
 */

import { LitElement, css, html, nothing } from 'lit'
import { MONO } from '../util'

type JsonValue = null | boolean | number | string | JsonValue[] | { [k: string]: JsonValue }

export class ShipJsonView extends LitElement {
  static styles = css`
    :host {
      display: block;
      font-family: ${MONO};
      font-size: var(--text-sm);
      line-height: var(--leading-normal);
      color: var(--foreground);
    }
    .node { display: block; }
    .branch { margin-left: 12px; border-left: 1px dashed var(--border); padding-left: 10px; }
    details > summary {
      list-style: none;
      cursor: pointer;
      user-select: none;
      display: inline-flex;
      align-items: baseline;
      gap: 4px;
    }
    details > summary::-webkit-details-marker { display: none; }
    .caret {
      display: inline-block;
      width: 10px;
      color: var(--muted-foreground);
      transition: transform var(--duration-fast) var(--ease-out);
    }
    details[open] > summary .caret { transform: rotate(90deg); }
    .key { color: var(--primary); font-weight: var(--font-weight-medium); }
    .s { color: var(--success); }
    .n { color: var(--info); }
    .b { color: var(--accent); font-weight: var(--font-weight-medium); }
    .null { color: var(--muted-foreground); font-style: italic; }
    .meta { color: var(--muted-foreground); font-size: var(--text-xs); margin-left: 4px; }
    .error { color: var(--destructive); }
  `

  static properties = {
    data: { type: String, reflect: true },
    expanded: { type: Boolean, reflect: true },
    depth: { type: Number, reflect: true },
  }

  data = ''
  expanded = false
  depth = 1

  private _renderValue(v: JsonValue, key: string | null, level: number): unknown {
    const keyPrefix = key !== null ? html`<span class="key">"${key}"</span>: ` : nothing
    if (v === null) return html`${keyPrefix}<span class="null">null</span>`
    if (typeof v === 'string') return html`${keyPrefix}<span class="s">"${v}"</span>`
    if (typeof v === 'number') return html`${keyPrefix}<span class="n">${v}</span>`
    if (typeof v === 'boolean') return html`${keyPrefix}<span class="b">${String(v)}</span>`
    const isArray = Array.isArray(v)
    const entries = isArray ? v.map((x, i) => [String(i), x] as const) : Object.entries(v)
    const open = this.expanded || level < this.depth
    const open1 = isArray ? '[' : '{'
    const close1 = isArray ? ']' : '}'
    const count = entries.length
    if (count === 0) return html`${keyPrefix}${open1}${close1}`
    return html`
      <details ?open=${open}>
        <summary>
          <span class="caret">▶</span>
          ${keyPrefix}${open1}<span class="meta">${count} ${isArray ? 'items' : 'keys'}</span>
        </summary>
        <div class="branch">
          ${entries.map(
            ([k, val]) => html`<div class="node">${this._renderValue(val, isArray ? null : k, level + 1)}</div>`,
          )}
        </div>
        ${close1}
      </details>
    `
  }

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'tree')
    if (!this.hasAttribute('aria-label')) this.setAttribute('aria-label', 'JSON tree')
  }

  render() {
    if (!this.data) return html``
    let parsed: JsonValue
    try {
      parsed = JSON.parse(this.data)
    } catch (e) {
      return html`<span class="error" part="error" role="alert">Invalid JSON: ${(e as Error).message}</span>`
    }
    return html`<div part="root">${this._renderValue(parsed, null, 0)}</div>`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-json-view')) {
  customElements.define('ship-json-view', ShipJsonView)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-json-view': ShipJsonView
  }
}
