/**
 * <ship-pipeline-step>
 *
 * A single step node in a pipeline DAG. Matches the pipelines view
 * `.step` element exactly (10px 14px padding, 8px radius, 130px min-width,
 * status-tinted backgrounds, absolute-positioned status icon top-right).
 *
 * Attributes:
 *   - name — step id (mono)
 *   - step-type — agent | gate | merge | run | human (displayed as kicker)
 *   - state — pending | running | completed | failed (drives color)
 *   - agent — optional agent label appended to the type line
 *
 * Composes:
 *   - <ship-spinner> for the running state's icon animation.
 *
 * Slots: (none — data is attribute-driven for clean SSE patching)
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import '../ship-spinner'
import { MONO } from '../util'

type StepState = 'pending' | 'running' | 'completed' | 'failed'
type StepType = 'agent' | 'gate' | 'merge' | 'run' | 'human'

@customElement('ship-pipeline-step')
export class ShipPipelineStep extends LitElement {
  static styles = css`
    :host {
      display: flex; flex-direction: column; gap: 4px;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 10px 14px;
      min-width: 130px;
      background: var(--background);
      position: relative;
      font-family: var(--font-body);
    }
    .icon {
      position: absolute;
      top: 10px; right: 12px;
      font-size: 10px;
      color: var(--muted-foreground);
      line-height: 1;
    }
    :host([state='completed']) {
      border-color: oklch(0.62 0.17 150 / 50%);
      background: oklch(0.62 0.17 150 / 5%);
    }
    :host([state='completed']) .icon { color: var(--success); }
    :host([state='running']) {
      border-color: oklch(0.65 0.15 240 / 60%);
      background: oklch(0.65 0.15 240 / 7%);
      box-shadow: 0 0 0 3px oklch(0.65 0.15 240 / 12%);
    }
    :host([state='running']) .icon { color: var(--info); }
    :host([state='failed']) {
      border-color: oklch(0.58 0.22 27 / 55%);
      background: oklch(0.58 0.22 27 / 6%);
    }
    :host([state='failed']) .icon { color: var(--destructive); }

    .name {
      font-family: ${MONO};
      font-size: 11.5px; font-weight: 600;
      padding-right: 16px;
      color: var(--foreground);
    }
    .type {
      font-size: 9px;
      text-transform: uppercase;
      letter-spacing: 0.07em;
      color: var(--muted-foreground);
    }
    .status {
      font-size: 10px;
      color: var(--muted-foreground);
    }
    ship-spinner {
      /* same size + placement as the glyph icon it replaces */
      position: absolute; top: 10px; right: 12px;
    }
  `

  @property({ reflect: true }) name = ''
  @property({ attribute: 'step-type', reflect: true }) stepType: StepType = 'agent'
  @property({ reflect: true }) state: StepState = 'pending'
  @property({ reflect: true }) agent = ''

  private glyphFor(state: StepState): string {
    switch (state) {
      case 'completed': return '\u2713' // ✓
      case 'failed': return '\u2717' // ✗
      case 'running': return '' // spinner
      default: return '\u25CB' // ○
    }
  }

  render() {
    const isRunning = this.state === 'running'
    return html`
      ${isRunning
        ? html`<ship-spinner size="sm" tone="info" label="Running"></ship-spinner>`
        : html`<span class="icon" aria-hidden="true">${this.glyphFor(this.state)}</span>`}
      <span class="name">${this.name}</span>
      <span class="type">
        ${this.stepType}${this.agent ? html` &middot; ${this.agent}` : ''}
      </span>
      <span class="status">${this.state}</span>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-pipeline-step': ShipPipelineStep
  }
}
