/**
 * <ship-tabs> + <ship-tab> + <ship-tab-panel>
 *
 * Declarative child-composition tabs. The active tab is the one whose
 * `name` matches <ship-tabs active="…">. Matches shadcn/ui's Tabs + Radix
 * Tabs visual weight: clean text tabs with an animated underline, or a
 * pill-group segmented control.
 *
 * Attributes on <ship-tabs>:
 *   - active — name of the active tab (also reflected on the element)
 *   - variant — underline | segmented (default: underline)
 *   - size — sm | md | lg (default: md)
 *   - tone — default | info | success | warning | destructive | accent | muted
 *     Drives the active-tab accent color (underline + focus ring + count chip).
 *   - sticky — boolean; sticky top positioning (pipelines uses this)
 *   - orientation — horizontal | vertical (default: horizontal)
 *
 * Attributes on <ship-tab>:
 *   - name (required)
 *   - label — if omitted, name is used
 *   - count — optional numeric count rendered as a pill chip after the label
 *   - disabled
 *
 * Attributes on <ship-tab-panel>:
 *   - name (required) — shown when matches parent <ship-tabs active>.
 *
 * Slots on <ship-tabs>:
 *   - (default) — projected children: <ship-tab> and <ship-tab-panel>
 *
 * Parts:
 *   - tablist — the role="tablist" wrapper
 *   - tab — each role="tab" button
 *   - indicator — the moving underline pseudo-element owner (tablist::after)
 *
 * Events (on <ship-tabs>):
 *   - ship-tab-change — CustomEvent { detail: { name } }
 */

import { LitElement, css, html } from 'lit'
import { shipTabsStyles, shipTabPanelStyles } from './ship-tabs.styles'
import type { Size, Tone } from './util'

export class ShipTab extends LitElement {
  static styles = css`:host { display: none; }`
  static properties = {
    name: { type: String, reflect: true },
    label: { type: String, reflect: true },
    count: { type: Number, reflect: true },
    disabled: { type: Boolean, reflect: true },
  }
  name = ''
  label = ''
  count: number | null = null
  disabled = false
  render() { return html`` }
}

export class ShipTabPanel extends LitElement {
  static styles = shipTabPanelStyles
  static properties = {
    name: { type: String, reflect: true },
    active: { type: Boolean, reflect: true },
  }
  name = ''
  active = false
  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'tabpanel')
    if (!this.hasAttribute('tabindex')) this.setAttribute('tabindex', '0')
    if (this.name && !this.id) this.id = `ship-tabpanel-${this.name}`
    if (this.name) this.setAttribute('aria-labelledby', `ship-tab-${this.name}`)
  }
  render() { return html`<slot></slot>` }
}

export class ShipTabs extends LitElement {
  static styles = shipTabsStyles

  static properties = {
    active: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    sticky: { type: Boolean, reflect: true },
    orientation: { type: String, reflect: true },
  }

  active = ''
  variant: 'underline' | 'segmented' = 'underline'
  size: Size = 'md'
  tone?: Tone
  sticky = false
  orientation: 'horizontal' | 'vertical' = 'horizontal'

  /** Observes the tab children so label/count/disabled edits after mount
   * trigger a re-render. Also catches tabs added/removed from the slot. */
  private mutationObserver?: MutationObserver
  /** Observes the tablist container for size changes so the underline
   * indicator repositions when the layout reflows (sidebar open/close,
   * window resize, font load). */
  private resizeObserver?: ResizeObserver
  /** Underline-indicator measurement runs through rAF so we never write
   * custom properties more than once per frame even under event storms. */
  private rafId = 0

