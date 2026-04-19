/**
 * <ship-context-menu>
 *
 * Right-click (contextmenu) triggered overlay menu with keyboard navigation.
 * Reuses the same menu-item pattern as <ship-dropdown-menu> but activates
 * on right-click instead of left-click. Children with `role="menuitem"` or
 * `data-value` become selectable rows.
 *
 * Attributes:
 *   - open: boolean — reflects the visible state.
 *
 * Slots:
 *   - trigger — the element whose contextmenu event opens the menu.
 *   - (default) — menuitem children.
 *
 * Parts:
 *   - menu — the positioned menu panel.
 *
 * Events:
 *   - ship-select — CustomEvent { detail: { value } } when an item is
 *     activated via click, Enter, or Space.
 *   - ship-open  — dispatched when the menu opens.
 *   - ship-close — dispatched when the menu closes.
 */

import { LitElement, css, html } from 'lit'
import { Typeahead, isTypeaheadKey } from './typeahead'
import { reducedMotion } from './util'

export class ShipContextMenu extends LitElement {
  static styles = [reducedMotion, css`
    :host { display: contents; }
    .menu {
      position: fixed;
      z-index: var(--z-dropdown);
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
      transform: scale(0.95);
      pointer-events: none;
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([open]) .menu {
      opacity: 1;
      transform: scale(1);
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
    @media (forced-colors: active) {
      .menu { border: 1px solid CanvasText; }
    }
  `]

  static properties = {
    open: { type: Boolean, reflect: true },
  }

  open = false
  private typeahead = new Typeahead()
  private menuX = 0
  private menuY = 0

  private items(): HTMLElement[] {
    return Array.from(
      this.querySelectorAll<HTMLElement>('[role="menuitem"], button[data-value], [data-value]'),
    ).filter(el => !el.hasAttribute('disabled'))
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

  private setupItems() {
    const items = this.items()
    items.forEach((el, idx) => {
      if (!el.hasAttribute('role')) el.setAttribute('role', 'menuitem')
      if (!el.hasAttribute('tabindex')) el.setAttribute('tabindex', idx === 0 ? '0' : '-1')
    })
  }

  show(x: number, y: number) {
    this.menuX = x
    this.menuY = y
    this.open = true
    this.emit('ship-open')
    queueMicrotask(() => {
      this.setupItems()
      this.focusItem(0)
    })
  }

  close() {
    this.open = false
    this.emit('ship-close')
  }

  private emit(name: string) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
  }

  private onContextMenu = (e: MouseEvent) => {
    e.preventDefault()
    this.show(e.clientX, e.clientY)
  }

  private onItemClick = (e: Event) => {
    const target = (e.target as HTMLElement).closest('[data-value]') as HTMLElement | null
    if (!target) return
    this.dispatchEvent(new CustomEvent('ship-select', {
      detail: { value: target.dataset.value || '' },
      bubbles: true,
      composed: true,
    }))
    this.close()
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
    else if (e.key === 'Tab') { this.close() }
    else if ((e.key === 'Enter' || e.key === ' ') && i >= 0) {
      e.preventDefault()
      items[i].click()
    } else if (isTypeaheadKey(e)) {
      this.typeahead.push(e.key)
      const labels = items.map(el => (el.textContent ?? '').trim())
      const next = this.typeahead.match(labels, i)
      if (next >= 0) { e.preventDefault(); this.focusItem(next) }
    }
  }

  private onDocClick = (e: MouseEvent) => {
    if (!this.open) return
    const target = e.composedPath()[0] as Node
    if (!this.contains(target) && target !== this) this.close()
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

  updated() {
    this.setAttribute('data-state', this.open ? 'open' : 'closed')
    const menu = this.renderRoot.querySelector('.menu') as HTMLElement | null
    if (menu && this.open) {
      menu.style.left = `${this.menuX}px`
      menu.style.top = `${this.menuY}px`
    }
    this.setupItems()
  }

  render() {
    return html`
      <span @contextmenu=${this.onContextMenu}><slot name="trigger"></slot></span>
      <div class="menu" part="menu" role="menu"
        aria-label="Context menu"
        @click=${this.onItemClick}>
        <slot @slotchange=${() => this.setupItems()}></slot>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-context-menu')) {
  customElements.define('ship-context-menu', ShipContextMenu)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-context-menu': ShipContextMenu
  }
}
