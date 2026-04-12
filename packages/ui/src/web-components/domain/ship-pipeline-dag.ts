/**
 * <ship-pipeline-dag>
 *
 * Horizontal step-flow container. Exactly matches the pipelines view
 * `.dag` element: flex row with overflow-x auto and a small vertical
 * padding. Expects <ship-pipeline-step> children interleaved with plain
 * arrow separators (emitted by the server between steps).
 *
 * Slots:
 *   - (default) steps + arrows
 */

import { LitElement, css, html } from 'lit'
import { customElement } from 'lit/decorators.js'

@customElement('ship-pipeline-dag')
export class ShipPipelineDag extends LitElement {
  static styles = css`
    :host {
      display: flex;
      align-items: stretch;
      gap: 0;
      overflow-x: auto;
      padding: 4px 0 2px;
    }
    ::slotted(.arrow) {
      display: flex; align-items: center;
      padding: 0 6px;
      color: var(--border);
      font-size: 14px;
      flex-shrink: 0;
    }
  `

  render() { return html`<slot></slot>` }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-pipeline-dag': ShipPipelineDag
  }
}
