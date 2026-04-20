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
import '../core/ship-badge'
import '../core/ship-list-item'
import { MONO } from '../core/util'
import type { Tone } from '../core/util'

type SpecStatus = 'draft' | 'ready' | 'launched' | 'completed' | 'failed'

const STATUS_TO_TONE: Record<SpecStatus, Tone> = {
  draft: 'muted',
  ready: 'success',
  launched: 'warning',
  completed: 'success',
  failed: 'destructive',
}

export class ShipSpecRow extends LitElement {
  static styles = css`
    :host { display: block; }
    .pipeline {
      font-family: ${MONO};
      color: var(--accent);
      font-size: 10px;
    }
  `

  static properties = {
    itemTitle: { type: String, reflect: true, attribute: 'item-title' },
    status: { type: String, reflect: true },
    pipeline: { type: String, reflect: true },
    selected: { type: Boolean, reflect: true },
  }

  itemTitle = ''
  status: SpecStatus = 'draft'
  pipeline = ''
  selected = false
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


if (typeof customElements !== 'undefined' && !customElements.get('ship-spec-row')) {
  customElements.define('ship-spec-row', ShipSpecRow)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-spec-row': ShipSpecRow
  }
}
