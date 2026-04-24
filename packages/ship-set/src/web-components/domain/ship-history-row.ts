/**
 * <ship-history-row>
 *
 * A single row in the git commit history list. Renders a graph lane SVG,
 * author avatar, commit message, and +/- stats.
 *
 * Attributes:
 *   sha         — short SHA (7 chars)
 *   subject     — full commit subject line
 *   author      — author display name
 *   timestamp   — human-readable relative time (e.g. "3h ago")
 *   additions   — number of added lines
 *   deletions   — number of deleted lines
 *   active      — boolean, reflects selected/HEAD state
 *   human       — boolean, human author (vs agent); affects avatar colour
 *
 * Events:
 *   ship-select — CustomEvent<{ sha: string }>, bubbles + composed.
 */

import { LitElement, css, html } from 'lit'
import { MONO } from '../core/util'

export class ShipHistoryRow extends LitElement {
  static styles = css`
    :host {
      display: flex;
      align-items: center;
      gap: 0;
      cursor: pointer;
      border-bottom: 1px solid var(--border-subtle);
      transition: background var(--duration-fast) ease;
      font-family: var(--font-body);
    }
    :host(:hover) { background: color-mix(in oklch, var(--muted) 50%, transparent); }
    :host([active]) { background: color-mix(in oklch, var(--primary) 8%, transparent); }

    /* Graph lane */
    .lane {
      width: 44px;
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      align-self: stretch;
    }
    .lane svg { display: block; }

    /* Author avatar */
    .avatar {
      width: 22px;
      height: 22px;
      border-radius: 50%;
      flex-shrink: 0;
      display: grid;
      place-items: center;
      font-size: 9px;
      font-weight: 600;
      color: var(--primary-foreground);
      background: color-mix(in oklch, var(--primary) 70%, var(--muted));
      text-transform: uppercase;
    }
    :host([human]) .avatar {
      background: conic-gradient(from 40deg, var(--status-blue), var(--accent));
    }

    /* Message area */
    .msg {
      flex: 1;
      min-width: 0;
      padding: 8px 6px 8px 10px;
    }
    .title {
      font-size: 12.5px;
      font-weight: 500;
      color: var(--foreground);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      line-height: 1.35;
    }
    :host([active]) .title { color: var(--primary); }
    .scope { color: var(--primary); opacity: 0.85; }
    .meta {
      display: flex;
      gap: 8px;
      align-items: center;
      margin-top: 2px;
      font-family: ${MONO};
      font-size: 10.5px;
      color: var(--muted-foreground);
    }

    /* Stats */
    .stats {
      display: flex;
      gap: 6px;
      padding-right: 14px;
      flex-shrink: 0;
      font-family: ${MONO};
      font-size: 10.5px;
    }
    .add { color: var(--success); }
    .del { color: var(--destructive); }
  `

  static properties = {
    sha:       { type: String, reflect: true },
    subject:   { type: String, reflect: true },
    author:    { type: String, reflect: true },
    timestamp: { type: String, reflect: true },
    additions: { type: Number, reflect: true },
    deletions: { type: Number, reflect: true },
    active:    { type: Boolean, reflect: true },
    human:     { type: Boolean, reflect: true },
  }

  sha       = ''
  subject   = ''
  author    = ''
  timestamp = ''
  additions = 0
  deletions = 0
  active    = false
  human     = false

  connectedCallback() {
    super.connectedCallback()
    this.setAttribute('role', 'button')
    this.setAttribute('tabindex', '0')
    this.addEventListener('keydown', this._onKeyDown)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this._onKeyDown)
  }

  private _onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this._select()
    }
  }

  private _select() {
    this.dispatchEvent(
      new CustomEvent('ship-select', {
        detail: { sha: this.sha },
        bubbles: true,
        composed: true,
      }),
    )
  }

  // Parse "scope(pkg): subject" into prefix + rest
  private _splitSubject(): { scope: string; rest: string } {
    const m = this.subject.match(/^([a-z]+(?:\([^)]+\))?!?):\s*(.+)$/)
    if (m) return { scope: m[1] + ':', rest: m[2] }
    return { scope: '', rest: this.subject }
  }

  private _initial(): string {
    return (this.author || '?').charAt(0).toUpperCase()
  }

  render() {
    const { scope, rest } = this._splitSubject()
    const dotR = this.active ? 6 : 4.5

    return html`
      <div class="lane" @click=${this._select}>
        <svg width="44" height="46" aria-hidden="true">
          <line x1="22" y1="0"  x2="22" y2="46"
            stroke="var(--primary)" stroke-width="1.5" opacity="0.45"/>
          <circle cx="22" cy="23" r=${dotR}
            fill=${this.active ? 'var(--primary)' : 'var(--background)'}
            stroke="var(--primary)" stroke-width="2"/>
        </svg>
      </div>

      <div class="avatar">${this._initial()}</div>

      <div class="msg" @click=${this._select}>
        <div class="title">
          ${scope ? html`<span class="scope">${scope}</span> ` : null}${rest}
        </div>
        <div class="meta">
          <span>${this.sha}</span>
          <span>${this.timestamp}</span>
        </div>
      </div>

      <div class="stats">
        ${this.additions > 0 ? html`<span class="add">+${this.additions}</span>` : null}
        ${this.deletions > 0 ? html`<span class="del">−${this.deletions}</span>` : null}
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-history-row')) {
  customElements.define('ship-history-row', ShipHistoryRow)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-history-row': ShipHistoryRow
  }
}
