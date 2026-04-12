/**
 * <ship-toolbar>
 *
 * Horizontal flex row of actions. Matches specs `.footer` (action bar at
 * bottom, spacer + buttons right-aligned). A child with `slot="spacer"`
 * expands and pushes following children to the end; simpler callers can
 * just use `justify="end"`.
 *
 * Attributes:
 *   - justify: start | center | end | between (default: start)
 *   - align: center | start | end | baseline (default: center)
 *   - gap: sm | md | lg (default: md — 8px)
 *   - variant: default | bordered (bordered adds a top 1px border + padding)
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

@customElement('ship-toolbar')
export class ShipToolbar extends LitElement {
  static styles = css`
    :host { display: flex; align-items: center; gap: 8px; }
    :host([gap='sm']) { gap: 4px; }
    :host([gap='lg']) { gap: 12px; }
    :host([justify='center']) { justify-content: center; }
    :host([justify='end']) { justify-content: flex-end; }
    :host([justify='between']) { justify-content: space-between; }
    :host([align='start']) { align-items: flex-start; }
    :host([align='end']) { align-items: flex-end; }
    :host([align='baseline']) { align-items: baseline; }
    :host([variant='bordered']) {
      padding: 12px 20px;
      border-top: 1px solid var(--border);
    }
    ::slotted([slot='spacer']) { flex: 1; }
  `

  @property({ reflect: true }) justify: 'start' | 'center' | 'end' | 'between' = 'start'
  @property({ reflect: true }) align: 'center' | 'start' | 'end' | 'baseline' = 'center'
  @property({ reflect: true }) gap: 'sm' | 'md' | 'lg' = 'md'
  @property({ reflect: true }) variant: 'default' | 'bordered' = 'default'

  render() {
    return html`<slot name="spacer"></slot><slot></slot>`
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-toolbar': ShipToolbar
  }
}
