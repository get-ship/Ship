/**
 * <ship-tree-view> + <ship-tree-node>
 *
 * Recursive expandable hierarchy.
 *
 * <ship-tree-view>
 *   Slots:
 *     - (default) <ship-tree-node> children
 *
 * <ship-tree-node>
 *   Attributes:
 *     - label: node label text
 *     - expanded: boolean
 *     - selected: boolean
 *     - leaf: boolean — hides the toggle caret
 *
 *   Events:
 *     - ship-toggle { expanded: boolean }
 *     - ship-select {}
 *
 *   Slots:
 *     - (default) nested ship-tree-node children
 *     - icon — leading icon
 *     - trailing — trailing content (badge, count, etc.)
 */

import { LitElement, css, html } from 'lit'

export class ShipTreeView extends LitElement {
  static styles = css`
    :host {
      display: block;
      font-family: var(--font-body);
      font-size: var(--text-sm);
      color: var(--foreground);
    }
  `
  connectedCallback(): void {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'tree')
  }
  render() { return html`<slot></slot>` }
}

export class ShipTreeNode extends LitElement {
  static styles = css`
    :host { display: block; }
    .row {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 3px 6px;
      border-radius: var(--radius-sm);
      cursor: pointer;
      outline: none;
      color: var(--foreground);
      transition: background-color var(--duration-fast) var(--ease-out);
    }
    .row:hover { background: var(--muted); }
    :host([selected]) .row {
      background: color-mix(in oklch, var(--primary) 14%, transparent);
      color: var(--foreground);
      font-weight: var(--font-weight-medium);
    }
    .row:focus-visible {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    .caret {
      display: inline-flex;
      width: 12px; height: 12px;
      color: var(--muted-foreground);
      transition: transform var(--duration-fast) var(--ease-out);
    }
    :host([expanded]) .caret { transform: rotate(90deg); }
    :host([leaf]) .caret { visibility: hidden; }
    .label { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .trailing { margin-left: auto; color: var(--muted-foreground); font-size: var(--text-xs); }
    .children { margin-left: 16px; border-left: 1px dashed var(--border); padding-left: 6px; }
    :host(:not([expanded])) .children { display: none; }
  `

  static properties = {
    label: { type: String, reflect: true },
    expanded: { type: Boolean, reflect: true },
    selected: { type: Boolean, reflect: true },
    leaf: { type: Boolean, reflect: true },
  }

  label = ''
  expanded = false
  selected = false
  leaf = false

  connectedCallback(): void {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'treeitem')
  }

  private _toggle = (e: Event) => {
    if (this.leaf) return
    e.stopPropagation()
    this.expanded = !this.expanded
    this.dispatchEvent(
      new CustomEvent('ship-toggle', {
        detail: { expanded: this.expanded },
        bubbles: true,
        composed: true,
      }),
    )
  }

  private _select = () => {
    this.dispatchEvent(new CustomEvent('ship-select', { bubbles: true, composed: true, detail: {} }))
  }

  private _onKey = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); this._select() }
    else if (e.key === 'ArrowRight' && !this.leaf && !this.expanded) { e.preventDefault(); this._toggle(e) }
    else if (e.key === 'ArrowLeft' && this.expanded) { e.preventDefault(); this._toggle(e) }
  }

  render() {
    return html`
      <div
        class="row"
        part="row"
        tabindex="0"
        aria-expanded=${this.leaf ? 'false' : String(this.expanded)}
        aria-selected=${String(this.selected)}
        @click=${this._select}
        @keydown=${this._onKey}
      >
        <span class="caret" part="caret" @click=${this._toggle} aria-hidden="true">
          <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="4 3 8 6 4 9"></polyline></svg>
        </span>
        <slot name="icon"></slot>
        <span class="label" part="label">${this.label}</span>
        <span class="trailing"><slot name="trailing"></slot></span>
      </div>
      <div class="children" part="children" role="group"><slot></slot></div>
    `
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-tree-view')) customElements.define('ship-tree-view', ShipTreeView)
  if (!customElements.get('ship-tree-node')) customElements.define('ship-tree-node', ShipTreeNode)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-tree-view': ShipTreeView
    'ship-tree-node': ShipTreeNode
  }
}
