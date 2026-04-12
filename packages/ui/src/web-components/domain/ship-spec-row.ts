/**
 * <ship-spec-row>
 *
 * Rail item for a single spec. Matches specs view `.spec-row` button
 * with title + meta row (status badge + pipeline tag).
 *
 * Composes <ship-list-item> + <ship-badge>.
 *
 * Attributes:
 *   - title (mono/body)
 *   - status — draft | ready | launched | completed | failed
 *   - pipeline — optional pipeline slug
 *   - selected: boolean
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import '../ship-badge'
import '../ship-list-item'
import { MONO } from '../util'
import type { Tone } from '../util'

type SpecStatus = 'draft' | 'ready' | 'launched' | 'completed' | 'failed'

const STATUS_TO_TONE: Record<SpecStatus, Tone> = {
  draft: 'muted',
  ready: 'success',
  launched: 'warning',
  completed: 'success',
  failed: 'destructive',
}

@customElement('ship-spec-row')
export class ShipSpecRow extends LitElement {
  static styles = css`
    :host { display: block; }
    .pipeline {
      font-family: ${MONO};
      color: var(--accent);
      font-size: 10px;
    }
  `

  @property({ reflect: true, attribute: 'item-title' }) itemTitle = ''
  @property({ reflect: true }) status: SpecStatus = 'draft'
  @property({ reflect: true }) pipeline = ''
  @property({ type: Boolean, reflect: true }) selected = false

  render() {
    const tone = STATUS_TO_TONE[this.status] ?? 'muted'
    return html`
      <ship-list-item ?selected=${this.selected}>
        ${this.itemTitle || '(untitled)'}
        <span slot="meta">
          <ship-badge tone=${tone} size="sm">${this.status}</ship-badge>
          ${this.pipeline ? html`<span class="pipeline">${this.pipeline}</span>` : null}
        </span>
      </ship-list-item>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-spec-row': ShipSpecRow
  }
}
