/**
 * <ship-toggle> + <ship-toggle-group>
 *
 * Toggle button and group container. <ship-toggle> is a pressed/unpressed
 * button. <ship-toggle-group> manages single or multiple selection across
 * its children with arrow-key navigation.
 *
 * === <ship-toggle> ===
 *
 * Attributes:
 *   - pressed: boolean — reflects the pressed state.
 *   - disabled: boolean
 *   - value: string — identifier for group selection.
 *   - size: sm | md | lg (default: md)
 *   - variant: outline | ghost (default: outline)
 *
 * Slots:
 *   - (default) — button content (text, icon, etc.)
 *
 * Parts:
 *   - button — the native <button>.
 *
 * Events:
 *   - ship-toggle — CustomEvent { detail: { pressed, value } }
 *
 * === <ship-toggle-group> ===
 *
 * Attributes:
 *   - type: single | multiple (default: single)
 *   - value: string — comma-separated selected toggle values.
 *   - size: sm | md | lg — propagated to children.
 *   - variant: outline | ghost — propagated to children.
 *
 * Slots:
 *   - (default) — <ship-toggle> children.
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { value } }
 */

import { LitElement, css, html } from 'lit'
import { reducedMotion, focusRingStyle, focusRingForcedColors } from './util'

export class ShipToggle extends LitElement {
  static styles = [reducedMotion, css`
    :host { display: inline-flex; }
    :host([hidden]) { display: none; }

    /* Size channel */
    :host(:not([size])), :host([size='md']) {
      --tgl-h: 36px; --tgl-px: 12px; --tgl-fs: var(--text-base);
    }
    :host([size='sm']) {
      --tgl-h: 28px; --tgl-px: 8px; --tgl-fs: var(--text-sm);
    }
    :host([size='lg']) {
      --tgl-h: 42px; --tgl-px: 16px; --tgl-fs: var(--text-base);
    }

    button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      height: var(--tgl-h);
      padding: 0 var(--tgl-px);
      font-family: var(--font-body);
      font-size: var(--tgl-fs);
      font-weight: var(--font-weight-medium);
      line-height: 1;
      color: var(--muted-foreground);
      background: transparent;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      cursor: pointer;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out);
    }
    :host([variant='ghost']) button {
      border-color: transparent;
    }
    button:hover {
      background: var(--muted);
      color: var(--foreground);
    }
    :host([pressed]) button {
      background: var(--muted);
      color: var(--foreground);
      border-color: var(--border);
    }
    :host([disabled]) button {
      cursor: not-allowed;
      opacity: 0.5;
      pointer-events: none;
    }
    button:focus-visible {
      ${focusRingStyle}
    }
    @media (forced-colors: active) {
      button:focus-visible {
        ${focusRingForcedColors}
      }
      :host([pressed]) button {
        outline: 2px solid CanvasText;
      }
    }
  `]

  static properties = {
    pressed: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    value: { type: String, reflect: true },
    size: { type: String, reflect: true },
    variant: { type: String, reflect: true },
  }

  pressed = false
  disabled = false
  value = ''
  size: 'sm' | 'md' | 'lg' = 'md'
  variant: 'outline' | 'ghost' = 'outline'

  private onClick = () => {
    if (this.disabled) return
    this.pressed = !this.pressed
    this.dispatchEvent(new CustomEvent('ship-toggle', {
      detail: { pressed: this.pressed, value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  /** Focus the button — used by parent group for keyboard nav. */
  focusButton() {
    const btn = this.renderRoot.querySelector('button') as HTMLButtonElement | null
    btn?.focus()
  }

  render() {
    return html`
      <button
        part="button"
        type="button"
        aria-pressed=${this.pressed ? 'true' : 'false'}
        ?disabled=${this.disabled}
        @click=${this.onClick}
      ><slot></slot></button>
    `
  }
}

export class ShipToggleGroup extends LitElement {
  static styles = [reducedMotion, css`
    :host {
      display: inline-flex;
      gap: 0;
    }
    :host([hidden]) { display: none; }
    /* Merge borders between children */
    ::slotted(ship-toggle:not(:first-child)) {
      margin-left: -1px;
    }
    ::slotted(ship-toggle:first-child) {
      border-radius: var(--radius) 0 0 var(--radius);
    }
    ::slotted(ship-toggle:last-child) {
      border-radius: 0 var(--radius) var(--radius) 0;
    }
    ::slotted(ship-toggle:not(:first-child):not(:last-child)) {
      border-radius: 0;
    }
  `]

  static properties = {
    type: { type: String, reflect: true },
    value: { type: String, reflect: true },
    size: { type: String, reflect: true },
    variant: { type: String, reflect: true },
  }

  type: 'single' | 'multiple' = 'single'
  value = ''
  size: 'sm' | 'md' | 'lg' = 'md'
  variant: 'outline' | 'ghost' = 'outline'

  private get toggles(): ShipToggle[] {
    return Array.from(this.querySelectorAll(':scope > ship-toggle')) as ShipToggle[]
  }

  private get enabledToggles(): ShipToggle[] {
    return this.toggles.filter(t => !t.disabled)
  }

  private getSelectedValues(): Set<string> {
    return new Set(this.value ? this.value.split(',').filter(Boolean) : [])
  }

  private setSelectedValues(vals: Set<string>) {
    this.value = Array.from(vals).join(',')
    this.syncToggles()
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  private onToggle = (e: CustomEvent) => {
    e.stopPropagation()
    const toggle = (e.target as HTMLElement).closest('ship-toggle') as ShipToggle | null
    if (!toggle) return
    const val = toggle.value
    const selected = this.getSelectedValues()

    if (this.type === 'single') {
      if (selected.has(val)) {
        this.setSelectedValues(new Set())
      } else {
        this.setSelectedValues(new Set([val]))
      }
    } else {
      if (selected.has(val)) {
        selected.delete(val)
      } else {
        selected.add(val)
      }
      this.setSelectedValues(selected)
    }
  }

  private onKeydown = (e: KeyboardEvent) => {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
    const items = this.enabledToggles
    if (!items.length) return
    const current = (e.composedPath().find(
      n => items.includes(n as ShipToggle),
    ) as ShipToggle) || null
    const i = current ? items.indexOf(current) : -1
    const next = e.key === 'ArrowRight'
      ? (i + 1) % items.length
      : (i - 1 + items.length) % items.length
    items[next].focusButton()
    e.preventDefault()
  }

  private syncToggles() {
    const selected = this.getSelectedValues()
    for (const toggle of this.toggles) {
      toggle.pressed = selected.has(toggle.value)
      if (this.size) toggle.size = this.size
      if (this.variant) toggle.variant = this.variant
    }
  }

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'group')
    this.addEventListener('ship-toggle', this.onToggle as EventListener)
    this.addEventListener('keydown', this.onKeydown)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('ship-toggle', this.onToggle as EventListener)
    this.removeEventListener('keydown', this.onKeydown)
  }

  updated() {
    this.syncToggles()
  }

  render() {
    return html`<slot @slotchange=${() => this.syncToggles()}></slot>`
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-toggle')) {
    customElements.define('ship-toggle', ShipToggle)
  }
  if (!customElements.get('ship-toggle-group')) {
    customElements.define('ship-toggle-group', ShipToggleGroup)
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-toggle': ShipToggle
    'ship-toggle-group': ShipToggleGroup
  }
}
