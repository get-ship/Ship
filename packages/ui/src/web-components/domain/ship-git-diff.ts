/**
 * <ship-git-diff>, <ship-git-diff-file>, <ship-git-diff-hunk>, <ship-git-diff-line>
 *
 * Port of the React DiffViewer. Daemon pre-parses the unified-diff into a tree
 * of these elements; client handles interactivity (per-file + expand/collapse
 * all) in Lit reactive state.
 */

import { LitElement, css, html } from 'lit'
import '../utility/ship-icon'
import { MONO } from '../util'

// -------------------- container: <ship-git-diff> -------------------- //

export class ShipGitDiff extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      min-height: 0;
      font-family: var(--font-body);
      background: var(--background);
      color: var(--foreground);
      border: 1px solid color-mix(in oklch, var(--border) 60%, transparent);
      border-radius: var(--radius-lg);
      overflow: hidden;
    }
    .toolbar {
      display: flex; align-items: center; gap: 12px;
      padding: 6px 16px;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 60%, transparent);
      background: color-mix(in oklch, var(--card) 50%, transparent);
      flex-shrink: 0;
    }
    .count { font-size: 12px; color: var(--muted-foreground); }
    .adds { font-size: 11px; color: var(--success); }
    .dels { font-size: 11px; color: var(--destructive); }
    .spacer { flex: 1; }
    .toggle {
      display: inline-flex; align-items: center; gap: 4px;
      padding: 2px 8px;
      border-radius: var(--radius-md);
      background: transparent; border: 0;
      font-family: inherit; font-size: 11px;
      color: var(--muted-foreground);
      cursor: pointer;
      transition: background var(--duration-fast) ease, color var(--duration-fast) ease;
    }
    .toggle:hover { background: var(--muted); color: var(--foreground); }
    .body { flex: 1; overflow: auto; }
    :host([empty]) .toolbar { display: none; }
    .empty {
      display: flex; align-items: center; justify-content: center;
      padding: 48px 16px;
      color: var(--muted-foreground);
      font-size: 13px;
    }
  `

  static properties = {
    files: { type: Number, reflect: true },
    additions: { type: Number, reflect: true },
    deletions: { type: Number, reflect: true },
    empty: { type: Boolean, reflect: true },
    allCollapsed: { type: Boolean, state: true },
  }

  files = 0
  additions = 0
  deletions = 0
  empty = false
  allCollapsed = false

  connectedCallback() { super.connectedCallback(); if (!this.hasAttribute('role')) this.setAttribute('role', 'region') }
  updated() { this.setAttribute('aria-label', `Diff: ${this.files} file${this.files === 1 ? '' : 's'} changed, ${this.additions} additions, ${this.deletions} deletions`) }

  private toggleAll() {
    const next = !this.allCollapsed
    this.allCollapsed = next
    const kids = this.querySelectorAll('ship-git-diff-file') as NodeListOf<ShipGitDiffFile>
    kids.forEach((f) => { f.collapsed = next })
  }

  render() {
    if (this.empty || this.files === 0) {
      return html`<div class="empty">No changes to display</div>`
    }
    return html`
      <div class="toolbar">
        <span class="count">${this.files} file${this.files !== 1 ? 's' : ''} changed</span>
        ${this.additions > 0 ? html`<span class="adds">+${this.additions}</span>` : null}
        ${this.deletions > 0 ? html`<span class="dels">-${this.deletions}</span>` : null}
        <span class="spacer"></span>
        <button class="toggle" type="button" @click=${this.toggleAll}
          title=${this.allCollapsed ? 'Expand all files' : 'Collapse all files'}>
          <ship-icon size="sm" name=${this.allCollapsed ? 'chevrons-up-down' : 'chevrons-down-up'}></ship-icon>
          ${this.allCollapsed ? 'Expand all' : 'Collapse all'}
        </button>
      </div>
      <div class="body"><slot></slot></div>
    `
  }
}

// -------------------- file: <ship-git-diff-file> -------------------- //

export class ShipGitDiffFile extends LitElement {
  static styles = css`
    :host {
      display: block;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 35%, transparent);
    }
    .header {
      display: flex; align-items: center; gap: 8px;
      width: 100%;
      padding: 6px 12px;
      background: transparent;
      border: 0;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 40%, transparent);
      text-align: left;
      cursor: pointer;
      color: var(--foreground);
      transition: background var(--duration-fast) ease;
      font-family: inherit;
    }
    .header:hover { background: color-mix(in oklch, var(--muted) 40%, transparent); }
    .chev { color: var(--muted-foreground); flex-shrink: 0; }
    .fileicon { color: var(--muted-foreground); flex-shrink: 0; }
    .path {
      flex: 1;
      font-family: ${MONO};
      font-size: 12px;
      color: var(--foreground);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    .stats { display: inline-flex; align-items: center; gap: 8px; flex-shrink: 0; }
    .adds, .dels {
      display: inline-flex; align-items: center; gap: 2px;
      font-size: 10px; font-family: ${MONO};
    }
    .adds { color: var(--success); }
    .dels { color: var(--destructive); }
    .body { display: block; }
    :host([collapsed]) .body { display: none; }
  `

  static properties = {
    path: { type: String, reflect: true },
    additions: { type: Number, reflect: true },
    deletions: { type: Number, reflect: true },
    collapsed: { type: Boolean, reflect: true },
  }

  path = ''
  additions = 0
  deletions = 0
  collapsed = false

  connectedCallback() { super.connectedCallback(); if (!this.hasAttribute('role')) this.setAttribute('role', 'group') }
  updated() { this.setAttribute('aria-label', `${this.path}: +${this.additions} -${this.deletions}`) }
  private toggle() { this.collapsed = !this.collapsed }

  render() {
    return html`
      <button class="header" type="button" @click=${this.toggle}
        aria-expanded=${this.collapsed ? 'false' : 'true'}>
        <ship-icon class="chev" size="sm" name=${this.collapsed ? 'chevron-right' : 'chevron-down'}></ship-icon>
        <ship-icon class="fileicon" size="sm" name="file-code"></ship-icon>
        <span class="path">${this.path}</span>
        <span class="stats">
          ${this.additions > 0 ? html`<span class="adds"><ship-icon size="sm" name="plus"></ship-icon>${this.additions}</span>` : null}
          ${this.deletions > 0 ? html`<span class="dels"><ship-icon size="sm" name="minus"></ship-icon>${this.deletions}</span>` : null}
        </span>
      </button>
      <div class="body"><slot></slot></div>
    `
  }
}

// -------------------- hunk: <ship-git-diff-hunk> -------------------- //

export class ShipGitDiffHunk extends LitElement {
  static styles = css`
    :host {
      display: block;
      padding: 2px 12px;
      background: color-mix(in oklch, var(--info) 10%, transparent);
      border-top: 1px solid color-mix(in oklch, var(--info) 20%, transparent);
      border-bottom: 1px solid color-mix(in oklch, var(--info) 20%, transparent);
      font-family: ${MONO};
      font-size: 11px;
      color: var(--info);
      user-select: none;
    }
  `
  static properties = { header: { type: String, reflect: true } }
  header = ''
  render() { return html`${this.header}` }
}

// -------------------- line: <ship-git-diff-line> -------------------- //

type LineType = 'add' | 'del' | 'context'

export class ShipGitDiffLine extends LitElement {
  static styles = css`
    :host {
      display: flex;
      font-family: ${MONO};
      font-size: 11px;
      line-height: 1.45;
    }
    :host([type='add']) { background: color-mix(in oklch, var(--success) 12%, transparent); }
    :host([type='del']) { background: color-mix(in oklch, var(--destructive) 12%, transparent); }
    .num {
      width: 42px;
      flex-shrink: 0;
      padding-right: 8px;
      text-align: right;
      color: color-mix(in oklch, var(--muted-foreground) 60%, transparent);
      user-select: none;
      border-right: 1px solid color-mix(in oklch, var(--border) 20%, transparent);
    }
    .sig {
      width: 16px;
      flex-shrink: 0;
      text-align: center;
      user-select: none;
      color: var(--muted-foreground);
    }
    .content {
      flex: 1;
      padding-right: 12px;
      white-space: pre-wrap;
      word-break: break-all;
      color: var(--muted-foreground);
    }
    :host([type='add']) .sig,
    :host([type='add']) .content { color: var(--success); }
    :host([type='del']) .sig,
    :host([type='del']) .content { color: var(--destructive); }
  `

  static properties = {
    type: { type: String, reflect: true },
    old: { type: String, reflect: true },
    new: { type: String, reflect: true },
  }

  type: LineType = 'context'
  old = ''
  new = ''

  updated() {
    const verb = this.type === 'add' ? 'added' : this.type === 'del' ? 'removed' : 'context'
    this.setAttribute('aria-label', `line ${this.new || this.old} ${verb}`)
  }

  private sig(): string {
    return this.type === 'add' ? '+' : this.type === 'del' ? '-' : ' '
  }

  render() {
    return html`
      <span class="num">${this.old}</span>
      <span class="num">${this.new}</span>
      <span class="sig">${this.sig()}</span>
      <span class="content"><slot></slot></span>
    `
  }
}

// ---------- registration ---------- //

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-git-diff')) customElements.define('ship-git-diff', ShipGitDiff)
  if (!customElements.get('ship-git-diff-file')) customElements.define('ship-git-diff-file', ShipGitDiffFile)
  if (!customElements.get('ship-git-diff-hunk')) customElements.define('ship-git-diff-hunk', ShipGitDiffHunk)
  if (!customElements.get('ship-git-diff-line')) customElements.define('ship-git-diff-line', ShipGitDiffLine)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-git-diff': ShipGitDiff
    'ship-git-diff-file': ShipGitDiffFile
    'ship-git-diff-hunk': ShipGitDiffHunk
    'ship-git-diff-line': ShipGitDiffLine
  }
}
