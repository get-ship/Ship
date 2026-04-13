/**
 * <ship-pipeline-run>
 *
 * A pipeline run row. Matches pipelines view `.run-card` with the
 * state-driven left rail (3px solid tinted border). Composes <ship-card>
 * with status-rail and an inline header (slug + agent code on the left,
 * status badge on the right).
 *
 * Attributes:
 *   - state — dispatched | gatepending | running | completed | failed |
 *     merged | pending — maps to card rail tone + badge tone at the caller
 *   - slug — run id
 *   - agent — agent name
 *
 * Slots:
 *   - (default) the DAG / step list
 *   - actions — right-side header chips (beyond the badge)
 *   - footer
 */

import { LitElement, css, html } from 'lit'
import '../ship-badge'
import '../ship-card'
import { MONO } from '../util'
import type { Tone } from '../util'

type RunState =
  | 'dispatched'
  | 'gatepending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'merged'
  | 'pending'

const STATE_TO_TONE: Record<RunState, Tone> = {
  dispatched: 'info',
  running: 'info',
  gatepending: 'warning',
  completed: 'success',
  failed: 'destructive',
  merged: 'accent',
  pending: 'muted',
}

export class ShipPipelineRun extends LitElement {
  static styles = css`
    :host { display: block; }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 12px;
      gap: 12px;
      flex-wrap: wrap;
    }
    .left { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
    .slug {
      font-family: ${MONO};
      font-size: 12.5px;
      font-weight: 600;
      color: var(--foreground);
    }
    .agent { font-size: 11px; color: var(--muted-foreground); }
    .agent code {
      background: var(--muted);
      padding: 1px 6px;
      border-radius: 4px;
      font-size: 10.5px;
      font-family: ${MONO};
    }
  `

  static properties = {
    state: { type: String, reflect: true },
    slug: { type: String, reflect: true },
    agent: { type: String, reflect: true },
  }

  state: RunState = 'pending'
  slug = ''
  agent = ''
  render() {
    const tone = STATE_TO_TONE[this.state] ?? 'muted'
    return html`
      <ship-card status-rail tone=${tone}>
        <div slot="header">
          <header>
            <div class="left">
              <span class="slug">${this.slug}</span>
              ${this.agent
                ? html`<span class="agent"><code>${this.agent}</code></span>`
                : null}
              <slot name="actions"></slot>
            </div>
            <ship-badge tone=${tone}>${this.state}</ship-badge>
          </header>
        </div>
        <slot></slot>
        <div slot="footer"><slot name="footer"></slot></div>
      </ship-card>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-pipeline-run')) {
  customElements.define('ship-pipeline-run', ShipPipelineRun)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-pipeline-run': ShipPipelineRun
  }
}