  connectedCallback() {
    super.connectedCallback()
    this.mutationObserver = new MutationObserver(() => this.requestUpdate())
    this.mutationObserver.observe(this, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['name', 'label', 'count', 'disabled'],
    })
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    this.mutationObserver?.disconnect()
    this.resizeObserver?.disconnect()
    if (this.rafId) cancelAnimationFrame(this.rafId)
  }

  private get tabs(): ShipTab[] {
    return Array.from(this.querySelectorAll(':scope > ship-tab')) as ShipTab[]
  }
  private get panels(): ShipTabPanel[] {
    return Array.from(this.querySelectorAll(':scope > ship-tab-panel')) as ShipTabPanel[]
  }

  firstUpdated() {
    this.syncPanels()
    this.attachResizeObserver()
    this.scheduleIndicatorSync()
  }
  updated(changed: Map<string, unknown>) {
    this.syncPanels()
    // Any change that could shift the active tab's geometry needs a sync:
    // active-tab change, variant swap, orientation flip, size change.
    if (
      changed.has('active') ||
      changed.has('variant') ||
      changed.has('orientation') ||
      changed.has('size')
    ) {
      this.scheduleIndicatorSync()
    }
  }

  private syncPanels() {
    for (const p of this.panels) p.active = p.name === this.active
  }

  private attachResizeObserver() {
    const tablist = this.renderRoot.querySelector('.tablist') as HTMLElement | null
    if (!tablist || typeof ResizeObserver === 'undefined') return
    this.resizeObserver = new ResizeObserver(() => this.scheduleIndicatorSync())
    this.resizeObserver.observe(tablist)
  }

  /** Coalesce indicator updates through rAF. We measure the active tab's
   * offset within the tablist and write the geometry into CSS custom
   * properties; the pseudo-element transition animates the move. */
  private scheduleIndicatorSync() {
    if (this.variant === 'segmented') return
    if (this.rafId) cancelAnimationFrame(this.rafId)
    this.rafId = requestAnimationFrame(() => this.syncIndicator())
  }

  private syncIndicator() {
    this.rafId = 0
    const tablist = this.renderRoot.querySelector('.tablist') as HTMLElement | null
    if (!tablist) return
    const activeBtn = tablist.querySelector<HTMLButtonElement>(
      'button[aria-selected="true"]',
    )
    if (!activeBtn) {
      this.removeAttribute('data-underline-ready')
      return
    }
    if (this.orientation === 'vertical') {
      tablist.style.setProperty('--underline-y', `${activeBtn.offsetTop}px`)
      tablist.style.setProperty('--underline-h', `${activeBtn.offsetHeight}px`)
    } else {
      tablist.style.setProperty('--underline-x', `${activeBtn.offsetLeft}px`)
      tablist.style.setProperty('--underline-w', `${activeBtn.offsetWidth}px`)
    }
    this.setAttribute('data-underline-ready', '')
  }

  private activate(name: string) {
    if (this.active === name) return
    this.active = name
    this.dispatchEvent(new CustomEvent('ship-tab-change', {
      detail: { name }, bubbles: true, composed: true,
    }))
  }

  private onKeydown(e: KeyboardEvent) {
    const vertical = this.orientation === 'vertical'
    const nextKey = vertical ? 'ArrowDown' : 'ArrowRight'
    const prevKey = vertical ? 'ArrowUp' : 'ArrowLeft'
    const nav = [nextKey, prevKey, 'Home', 'End']
    const activate = ['Enter', ' ']
    if (!nav.includes(e.key) && !activate.includes(e.key)) return
    e.preventDefault()
    const enabled = this.tabs.filter(t => !t.disabled)
    if (!enabled.length) return
    const names = enabled.map(t => t.name)
    const i = Math.max(0, names.indexOf(this.active))
    let next = i
    if (e.key === nextKey) next = (i + 1) % names.length
    else if (e.key === prevKey) next = (i - 1 + names.length) % names.length
    else if (e.key === 'Home') next = 0
    else if (e.key === 'End') next = names.length - 1
    // Enter/Space activate the currently-focused tab (if different from
    // active) so manual-activation consumers still get the event.
    const targetName = names[next]
    this.activate(targetName)
    queueMicrotask(() => {
      const btn = this.renderRoot.querySelector<HTMLButtonElement>(
        `button[data-name="${targetName}"]`,
      )
      btn?.focus()
    })
  }

  render() {
    const tabs = this.tabs.map((t, idx) => ({
      index: idx,
      name: t.name,
      label: t.label || t.name,
      count: t.count,
      disabled: t.disabled,
    }))

    return html`
      <div class="tablist ${this.variant}" part="tablist" role="tablist"
           aria-orientation=${this.orientation}
           @keydown=${this.onKeydown}>
        ${tabs.map(t => html`
          <button
            class="tab"
            part="tab"
            role="tab"
            id=${`ship-tab-${t.name}`}
            data-name=${t.name}
            aria-selected=${t.name === this.active ? 'true' : 'false'}
            aria-controls=${`ship-tabpanel-${t.name}`}
            aria-disabled=${t.disabled ? 'true' : 'false'}
            tabindex=${t.name === this.active ? '0' : '-1'}
            ?disabled=${t.disabled}
            @click=${() => !t.disabled && this.activate(t.name)}
          >
            <span class="label"><slot name=${`label-${t.name}`}>${t.label}</slot></span>
            ${t.count != null
              ? html`<span class="count" aria-hidden="true">${t.count}</span>`
              : null}
          </button>
        `)}
      </div>
      <div class="panels" part="panels"><slot></slot></div>
    `
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-tab')) customElements.define('ship-tab', ShipTab)
  if (!customElements.get('ship-tab-panel')) customElements.define('ship-tab-panel', ShipTabPanel)
  if (!customElements.get('ship-tabs')) customElements.define('ship-tabs', ShipTabs)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-tabs': ShipTabs
    'ship-tab': ShipTab
    'ship-tab-panel': ShipTabPanel
  }
}
