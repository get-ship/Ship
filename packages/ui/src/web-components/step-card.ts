import { MONO_STACK, defineOnce } from './util'

type Status = 'pending' | 'running' | 'completed' | 'failed'
type StepType = 'agent' | 'gate' | 'merge' | 'run' | 'human'

const STATUS_ICON: Record<Status, { glyph: string; color: string }> = {
  pending:   { glyph: '○', color: 'var(--muted-foreground)' },
  running:   { glyph: '◐', color: 'var(--primary)' },
  completed: { glyph: '●', color: 'var(--success)' },
  failed:    { glyph: '✕', color: 'var(--destructive)' },
}

const TYPE_COLOR: Record<StepType, string> = {
  agent: 'var(--accent)',
  gate: 'var(--warning)',
  merge: 'var(--info)',
  run: 'var(--primary)',
  human: 'var(--muted-foreground)',
}

export class ShipStepCard extends HTMLElement {
  static get observedAttributes() {
    return ['name', 'type', 'status', 'agent', 'commits', 'duration', 'selected']
  }

  private root: ShadowRoot
  constructor() { super(); this.root = this.attachShadow({ mode: 'open' }) }
  connectedCallback() { this.render() }
  attributeChangedCallback() { if (this.root) this.render() }

  private formatDuration(secs: number): string {
    if (secs < 60) return `${secs}s`
    const m = Math.floor(secs / 60)
    const s = secs % 60
    return s ? `${m}m ${s}s` : `${m}m`
  }

  private render() {
    const name = this.getAttribute('name') || ''
    const type = (this.getAttribute('type') as StepType) || 'agent'
    const status = (this.getAttribute('status') as Status) || 'pending'
    const agent = this.getAttribute('agent')
    const commitsAttr = this.getAttribute('commits')
    const durationAttr = this.getAttribute('duration')
    const selected = this.hasAttribute('selected')

    const icon = STATUS_ICON[status] || STATUS_ICON.pending
    const typeColor = TYPE_COLOR[type] || 'var(--muted-foreground)'

    const evidence: string[] = []
    if (agent) evidence.push(`<span class="evi">${agent}</span>`)
    if (commitsAttr) evidence.push(`<span class="evi">${commitsAttr} commit${commitsAttr === '1' ? '' : 's'}</span>`)
    if (durationAttr) {
      const n = Number(durationAttr)
      if (Number.isFinite(n)) evidence.push(`<span class="evi">${this.formatDuration(n)}</span>`)
    }

    this.root.innerHTML = `
      <style>
        :host { display: block; }
        .card {
          display: flex; align-items: center; gap: 10px;
          padding: 10px 12px;
          background: var(--card);
          color: var(--card-foreground);
          border: 1px solid ${selected ? 'var(--primary)' : 'var(--border)'};
          border-radius: var(--radius);
          font-family: var(--font-body);
          font-size: 13px;
          box-shadow: ${selected ? '0 0 0 1px var(--primary)' : 'none'};
          transition: border-color 120ms ease, box-shadow 120ms ease;
        }
        .icon {
          font-size: 16px; line-height: 1; color: ${icon.color};
          ${status === 'running' ? 'animation: spin 1.2s linear infinite;' : ''}
        }
        .main { display: flex; flex-direction: column; gap: 2px; min-width: 0; flex: 1; }
        .row { display: flex; align-items: center; gap: 8px; }
        .name {
          font-family: ${MONO_STACK};
          font-size: 13px; font-weight: 500;
          overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
        }
        .type {
          font-size: 10px; font-weight: 600; letter-spacing: 0.04em;
          text-transform: uppercase;
          color: ${typeColor};
          padding: 1px 6px;
          border: 1px solid ${typeColor};
          border-radius: 999px;
          background: color-mix(in oklch, ${typeColor} 10%, transparent);
        }
        .evidence { display: flex; gap: 8px; color: var(--muted-foreground); font-size: 11px; }
        .evi:not(:last-child)::after { content: '·'; margin-left: 8px; }
        @keyframes spin { to { transform: rotate(360deg); } }
      </style>
      <div class="card">
        <span class="icon">${icon.glyph}</span>
        <div class="main">
          <div class="row">
            <span class="name">${name}</span>
            <span class="type">${type}</span>
          </div>
          ${evidence.length ? `<div class="evidence">${evidence.join('')}</div>` : ''}
        </div>
      </div>
    `
  }
}

export function registerStepCard() {
  defineOnce('ship-step-card', ShipStepCard)
}
