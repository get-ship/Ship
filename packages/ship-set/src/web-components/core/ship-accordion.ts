/**
 * <ship-accordion> + <ship-accordion-item>
 *
 * Declarative accordion matching Radix Accordion behavior. Items are
 * composed as children. Supports single (one open at a time) or
 * multiple (many open) mode.
 *
 * Attributes on <ship-accordion>:
 *   - type: single | multiple (default: single)
 *   - value: string — comma-separated list of open item values
 *
 * Attributes on <ship-accordion-item>:
 *   - value: string (required) — unique identifier
 *   - disabled: boolean
 *   - open: boolean (reflect) — managed by parent
 *
 * Slots on <ship-accordion-item>:
 *   - trigger — the header content
 *   - (default) — collapsible body content
 *
 * Parts on <ship-accordion-item>:
 *   - trigger — the button element
 *   - content — the collapsible region
 *   - chevron — the rotate indicator
 *
 * Events on <ship-accordion>:
 *   - ship-change — CustomEvent { detail: { value } }, bubbles, composed.
 *     value is a string (single) or comma-separated string (multiple).
 */

import { LitElement, css, html } from 'lit'
import {
  shipAccordionStyles,
  shipAccordionItemStyles,
} from './ship-accordion.styles'

export class ShipAccordionItem extends LitElement {
  static styles = shipAccordionItemStyles

  static properties = {
    value: { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
    open: { type: Boolean, reflect: true },
  }

  value = ''
  disabled = false
  open = false

  private static _idSeq = 0
  private uid = ++ShipAccordionItem._idSeq
  private get triggerId(): string { return `ship-acc-trigger-${this.uid}` }
  private get contentId(): string { return `ship-acc-content-${this.uid}` }

  private onTriggerClick = () => {
    if (this.disabled) return
    this.dispatchEvent(new CustomEvent('ship-accordion-item-toggle', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  private onKeydown = (e: KeyboardEvent) => {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault()
      this.dispatchEvent(new CustomEvent('ship-accordion-nav', {
        detail: { direction: e.key === 'ArrowDown' ? 'next' : 'prev' },
        bubbles: true,
        composed: true,
      }))
    } else if (e.key === 'Home' || e.key === 'End') {
      e.preventDefault()
      this.dispatchEvent(new CustomEvent('ship-accordion-nav', {
        detail: { direction: e.key === 'Home' ? 'first' : 'last' },
        bubbles: true,
        composed: true,
      }))
    }
  }

  /** Focus the trigger button from the parent accordion's keyboard handler. */
  focusTrigger() {
    const btn = this.renderRoot.querySelector('button') as HTMLButtonElement | null
    btn?.focus()
  }

  render() {
    return html`
      <button
        class="trigger"
        part="trigger"
        id=${this.triggerId}
        aria-expanded=${this.open ? 'true' : 'false'}
        aria-controls=${this.contentId}
        ?disabled=${this.disabled}
        @click=${this.onTriggerClick}
        @keydown=${this.onKeydown}
      >
        <slot name="trigger"></slot>
        <svg class="chevron" part="chevron" viewBox="0 0 16 16" fill="none"
          stroke="currentColor" stroke-width="1.75" stroke-linecap="round"
          stroke-linejoin="round" aria-hidden="true">
          <path d="M4 6l4 4 4-4"></path>
        </svg>
      </button>
      <div class="content-wrap" part="content"
        id=${this.contentId}
        role="region"
        aria-labelledby=${this.triggerId}
      >
        <div class="content-inner">
          <div class="content-body">
            <slot></slot>
          </div>
        </div>
      </div>
    `
  }
}

export class ShipAccordion extends LitElement {
  static styles = shipAccordionStyles

  static properties = {
    type: { type: String, reflect: true },
    value: { type: String, reflect: true },
  }

  type: 'single' | 'multiple' = 'single'
  value = ''

  connectedCallback() {
    super.connectedCallback()
    this.addEventListener('ship-accordion-item-toggle', this.onItemToggle as EventListener)
    this.addEventListener('ship-accordion-nav', this.onItemNav as EventListener)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('ship-accordion-item-toggle', this.onItemToggle as EventListener)
    this.removeEventListener('ship-accordion-nav', this.onItemNav as EventListener)
  }

  private get items(): ShipAccordionItem[] {
    return Array.from(
      this.querySelectorAll(':scope > ship-accordion-item'),
    ) as ShipAccordionItem[]
  }

  private get enabledItems(): ShipAccordionItem[] {
    return this.items.filter(i => !i.disabled)
  }

  private getOpenValues(): Set<string> {
    return new Set(this.value ? this.value.split(',').filter(Boolean) : [])
  }

  private setOpenValues(vals: Set<string>) {
    this.value = Array.from(vals).join(',')
    this.syncItems()
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  private onItemToggle = (e: CustomEvent) => {
    e.stopPropagation()
    const itemVal = e.detail.value as string
    const open = this.getOpenValues()

    if (this.type === 'single') {
      if (open.has(itemVal)) {
        this.setOpenValues(new Set())
      } else {
        this.setOpenValues(new Set([itemVal]))
      }
    } else {
      if (open.has(itemVal)) {
        open.delete(itemVal)
      } else {
        open.add(itemVal)
      }
      this.setOpenValues(open)
    }
  }

  private onItemNav = (e: CustomEvent) => {
    e.stopPropagation()
    const enabled = this.enabledItems
    if (!enabled.length) return

    const current = (e.target as HTMLElement).closest('ship-accordion-item') as ShipAccordionItem | null
    const idx = current ? enabled.indexOf(current) : -1
    let target: ShipAccordionItem | undefined

    switch (e.detail.direction) {
      case 'next':
        target = enabled[(idx + 1) % enabled.length]
        break
      case 'prev':
        target = enabled[(idx - 1 + enabled.length) % enabled.length]
        break
      case 'first':
        target = enabled[0]
        break
      case 'last':
        target = enabled[enabled.length - 1]
        break
    }
    target?.focusTrigger()
  }

  updated() {
    this.syncItems()
  }

  private syncItems() {
    const open = this.getOpenValues()
    for (const item of this.items) {
      item.open = open.has(item.value)
    }
  }

  render() {
    return html`<slot></slot>`
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-accordion-item')) {
    customElements.define('ship-accordion-item', ShipAccordionItem)
  }
  if (!customElements.get('ship-accordion')) {
    customElements.define('ship-accordion', ShipAccordion)
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-accordion': ShipAccordion
    'ship-accordion-item': ShipAccordionItem
  }
}
