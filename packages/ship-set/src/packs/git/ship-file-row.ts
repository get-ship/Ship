/**
 * <ship-file-row> — a single changed-file entry in the git file list.
 *
 * Props:
 *   path       full file path
 *   basename   filename portion (displayed)
 *   status     M | A | D | R | ? (git status code)
 *   active     boolean — highlighted when this file's diff is showing
 *
 * Events:
 *   ship-select  CustomEvent<{ path: string }> — user clicked the row
 */

import { LitElement, css, html } from 'lit'

const STATUS_LABELS: Record<string, string> = {
  M: 'M', A: 'A', D: 'D', R: 'R', '?': '?',
}

export class ShipFileRow extends LitElement {
  static properties = {
    path:     { type: String, reflect: true },
    basename: { type: String, reflect: true },
    status:   { type: String, reflect: true },
    active:   { type: Boolean, reflect: true },
  }

  path = ''
  basename = ''
  status = 'M'
  active = false

  static styles = css`
    :host {
      display: flex;
      align-items: center;
      gap: 6px;
      width: 100%;
      padding: 5px 12px;
      background: transparent;
      border: 0;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 25%, transparent);
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--muted-foreground);
      cursor: pointer;
      text-align: left;
      box-sizing: border-box;
      transition: background var(--duration-fast) ease, color var(--duration-fast) ease;
    }
    :host(:hover) {
      background: color-mix(in oklch, var(--muted) 40%, transparent);
      color: var(--foreground);
    }
    :host([active]) {
      background: color-mix(in oklch, var(--primary) 8%, transparent);
      color: var(--foreground);
    }
    .badge {
      display: inline-block;
      font-family: var(--font-mono);
      font-size: 9px;
      line-height: 1;
      padding: 2px 4px;
      border-radius: var(--radius-sm);
      font-weight: 700;
      flex-shrink: 0;
      min-width: 16px;
      text-align: center;
    }
    :host([status="M"]) .badge,
    :host([status="S"]) .badge { background: color-mix(in oklch, var(--warning) 15%, transparent); color: var(--warning); }
    :host([status="A"]) .badge { background: color-mix(in oklch, var(--success) 15%, transparent); color: var(--success); }
    :host([status="D"]) .badge { background: color-mix(in oklch, var(--destructive) 15%, transparent); color: var(--destructive); }
    :host([status="?"]) .badge { background: color-mix(in oklch, var(--muted-foreground) 15%, transparent); color: var(--muted-foreground); }
    :host([status="R"]) .badge { background: color-mix(in oklch, var(--info) 15%, transparent); color: var(--info); }
    .name {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      flex: 1;
    }
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
      detail: { path: this.path },
      bubbles: true,
      composed: true,
    }))
  }

  private _onKey = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); this._onClick() }
  }

  render() {
    return html`
      <span class="badge">${STATUS_LABELS[this.status] ?? this.status}</span>
      <span class="name" title="${this.path}">${this.basename || this.path}</span>
    `
  }
}

customElements.define('ship-file-row', ShipFileRow)
