/**
 * <ship-pipeline-definition>
 *
 * Tile for a pipeline YAML definition. Matches pipelines view `.def-card`
 * exactly: interactive, accent-bar hover ::before gradient, three-row
 * layout (name + step-count header, description body).
 *
 * Attributes:
 *   - name (mono kicker, display font)
 *   - steps: number — step count in the definition
 *   - description — short descriptor; falls back to a muted "No description"
 *
 * Slots:
 *   - actions — trailing header actions
 */

import { LitElement, css, html } from 'lit'
import '../core/ship-card'

export class ShipPipelineDefinition extends LitElement {
  static styles = css`
    :host { display: block; }
    header {
      display: flex; justify-content: space-between;
      align-items: baseline; gap: 12px;
    }
    .name {
      font-family: var(--font-display);
      font-size: 15px;
      font-weight: 600;
      letter-spacing: -0.01em;
      color: var(--foreground);
    }
    .steps {
      font-size: 10.5px;
      color: var(--muted-foreground);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      font-variant-numeric: tabular-nums;
      white-space: nowrap;
    }
    .desc {
      font-size: 12.5px;
      color: var(--muted-foreground);
      line-height: 1.5;
      min-height: 1.5em;
      margin-top: 6px;
    }
    .no-desc { opacity: 0.5; }
    ship-card { display: block; padding: 18px 20px; }
  `

  static properties = {
    name: { type: String, reflect: true },
    steps: { type: Number, reflect: true },
    description: { type: String, reflect: true },
  }

  name = ''
  steps = 0
  description = ''
  updated() {
    this.setAttribute('role', 'article')
    const desc = this.description ? `: ${this.description}` : ''
    this.setAttribute('aria-label', `Pipeline ${this.name}, ${this.steps} step${this.steps === 1 ? '' : 's'}${desc}`)
  }
  render() {
    const s = this.steps
    const suffix = s === 1 ? '' : 's'
    return html`
      <ship-card interactive accent-bar>
        <div slot="header">
          <header>
            <span class="name">${this.name}</span>
            <span class="steps">${s} step${suffix}</span>
            <slot name="actions"></slot>
          </header>
        </div>
        ${this.description
          ? html`<p class="desc">${this.description}</p>`
          : html`<p class="desc no-desc">No description</p>`}
      </ship-card>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-pipeline-definition')) {
  customElements.define('ship-pipeline-definition', ShipPipelineDefinition)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-pipeline-definition': ShipPipelineDefinition
  }
}
