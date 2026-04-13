/**
 * <ship-tabs> + <ship-tab> + <ship-tab-panel>
 *
 * Declarative child-composition tabs. The active tab is the one with
 * `name` matching the <ship-tabs active="..."> attribute.
 *
 * Two visual variants:
 *   - underline (default) — pipelines `.tabs-bar` style (bottom border,
 *     highlight bar).
 *   - segmented — specs `.tabs` style (pill background, active has white
 *     chip with soft shadow). Uppercase micro-labels.
 *
 * Attributes on <ship-tabs>:
 *   - active: name of the active tab
 *   - variant: underline | segmented (default underline)
 *   - sticky: boolean — sticky top positioning (pipelines uses this)
 *
 * Attributes on <ship-tab>:
 *   - name (required)
 *   - label (if omitted, name is used)
 *   - count: optional numeric count rendered as a pill chip
 *   - disabled
 *
 * Attributes on <ship-tab-panel>:
 *   - name (required) — shown when matches parent <ship-tabs active>.
 *
 * Events (on <ship-tabs>):
 *   - ship-tab-change — CustomEvent { detail: { name } }
 */

import { LitElement, css, html } from 'lit'

export class ShipTab extends LitElement {
  static styles = css`:host { display: none; }`
  static properties = {
    name: { type: String },
    label: { type: String },
    count: { type: Number },
    disabled: { type: Boolean },
  }
  name = ''
  label = ''
  count: number | null = null
  disabled = false
  render() { return html`` }
}

export class ShipTabPanel extends LitElement {
  static styles = css`
    :host { display: none; }
    :host([active]) { display: block; animation: fade 180ms ease; }
    @keyframes fade {
      from { opacity: 0; transform: translateY(4px); }
      to { opacity: 1; transform: translateY(0); }
    }
  `
  static properties = {
    name: { type: String },
    active: { type: Boolean, reflect: true },
  }
  name = ''
  active = false
  render() { return html`<slot></slot>` }
}

export class ShipTabs extends LitElement {
  static styles = css`
    :host { display: block; font-family: var(--font-body); }

    /* Underline variant (default) — matches pipelines .tabs-bar */
    .tablist.underline {
      display: flex; gap: 2px;
      border-bottom: 1px solid var(--border);
      margin-bottom: 24px;
    }
    :host([sticky]) .tablist.underline {
      position: sticky; top: 0; background: var(--background); z-index: 5;
      padding-top: 4px;
    }
    .tablist.underline button {
      background: transparent; border: 0;
      color: var(--muted-foreground);
      padding: 10px 16px;
      font: inherit;
      font-size: 13px; font-weight: 500;
      cursor: pointer;
      border-bottom: 2px solid transparent;
      margin-bottom: -1px;
      display: inline-flex; align-items: center; gap: 8px;
      transition: color 120ms;
    }
    .tablist.underline button:hover:not(:disabled) { color: var(--foreground); }
    .tablist.underline button[aria-selected='true'] {
      color: var(--foreground);
      border-bottom-color: var(--primary);
    }

    /* Segmented variant — matches specs .tabs */
    .tablist.segmented {
      display: flex; gap: 2px;
      padding: 8px 10px;
      border-bottom: 1px solid var(--border);
      background: var(--muted);
    }
    .tablist.segmented button {
      flex: 1;
      padding: 6px 8px;
      border: none;
      background: transparent;
      color: var(--muted-foreground);
      font: inherit;
      font-size: 11px; font-weight: 500;
      cursor: pointer;
      border-radius: 4px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    .tablist.segmented button[aria-selected='true'] {
      background: var(--background);
      color: var(--foreground);
      box-shadow: 0 1px 2px rgba(0,0,0,0.05);
    }

    button:disabled { opacity: 0.5; cursor: not-allowed; }
    button:focus-visible {
      outline: 2px solid var(--primary);
      outline-offset: -2px;
      border-radius: 4px;
    }

    .count {
      background: var(--muted);
      color: var(--muted-foreground);
      padding: 1px 7px;
      border-radius: 999px;
      font-size: 10px; font-weight: 600;
      font-variant-numeric: tabular-nums;
      min-width: 18px;
      text-align: center;
    }
    .tablist.underline button[aria-selected='true'] .count {
      background: color-mix(in oklch, var(--primary) 15%, transparent);
      color: var(--primary);
    }

    .panels { display: block; }
  `

  static properties = {
    active: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    sticky: { type: Boolean, reflect: true },
  }

  active = ''
  variant: 'underline' | 'segmented' = 'underline'
  sticky = false

  private get tabs(): ShipTab[] {
    return Array.from(this.querySelectorAll(':scope > ship-tab')) as ShipTab[]
  }
  private get panels(): ShipTabPanel[] {
    return Array.from(this.querySelectorAll(':scope > ship-tab-panel')) as ShipTabPanel[]
  }

  firstUpdated() { this.syncPanels() }
  updated() { this.syncPanels() }
  private syncPanels() {
    for (const p of this.panels) p.active = p.name === this.active
  }

  private activate(name: string) {
    this.active = name
    this.dispatchEvent(new CustomEvent('ship-tab-change', {
      detail: { name }, bubbles: true, composed: true,
    }))
  }

  private onKeydown(e: KeyboardEvent) {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
    e.preventDefault()
    const tabs = this.tabs.filter(t => !t.disabled)
    const names = tabs.map(t => t.name)
    const i = names.indexOf(this.active)
    const next = e.key === 'ArrowRight'
      ? (i + 1) % names.length
      : (i - 1 + names.length) % names.length
    this.activate(names[next])
  }

  render() {
    const tabs = this.tabs.map(t => ({
      name: t.name,
      label: t.label || t.name,
      count: t.count,
      disabled: t.disabled,
    }))

    return html`
      <div class="tablist ${this.variant}" role="tablist" @keydown=${this.onKeydown}>
        ${tabs.map(t => html`
          <button
            role="tab"
            aria-selected=${t.name === this.active ? 'true' : 'false'}
            tabindex=${t.name === this.active ? '0' : '-1'}
            ?disabled=${t.disabled}
            @click=${() => this.activate(t.name)}
          >
            ${t.label}
            ${t.count != null ? html`<span class="count">${t.count}</span>` : null}
          </button>
        `)}
      </div>
      <div class="panels"><slot></slot></div>
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
