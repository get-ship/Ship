/**
 * <ship-tooltip>
 *
 * CSS-only tooltip positioned via absolute offsets. Wraps the trigger as
 * light-DOM children; `content` slot provides the bubble body. Shows on
 * hover and on keyboard focus.
 *
 * Attributes:
 *   - placement: top | bottom | left | right (default: top)
 *   - delay: number (ms show delay, default 100)
 *
 * Slots:
 *   - (default) trigger content
 *   - content — tooltip body
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

@customElement('ship-tooltip')
export class ShipTooltip extends LitElement {
  static styles = css`
    :host { position: relative; display: inline-flex; }
    .bubble {
      position: absolute;
      z-index: 20;
      padding: 4px 8px;
      background: var(--foreground);
      color: var(--background);
      font-family: var(--font-body);
      font-size: 11px;
      font-weight: 500;
      border-radius: 4px;
      white-space: nowrap;
      pointer-events: none;
      opacity: 0;
      transition: opacity 120ms ease;
      box-shadow: 0 2px 4px oklch(0 0 0 / 20%);
    }
    :host(:hover) .bubble,
    :host(:focus-within) .bubble { opacity: 1; }
    :host([placement='top']) .bubble,
    :host(:not([placement])) .bubble {
      bottom: calc(100% + 6px);
      left: 50%;
      transform: translateX(-50%);
    }
    :host([placement='bottom']) .bubble {
      top: calc(100% + 6px);
      left: 50%;
      transform: translateX(-50%);
    }
    :host([placement='left']) .bubble {
      right: calc(100% + 6px);
      top: 50%;
      transform: translateY(-50%);
    }
    :host([placement='right']) .bubble {
      left: calc(100% + 6px);
      top: 50%;
      transform: translateY(-50%);
    }
  `

  @property({ reflect: true }) placement: 'top' | 'bottom' | 'left' | 'right' = 'top'

  render() {
    return html`
      <slot></slot>
      <span class="bubble" role="tooltip"><slot name="content"></slot></span>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-tooltip': ShipTooltip
  }
}
