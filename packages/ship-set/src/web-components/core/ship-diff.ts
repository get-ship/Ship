/**
 * <ship-diff>
 *
 * Unified diff renderer. Two modes:
 *   1. Supply `from` + `to`: a line-level diff is computed (LCS).
 *   2. Supply `value`: already-formatted unified-diff text (+/-/space/@@).
 *
 * Attributes:
 *   - from: original text
 *   - to: revised text
 *   - value: raw unified-diff string (overrides from/to)
 */

import { LitElement, css, html } from 'lit'
import { MONO } from './util'

type Line = { kind: 'add' | 'del' | 'ctx' | 'hunk'; text: string }

function diffLines(a: string, b: string): Line[] {
  const A = a.split('\n')
  const B = b.split('\n')
  const n = A.length
  const m = B.length
  // LCS DP
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0))
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] = A[i] === B[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1])
    }
  }
  const out: Line[] = []
  let i = 0
  let j = 0
  while (i < n && j < m) {
    if (A[i] === B[j]) { out.push({ kind: 'ctx', text: A[i] }); i++; j++ }
    else if (dp[i + 1][j] >= dp[i][j + 1]) { out.push({ kind: 'del', text: A[i] }); i++ }
    else { out.push({ kind: 'add', text: B[j] }); j++ }
  }
  while (i < n) out.push({ kind: 'del', text: A[i++] })
  while (j < m) out.push({ kind: 'add', text: B[j++] })
  return out
}

function parseUnified(value: string): Line[] {
  return value.split('\n').map((line) => {
    if (line.startsWith('@@')) return { kind: 'hunk', text: line }
    if (line.startsWith('+')) return { kind: 'add', text: line.slice(1) }
    if (line.startsWith('-')) return { kind: 'del', text: line.slice(1) }
    return { kind: 'ctx', text: line.startsWith(' ') ? line.slice(1) : line }
  })
}

export class ShipDiff extends LitElement {
  static styles = css`
    :host {
      display: block;
      font-family: ${MONO};
      font-size: var(--text-sm);
      line-height: var(--leading-normal);
      background: var(--muted);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      overflow: hidden;
    }
    .line {
      display: grid;
      grid-template-columns: 16px 1fr;
      padding: 1px 8px;
      white-space: pre-wrap;
      word-break: break-word;
    }
    .line.add { background: var(--tone-success-bg); color: var(--foreground); }
    .line.del { background: var(--tone-destructive-bg); color: var(--foreground); }
    .line.hunk { background: var(--tone-info-bg); color: var(--info); }
    .sign { color: var(--muted-foreground); user-select: none; }
    .line.add .sign { color: var(--success); }
    .line.del .sign { color: var(--destructive); }
  `

  static properties = {
    from: { type: String, reflect: true },
    to: { type: String, reflect: true },
    value: { type: String, reflect: true },
  }

  from = ''
  to = ''
  value = ''

  private _lines(): Line[] {
    if (this.value) return parseUnified(this.value)
    if (this.from || this.to) return diffLines(this.from, this.to)
    return []
  }

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'region')
    if (!this.hasAttribute('aria-label')) this.setAttribute('aria-label', 'diff')
  }

  render() {
    const lines = this._lines()
    return html`${lines.map((l) => {
      const sign = l.kind === 'add' ? '+' : l.kind === 'del' ? '-' : l.kind === 'hunk' ? '@' : ' '
      const kindLabel = l.kind === 'add' ? 'added' : l.kind === 'del' ? 'removed' : l.kind === 'hunk' ? 'hunk' : 'context'
      return html`<div class="line ${l.kind}" part="line" role="listitem" aria-label=${`${kindLabel}: ${l.text}`}><span class="sign" aria-hidden="true">${sign}</span><span>${l.text}</span></div>`
    })}`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-diff')) {
  customElements.define('ship-diff', ShipDiff)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-diff': ShipDiff
  }
}
