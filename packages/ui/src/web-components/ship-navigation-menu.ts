/**
 * <ship-navigation-menu> + <ship-navigation-item>
 *
 * Horizontal navigation bar with an active-item indicator. Simplified v1:
 * horizontal links with active state and keyboard navigation. No dropdown
 * submenus in this version.
 *
 * === <ship-navigation-menu> ===
 *
 * Attributes:
 *   - value: string — the active item's value.
 *
 * Slots:
 *   - (default) — <ship-navigation-item> children.
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { value } } when the active item
 *     changes via click or keyboard.
 *
 * === <ship-navigation-item> ===
 *
 * Attributes:
 *   - value: string (required) — unique identifier.
 *   - active: boolean — reflects whether this item is the current selection.
 *   - disabled: boolean
 *   - href: string — optional link URL. If provided, renders an anchor.
 *
 * Slots:
 *   - (default) — label content.
 *
 * Parts:
 *   - link — the interactive element (button or anchor).
 *   - indicator — the active indicator line.
 */

import { LitElement, css, html } from 'lit'
import { reducedMotion, focusRingStyle, focusRingForcedColors } from './util'

export class ShipNavigationItem extends LitElement {
  static styles = [reducedMotion, css`
    :host { display: inline-flex; position: relative; }
    :host([hidden]) { display: none; }

    .link {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 8px 14px;
      font-family: var(--font-body);
      font-size: var(--text-base);
      font-weight: var(--font-weight-medium);
      color: var(--muted-foreground);
      text-decoration: none;
      background: transparent;
      border: none;
      cursor: pointer;
      transition:
        color var(--duration-fast) var(--ease-out),
        background-color var(--duration-fast) var(--ease-out);
      line-height: var(--leading-normal);
      border-radius: var(--radius);
    }
    .link:hover {
      color: var(--foreground);
      background: color-mix(in oklch, var(--muted) 50%, transparent);
    }
    :host([active]) .link {
      color: var(--foreground);
      font-weight: var(--font-weight-semibold);
    }
    :host([disabled]) .link {
      cursor: not-allowed;
      opacity: 0.5;
      pointer-events: none;
    }
    .link:focus-visible {
      ${focusRingStyle}
    }

    /* Active indicator line */
    .indicator {
      position: absolute;
      bottom: 0;
      left: 14px;
      right: 14px;
      height: 2px;
      background: var(--foreground);
      border-radius: var(--radius-full);
      opacity: 0;
      transform: scaleX(0);
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([active]) .indicator {
      opacity: 1;
      transform: scaleX(1);
    }

    @media (forced-colors: active) {
      .link:focus-visible {
        ${focusRingForcedColors}
      }
      :host([active]) .indicator {
        background: CanvasText;
      }
    }
  `]

  static properties = {
    value: { type: String, reflect: true },
    active: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    href: { type: String },
  }

  value = ''
  active = false
  disabled = false
  href = ''

  private onClick = (e: Event) => {
    if (this.disabled) return
    if (!this.href) e.preventDefault()
    this.dispatchEvent(new CustomEvent('ship-nav-select', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  /** Focus the interactive element — used by parent nav for keyboard nav. */
  focusLink() {
    const el = this.renderRoot.querySelector('.link') as HTMLElement | null
    el?.focus()
  }

  render() {
    if (this.href) {
      return html`
        <a class="link" part="link"
          href=${this.href}
          aria-current=${this.active ? 'page' : 'false'}
          @click=${this.onClick}
        ><slot></slot></a>
        <div class="indicator" part="indicator" aria-hidden="true"></div>
      `
    }
    return html`
      <button class="link" part="link" type="button"
        aria-current=${this.active ? 'true' : 'false'}
        ?disabled=${this.disabled}
        @click=${this.onClick}
      ><slot></slot></button>
      <div class="indicator" part="indicator" aria-hidden="true"></div>
    `
  }
}

export class ShipNavigationMenu extends LitElement {
  static styles = [reducedMotion, css`
    :host {
      display: flex;
      align-items: stretch;
      gap: 0;
      border-bottom: 1px solid var(--border);
      font-family: var(--font-body);
    }
    :host([hidden]) { display: none; }
  `]

  static properties = {
    value: { type: String, reflect: true },
  }

  value = ''

  private get navItems(): ShipNavigationItem[] {
    return Array.from(
      this.querySelectorAll(':scope > ship-navigation-item'),
    ) as ShipNavigationItem[]
  }

  private get enabledItems(): ShipNavigationItem[] {
    return this.navItems.filter(i => !i.disabled)
  }

  private onNavSelect = (e: CustomEvent) => {
    e.stopPropagation()
    const val = e.detail.value as string
    this.value = val
    this.syncItems()
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: val },
      bubbles: true,
      composed: true,
    }))
  }

  private onKeydown = (e: KeyboardEvent) => {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft'
      && e.key !== 'Home' && e.key !== 'End') return
    const items = this.enabledItems
    if (!items.length) return
    const current = (e.composedPath().find(
      n => items.includes(n as ShipNavigationItem),
    ) as ShipNavigationItem) || null
    const i = current ? items.indexOf(current) : -1
    let target: ShipNavigationItem | undefined

    if (e.key === 'ArrowRight') target = items[(i + 1) % items.length]
    else if (e.key === 'ArrowLeft') target = items[(i - 1 + items.length) % items.length]
    else if (e.key === 'Home') target = items[0]
    else if (e.key === 'End') target = items[items.length - 1]

    if (target) { e.preventDefault(); target.focusLink() }
  }

  private syncItems() {
    for (const item of this.navItems) {
      item.active = item.value === this.value
    }
  }

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'navigation')
    this.addEventListener('ship-nav-select', this.onNavSelect as EventListener)
    this.addEventListener('keydown', this.onKeydown)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('ship-nav-select', this.onNavSelect as EventListener)
    this.removeEventListener('keydown', this.onKeydown)
  }

  updated() {
    this.syncItems()
  }

  render() {
    return html`<slot @slotchange=${() => this.syncItems()}></slot>`
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-navigation-item')) {
    customElements.define('ship-navigation-item', ShipNavigationItem)
  }
  if (!customElements.get('ship-navigation-menu')) {
    customElements.define('ship-navigation-menu', ShipNavigationMenu)
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-navigation-item': ShipNavigationItem
    'ship-navigation-menu': ShipNavigationMenu
  }
}
