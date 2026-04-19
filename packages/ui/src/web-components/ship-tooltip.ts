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
export class ShipTooltip extends LitElement {
  static styles = css`
    :host { position: relative; display: inline-flex; }
    .bubble {
      position: absolute;
      z-index: var(--z-popover);
      padding: 4px 8px;
      background: var(--foreground);
      color: var(--background);
      font-family: var(--font-body);
      font-size: 11px;
      font-weight: var(--font-weight-medium);
      border-radius: var(--radius-sm);
      white-space: nowrap;
      pointer-events: none;
      opacity: 0;
      transition: opacity var(--duration-fast) var(--ease-out) var(--tooltip-hide-delay);
      box-shadow: var(--shadow-sm);
    }
    :host(:hover) .bubble,
    :host(:focus-within) .bubble {
      opacity: 1;
      transition-delay: var(--tooltip-show-delay);
    }
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

  static properties = {
    placement: { type: String, reflect: true },
  }

  placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
  private static _idSeq = 0
  private bubbleId = `ship-tooltip-${++ShipTooltip._idSeq}`

  private onKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') (this as HTMLElement).blur()
  }

  private wireTrigger = () => {
    const slot = this.renderRoot.querySelector('slot:not([name])') as HTMLSlotElement | null
    const first = slot?.assignedElements({ flatten: true })[0] as HTMLElement | undefined
    if (!first) return
    const existing = first.getAttribute('aria-describedby') || ''
    if (!existing.split(/\s+/).includes(this.bubbleId)) {
      first.setAttribute('aria-describedby', existing ? `${existing} ${this.bubbleId}` : this.bubbleId)
    }
  }

  connectedCallback() {
    super.connectedCallback()
    this.addEventListener('keydown', this.onKeydown)
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this.onKeydown)
  }

  render() {
    return html`
      <slot @slotchange=${this.wireTrigger}></slot>
      <span id=${this.bubbleId} class="bubble" role="tooltip"><slot name="content"></slot></span>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-tooltip')) {
  customElements.define('ship-tooltip', ShipTooltip)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-tooltip': ShipTooltip
  }
}
