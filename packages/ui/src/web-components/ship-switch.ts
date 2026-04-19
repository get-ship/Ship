/**
 * <ship-switch>
 *
 * Toggle switch backed by a native `<input type="checkbox">` for form
 * participation and keyboard access. Visually a sliding pill that matches
 * shadcn/ui's Switch + Radix Themes' Switch: crisp pill track, thumb
 * with subtle shadow, tone-aware fill on check, stacked focus ring.
 *
 * Attributes:
 *   - checked: boolean — on state
 *   - disabled: boolean
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     Drives the on-state fill and focus ring.
 *   - size: sm | md | lg (default: md)
 *     sm=24×14, md=28×16, lg=32×18. Matches shadcn/Radix dimensions so
 *     rows of mixed controls align.
 *   - name, value — forwarded to the native input for form submission.
 *
 * Slots:
 *   - (default) — label content rendered next to the pill.
 *
 * Parts:
 *   - track — the pill.
 *   - thumb — the sliding pellet.
 *   - input — the native checkbox.
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { checked } }, bubbles, composed.
 */

import { LitElement, html } from 'lit'
import { shipSwitchStyles } from './ship-switch.styles'
import type { Size, Tone } from './util'

export class ShipSwitch extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultChecked = false

  constructor() {
    super()
    this.internals = this.attachInternals()
  }

  static styles = shipSwitchStyles

  static properties = {
    checked: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    name: { type: String },
    value: { type: String },
    label: { type: String },
  }

  checked = false
  disabled = false
  tone?: Tone
  size: Size = 'md'
  name = ''
  value = 'on'
  label = ''

  connectedCallback() {
    super.connectedCallback()
    this.defaultChecked = this.checked
  }

  updated() {
    this.setFormValue(this.checked ? this.value : null)
  }

  formResetCallback() {
    this.checked = this.defaultChecked
  }

  formStateRestoreCallback(state: string | null) {
    this.checked = state === this.value
  }

  private setFormValue(v: string | null) {
    if (typeof this.internals.setFormValue === 'function') {
      this.internals.setFormValue(v)
    }
  }

  private onChange = (e: Event) => {
    const c = (e.target as HTMLInputElement).checked
    this.checked = c
    this.setFormValue(c ? this.value : null)
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { checked: c },
      bubbles: true,
      composed: true,
    }))
  }

  render() {
    return html`
      <label>
        <span class="track" part="track">
          <input
            part="input"
            type="checkbox"
            role="switch"
            aria-checked=${this.checked ? 'true' : 'false'}
            aria-label=${this.label || this.name || 'toggle'}
            .checked=${this.checked}
            ?disabled=${this.disabled}
            name=${this.name}
            value=${this.value}
            @change=${this.onChange}
          />
          <span class="thumb" part="thumb"></span>
        </span>
        <slot></slot>
      </label>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-switch')) {
  customElements.define('ship-switch', ShipSwitch)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-switch': ShipSwitch
  }
}
