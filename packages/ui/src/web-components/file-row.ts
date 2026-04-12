import { MONO_STACK, defineOnce } from './util'

type FileStatus = 'M' | 'A' | 'D' | 'R' | '?'

const STATUS_META: Record<FileStatus, { label: string; color: string; title: string }> = {
  M: { label: 'M', color: 'var(--warning)', title: 'Modified' },
  A: { label: 'A', color: 'var(--success)', title: 'Added' },
  D: { label: 'D', color: 'var(--destructive)', title: 'Deleted' },
  R: { label: 'R', color: 'var(--info)', title: 'Renamed' },
  '?': { label: '?', color: 'var(--muted-foreground)', title: 'Untracked' },
}

export class ShipFileRow extends HTMLElement {
  static get observedAttributes() {
    return ['path', 'status', 'additions', 'deletions', 'truncate']
  }

  private root: ShadowRoot
  constructor() { super(); this.root = this.attachShadow({ mode: 'open' }) }
  connectedCallback() { this.render() }
  attributeChangedCallback() { if (this.root) this.render() }

  private render() {
    const path = this.getAttribute('path') || ''
    const status = ((this.getAttribute('status') || '?') as FileStatus)
    const additions = this.getAttribute('additions')
    const deletions = this.getAttribute('deletions')
    const truncate = this.hasAttribute('truncate')

    const meta = STATUS_META[status] || STATUS_META['?']

    const stats: string[] = []
    if (additions && Number(additions) > 0) {
      stats.push(`<span class="add">+${additions}</span>`)
    }
    if (deletions && Number(deletions) > 0) {
      stats.push(`<span class="del">-${deletions}</span>`)
    }

    this.root.innerHTML = `
      <style>
        :host { display: block; }
        .row {
          display: flex; align-items: center; gap: 8px;
          padding: 6px 10px;
          font-family: var(--font-body);
          font-size: 13px;
          color: var(--foreground);
          border-radius: calc(var(--radius) - 4px);
        }
        .row:hover { background: var(--muted); }
        .status {
          flex-shrink: 0;
          width: 18px; height: 18px;
          display: inline-flex; align-items: center; justify-content: center;
          font-family: ${MONO_STACK};
          font-size: 11px; font-weight: 700;
          color: ${meta.color};
          background: color-mix(in oklch, ${meta.color} 15%, transparent);
          border-radius: 4px;
        }
        .path {
          font-family: ${MONO_STACK};
          font-size: 12.5px;
          flex: 1;
          min-width: 0;
          ${truncate ? 'overflow: hidden; text-overflow: ellipsis; white-space: nowrap; direction: rtl; text-align: left;' : 'word-break: break-all;'}
        }
        .stats {
          flex-shrink: 0;
          display: inline-flex; gap: 6px;
          font-family: ${MONO_STACK};
          font-size: 11.5px;
        }
        .add { color: var(--success); }
        .del { color: var(--destructive); }
      </style>
      <div class="row">
        <span class="status" title="${meta.title}">${meta.label}</span>
        <span class="path" title="${path}">${path}</span>
        ${stats.length ? `<span class="stats">${stats.join('')}</span>` : ''}
      </div>
    `
  }
}

export function registerFileRow() {
  defineOnce('ship-file-row', ShipFileRow)
}
