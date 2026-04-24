/**
 * <ship-branch-row> — a single branch entry in the branches list.
 *
 * Props:
 *   name      branch name
 *   active    boolean — this is the currently checked-out branch
 *   upstream  tracking remote ref, e.g. "origin/main"
 *   ahead     commits ahead of upstream
 *   behind    commits behind upstream
 *   selected  boolean — highlighted in the detail panel
 *
 * Events:
 *   ship-select  CustomEvent<{ name: string }> — user clicked the row
 */

import { LitElement, css, html } from 'lit'

export class ShipBranchRow extends LitElement {
  static properties = {
    name:     { type: String, reflect: true },
    active:   { type: Boolean, reflect: true },
    upstream: { type: String, reflect: true },
    ahead:    { type: Number, reflect: true },
    behind:   { type: Number, reflect: true },
    selected: { type: Boolean, reflect: true },
  }

  name = ''
  active = false
  upstream: string | null = null
  ahead = 0
  behind = 0
  selected = false

  static styles = css`
    :host {
      display: flex;
      align-items: center;
      gap: 8px;
      width: 100%;
      padding: 7px 12px;
      background: transparent;
      border: 0;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 25%, transparent);
      font-size: 12px;
      color: var(--foreground);
      cursor: pointer;
      text-align: left;
      box-sizing: border-box;
      transition: background var(--duration-fast) ease;
    }
    :host(:hover) { background: color-mix(in oklch, var(--muted) 40%, transparent); }
    :host([selected]) { background: color-mix(in oklch, var(--primary) 8%, transparent); }
    .dot {
      width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
      background: var(--muted-foreground);
    }
    :host([active]) .dot { background: var(--success); }
    .info { flex: 1; min-width: 0; }
    .branch-name {
      font-family: var(--font-mono);
      font-size: 11.5px;
      font-weight: 500;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .upstream {
      font-size: 10px;
      color: var(--muted-foreground);
      margin-top: 1px;
    }
    .counters {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      gap: 1px;
      font-family: var(--font-mono);
      font-size: 10px;
      flex-shrink: 0;
    }
    .up { color: var(--success); }
    .down { color: var(--warning); }
  `

  connectedCallback() {
    super.connectedCallback()
    this.setAttribute('role', 'button')
    this.setAttribute('tabindex', '0')
    this.addEventListener('click', this._onClick)
    this.addEventListener('keydown', this._onKey)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('click', this._onClick)
    this.removeEventListener('keydown', this._onKey)
  }

  private _onClick = () => {
    this.dispatchEvent(new CustomEvent('ship-select', {
      detail: { name: this.name },
      bubbles: true,
      composed: true,
    }))
  }

  private _onKey = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); this._onClick() }
  }

  render() {
    return html`
      <span class="dot"></span>
      <div class="info">
        <div class="branch-name" title="${this.name}">${this.name}</div>
        ${this.upstream ? html`<div class="upstream">${this.upstream}</div>` : ''}
      </div>
      <div class="counters">
        ${this.ahead > 0 ? html`<span class="up">↑${this.ahead}</span>` : ''}
        ${this.behind > 0 ? html`<span class="down">↓${this.behind}</span>` : ''}
      </div>
    `
  }
}

customElements.define('ship-branch-row', ShipBranchRow)
