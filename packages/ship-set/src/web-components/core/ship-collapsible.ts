/**
 * <ship-collapsible>
 *
 * Simple open/close container with smooth height animation. Matches
 * Radix Collapsible behavior: a trigger slot toggles a content region.
 *
 * Attributes:
 *   - open: boolean — whether the content is visible
 *   - disabled: boolean — prevents toggling
 *
 * Slots:
 *   - trigger — the element that toggles open/close
 *   - (default) — collapsible content
 *
 * Parts:
 *   - content — the wrapper around the collapsible region
 *
 * Events:
 *   - ship-toggle — CustomEvent { detail: { open } }, bubbles, composed.
 */

import { LitElement, html } from 'lit'
import { shipCollapsibleStyles } from './ship-collapsible.styles'

export class ShipCollapsible extends LitElement {
  static styles = shipCollapsibleStyles

  static properties = {
    open: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
  }

  open = false
  disabled = false

  private static _idSeq = 0
  private uid = ++ShipCollapsible._idSeq
  private get contentId(): string { return `ship-collapsible-${this.uid}` }

  private toggle = () => {
    if (this.disabled) return
    this.open = !this.open
    this.dispatchEvent(new CustomEvent('ship-toggle', {
      detail: { open: this.open },
      bubbles: true,
      composed: true,
    }))
  }

  private onTriggerKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this.toggle()
    }
  }

  private onTriggerSlotChange = (e: Event) => {
    const slot = e.target as HTMLSlotElement
    for (const el of slot.assignedElements()) {
      const h = el as HTMLElement
      h.setAttribute('aria-expanded', String(this.open))
      h.setAttribute('aria-controls', this.contentId)
      if (!h.hasAttribute('tabindex') && !h.matches('button, a, input, select, textarea')) {
        h.setAttribute('tabindex', '0')
      }
    }
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('open')) {
      // Keep trigger aria-expanded in sync
      const slot = this.renderRoot.querySelector('slot[name=trigger]') as HTMLSlotElement | null
      if (slot) {
        for (const el of slot.assignedElements()) {
          (el as HTMLElement).setAttribute('aria-expanded', String(this.open))
        }
      }
    }
  }

  render() {
    return html`
      <div @click=${this.toggle} @keydown=${this.onTriggerKeydown}>
        <slot name="trigger" @slotchange=${this.onTriggerSlotChange}></slot>
      </div>
      <div class="content-wrap" part="content"
        id=${this.contentId}
        role="region"
      >
        <div class="content-inner">
          <slot></slot>
        </div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-collapsible')) {
  customElements.define('ship-collapsible', ShipCollapsible)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-collapsible': ShipCollapsible
  }
}
