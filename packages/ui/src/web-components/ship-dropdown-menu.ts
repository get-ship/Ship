/**
 * <ship-dropdown-menu>
 *
 * Popover with a vertical list of actionable items. Light-DOM children
 * with `role="menuitem"` and a `data-value` attr become selectable rows.
 * Use child <button data-value="..."> elements for best semantics.
 *
 * Attributes:
 *   - open: boolean
 *
 * Slots:
 *   - trigger — the triggering button
 *   - (default) menuitem children
 *
 * Events:
 *   - ship-select — CustomEvent { detail: { value } } when an item is
 *     clicked or activated with Enter/Space.
 */

import { LitElement, css, html } from 'lit'
import { Typeahead, isTypeaheadKey } from './typeahead'
export class ShipDropdownMenu extends LitElement {
  static styles = css`
    :host { position: relative; display: inline-flex; }
    .menu {
      position: absolute;
      z-index: var(--z-dropdown);
      top: calc(100% + 4px);
      left: 0;
      min-width: 160px;
      padding: 4px;
      background: var(--popover);
      color: var(--popover-foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      box-shadow: var(--shadow-lg);
      display: flex; flex-direction: column;
      font-family: var(--font-body);
      font-size: var(--text-base);
      opacity: 0;
      transform: translateY(-4px);
      pointer-events: none;
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    :host([open]) .menu {
      opacity: 1;
      transform: translateY(0);
      pointer-events: auto;
    }
    ::slotted(button), ::slotted([role='menuitem']) {
      text-align: left;
      background: transparent; border: 0;
      padding: 6px 10px;
      border-radius: var(--radius-sm);
      color: inherit; font: inherit;
      cursor: pointer;
      transition: background-color var(--duration-fast) var(--ease-out);
    }
    ::slotted(button:hover), ::slotted([role='menuitem']:hover) {
      background: var(--muted);
    }
  `

  static properties = {
    open: { type: Boolean, reflect: true },
  }

  open = false
  private typeahead = new Typeahead()
  private onTriggerClick = () => {
    this.open = !this.open
    if (this.open) queueMicrotask(() => this.focusItem(0))
  }

  private items(): HTMLElement[] {
    return Array.from(this.querySelectorAll<HTMLElement>('[role="menuitem"], button[data-value], [data-value]'))
      .filter(el => !el.hasAttribute('disabled'))
  }

  private focusItem(index: number) {
    const items = this.items()
    if (!items.length) return
    const i = (index + items.length) % items.length
    for (const it of items) it.setAttribute('tabindex', '-1')
    items[i].setAttribute('tabindex', '0')
    items[i].focus()
  }

  private currentIndex(): number {
    const items = this.items()
    return items.indexOf(document.activeElement as HTMLElement)
  }

  private triggerEl(): HTMLElement | null {
    const slot = this.renderRoot.querySelector('slot[name="trigger"]') as HTMLSlotElement | null
    const assigned = slot?.assignedElements({ flatten: true }) as HTMLElement[] | undefined
    return assigned?.[0] ?? null
  }

  private onItemClick = (e: Event) => {
    const target = (e.target as HTMLElement).closest('[data-value]') as HTMLElement | null
    if (!target) return
    const value = target.dataset.value || ''
    this.dispatchEvent(new CustomEvent('ship-select', {
      detail: { value }, bubbles: true, composed: true,
    }))
    this.close()
  }

  private close() {
    this.open = false
    this.triggerEl()?.focus()
  }

  private onKeydown = (e: KeyboardEvent) => {
    if (!this.open) return
    const items = this.items()
    if (!items.length) return
    const i = this.currentIndex()
    if (e.key === 'ArrowDown') { e.preventDefault(); this.focusItem(i + 1) }
    else if (e.key === 'ArrowUp') { e.preventDefault(); this.focusItem(i - 1) }
    else if (e.key === 'Home') { e.preventDefault(); this.focusItem(0) }
    else if (e.key === 'End') { e.preventDefault(); this.focusItem(items.length - 1) }
    else if (e.key === 'Escape') { e.preventDefault(); this.close() }
    else if (e.key === 'Tab') { this.open = false }
    else if ((e.key === 'Enter' || e.key === ' ') && i >= 0) {
      e.preventDefault()
      items[i].click()
    }
    else if (isTypeaheadKey(e)) {
      this.typeahead.push(e.key)
      const labels = items.map((el) => (el.textContent ?? '').trim())
      const next = this.typeahead.match(labels, i)
      if (next >= 0) { e.preventDefault(); this.focusItem(next) }
    }
  }

  private onDocClick = (e: MouseEvent) => {
    if (!this.open) return
    const target = e.composedPath()[0] as Node
    if (!this.contains(target) && target !== this) this.open = false
  }

  private setupTrigger() {
    const t = this.triggerEl()
    if (!t) return
    t.setAttribute('aria-haspopup', 'menu')
    t.setAttribute('aria-expanded', this.open ? 'true' : 'false')
  }

  private setupItems() {
    const items = this.items()
    items.forEach((el, idx) => {
      if (!el.hasAttribute('role')) el.setAttribute('role', 'menuitem')
      if (!el.hasAttribute('tabindex')) el.setAttribute('tabindex', idx === 0 ? '0' : '-1')
    })
  }

  updated() {
    this.setAttribute('data-state', this.open ? 'open' : 'closed')
    this.setupTrigger()
    this.setupItems()
  }

  connectedCallback() {
    super.connectedCallback()
    document.addEventListener('click', this.onDocClick)
    this.addEventListener('keydown', this.onKeydown)
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener('click', this.onDocClick)
    this.removeEventListener('keydown', this.onKeydown)
  }

  render() {
    return html`
      <span @click=${this.onTriggerClick}><slot name="trigger" @slotchange=${() => this.setupTrigger()}></slot></span>
      <div class="menu" role="menu" @click=${this.onItemClick}>
        <slot @slotchange=${() => this.setupItems()}></slot>
      </div>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-dropdown-menu')) {
  customElements.define('ship-dropdown-menu', ShipDropdownMenu)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-dropdown-menu': ShipDropdownMenu
  }
}
