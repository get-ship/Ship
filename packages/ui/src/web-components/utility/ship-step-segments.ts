/**
 * <ship-step-segments>
 *
 * Compact row of colored rectangles showing per-step outcome for a
 * pipeline run. Used in history rows to summarize N steps at a glance.
 *
 * Attributes:
 *   - values: comma-separated segment states. Each token is one of
 *       `done | failed | pending | running`. Unknown tokens render as
 *       `pending`. Empty / missing attribute renders nothing.
 *   - size: sm | md (default sm). Controls bar height/width.
 *
 * Colors pull from the shared token palette via `--success`,
 * `--destructive`, `--warning`, `--muted-foreground`. No Shadow DOM
 * leakage — all styling is host-scoped.
 */

import { LitElement, css, html } from 'lit'

export type SegmentState = 'done' | 'failed' | 'pending' | 'running'

const KNOWN: ReadonlySet<SegmentState> = new Set(['done', 'failed', 'pending', 'running'])

export class ShipStepSegments extends LitElement {
  static styles = css`
    :host { display: inline-flex; vertical-align: middle; }
    .row { display: inline-flex; gap: 3px; align-items: center; }
    .seg {
      display: inline-block;
      width: 18px; height: 6px;
      border-radius: 2px;
      background: color-mix(in oklch, var(--muted-foreground) 28%, transparent);
    }
    :host([size='md']) .seg { width: 24px; height: 8px; border-radius: 3px; }
    .seg.done { background: var(--success); }
    .seg.failed { background: var(--destructive); }
    .seg.running {
      background: var(--warning);
      animation: pulse var(--duration-slow, 1200ms) ease-in-out infinite;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.55; }
    }
  `

  static properties = {
    values: { type: String, reflect: true },
    size: { type: String, reflect: true },
  }

  values = ''
  size: 'sm' | 'md' = 'sm'

  private parsed(): SegmentState[] {
    return this.values
      .split(',')
      .map((s) => s.trim().toLowerCase())
      .filter((s) => s.length > 0)
      .map((s) => (KNOWN.has(s as SegmentState) ? (s as SegmentState) : 'pending'))
  }

  render() {
    const segs = this.parsed()
    if (segs.length === 0) return html``
    return html`
      <span class="row" part="row" role="img"
            aria-label="${segs.length} step outcome${segs.length === 1 ? '' : 's'}">
        ${segs.map(
          (s) => html`<span class="seg ${s}" part="seg" data-state=${s}></span>`,
        )}
      </span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-step-segments')) {
  customElements.define('ship-step-segments', ShipStepSegments)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-step-segments': ShipStepSegments
  }
}
