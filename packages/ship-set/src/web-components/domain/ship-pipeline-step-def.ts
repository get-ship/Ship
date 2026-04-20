/**
 * <ship-pipeline-step-def>
 *
 * Renders the DECLARED shape of a pipeline step — what the YAML
 * says the step is. Distinct from <ship-pipeline-step>, which
 * renders an individual run's live step state.
 *
 * Use on pipeline-definition views (focus page, editor, registry
 * detail), not on live-run views.
 *
 * Attributes:
 *   - name: step id (required).
 *   - step-type: agent | gate | run | merge | human. Drives the kind chip.
 *   - agent, provider, model: optional chips displayed below the header.
 *   - after: comma-separated list of parent step ids.
 *   - command: shell command body (for `step-type=run`). Rendered as a
 *     mono <pre>. Leading/trailing whitespace trimmed.
 *   - index: 1-based step number (optional, rendered as a gutter).
 *
 * Slots:
 *   - gate: gate rules (require_changed / forbid_changed) when applicable.
 *     Caller renders these as <ship-chip> or similar.
 *   - trailing: right-side content on the header row (e.g. an edit button).
 *
 * Parts:
 *   - row — outer grid container
 *   - header — name + chips row
 *   - command — the <pre> command block
 */

import { LitElement, css, html } from 'lit'
import '../core/ship-badge'
import '../core/ship-chip'
import { defineOnce, MONO } from '../core/util'

const TYPE_TONE: Record<string, 'muted' | 'info' | 'warning' | 'accent'> = {
  run: 'muted',
  agent: 'info',
  gate: 'warning',
  merge: 'accent',
  human: 'warning',
}

export class ShipPipelineStepDef extends LitElement {
  static styles = css`
    :host {
      display: block;
      padding: 16px 0;
      border-top: 1px solid color-mix(in oklch, var(--border) 40%, transparent);
    }
    :host(:first-of-type), :host([first]) { border-top: none; padding-top: 0; }
    :host(:last-of-type) { padding-bottom: 0; }

    [part='row'] {
      display: grid;
      grid-template-columns: 2ch 1fr;
      gap: 16px;
    }
    .index {
      font-family: ${MONO};
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      padding-top: 3px;
    }
    .body { display: flex; flex-direction: column; gap: 8px; min-width: 0; }
    [part='header'] {
      display: flex;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
    }
    .name {
      font-family: ${MONO};
      font-size: var(--text-sm);
      font-weight: var(--font-weight-semibold);
      color: var(--foreground);
    }
    .after {
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .after code {
      font-family: ${MONO};
      background: color-mix(in oklch, var(--muted) 40%, transparent);
      padding: 1px 5px;
      border-radius: 4px;
      font-size: var(--text-xs);
    }
    .chips { display: flex; gap: 6px; flex-wrap: wrap; }
    [part='command'] {
      margin: 0;
      padding: 10px 12px;
      border-radius: var(--radius-md);
      border: 1px solid color-mix(in oklch, var(--border) 40%, transparent);
      background: color-mix(in oklch, var(--muted) 35%, transparent);
      font-family: ${MONO};
      font-size: var(--text-xs);
      line-height: 1.55;
      color: var(--foreground);
      white-space: pre-wrap;
      word-break: break-word;
      overflow-x: auto;
    }
    .gate-slot {
      padding: 8px 12px;
      border-left: 2px solid color-mix(in oklch, var(--warning) 45%, transparent);
      display: flex;
      flex-direction: column;
      gap: 4px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .trailing { margin-left: auto; }
  `

  static properties = {
    name: { type: String, reflect: true },
    stepType: { type: String, attribute: 'step-type', reflect: true },
    agent: { type: String, reflect: true },
    provider: { type: String, reflect: true },
    model: { type: String, reflect: true },
    afterIds: { type: String, attribute: 'after', reflect: true },
    command: { type: String },
    index: { type: Number, reflect: true },
  }

  name = ''
  stepType: string = 'agent'
  agent = ''
  provider = ''
  model = ''
  afterIds = ''
  command = ''
  index?: number

  render() {
    const tone = TYPE_TONE[this.stepType] ?? 'muted'
    const afterList = (this.afterIds || '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
    const cmd = (this.command || '').trim()

    return html`
      <div part="row">
        <span class="index">${this.index ?? ''}</span>
        <div class="body">
          <div part="header">
            <span class="name">${this.name}</span>
            <ship-chip size="sm" tone=${tone}>${this.stepType}</ship-chip>
            ${afterList.length > 0
              ? html`<span class="after">after
                  ${afterList.map(
                    (a, i) => html`${i > 0 ? ', ' : ''}<code>${a}</code>`
                  )}
                </span>`
              : null}
            <span class="trailing"><slot name="trailing"></slot></span>
          </div>
          ${this.agent || this.provider || this.model
            ? html`<div class="chips">
                ${this.agent ? html`<ship-chip size="sm">${this.agent}</ship-chip>` : null}
                ${this.provider ? html`<ship-chip size="sm">${this.provider}</ship-chip>` : null}
                ${this.model ? html`<ship-chip size="sm">${this.model}</ship-chip>` : null}
              </div>`
            : null}
          ${cmd ? html`<pre part="command">${cmd}</pre>` : null}
          <slot name="gate"></slot>
        </div>
      </div>
    `
  }
}

defineOnce('ship-pipeline-step-def', ShipPipelineStepDef)
