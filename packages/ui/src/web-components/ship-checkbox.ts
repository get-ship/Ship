/**
 * <ship-checkbox>
 *
 * Wraps a native `<input type="checkbox">` for form participation and
 * keyboard access. Visually a rounded square that matches shadcn/ui's
 * Checkbox + Radix Themes' Checkbox: tone-aware fill on check, white
 * glyph, stacked focus ring, border hover, indeterminate as a bar.
 *
 * Attributes:
 *   - checked: boolean
 *   - indeterminate: boolean — shows a horizontal bar glyph
 *   - disabled: boolean
 *   - required: boolean
 *   - invalid: boolean — destructive border + focus ring override
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     Drives the checked/indeterminate fill and focus ring.
 *   - size: sm | md | lg (default: md) — 14/16/18px square
 *   - name, value — forwarded to the native input for form submission.
 *
 * Slots:
 *   - (default) — label content rendered next to the box.
 *
 * Parts:
 *   - input — the native element.
 *   - box — the square surface.
 *   - label — the text wrapper.
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { checked } }, bubbles, composed.
 */

import { LitElement, html } from 'lit'
import { shipCheckboxStyles } from './ship-checkbox.styles'
import type { Size, Tone } from './util'

export class ShipCheckbox extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultChecked = false

  constructor() {
    super()
    this.internals = this.attachInternals()
  }

  static styles = shipCheckboxStyles

  static properties = {
    checked: { type: Boolean, reflect: true },
    indeterminate: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    required: { type: Boolean, reflect: true },
    invalid: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    name: { type: String },
    value: { type: String },
    label: { type: String },
  }

  checked = false
  indeterminate = false
  disabled = false
  required = false
  invalid = false
  tone?: Tone
  size: Size = 'md'
  name = ''
  value = 'on'
  label = ''

  get input(): HTMLInputElement {
    return this.renderRoot.querySelector('input') as HTMLInputElement
  }

  connectedCallback() {
    super.connectedCallback()
    this.defaultChecked = this.checked
  }

  updated() {
    // Keep native indeterminate in sync — it's not reflected via attrs.
    if (this.input) this.input.indeterminate = this.indeterminate
    this.setFormValue(this.checked ? this.value : null)
  }

  formResetCallback() {
    this.checked = this.defaultChecked
    this.indeterminate = false
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
    this.indeterminate = false
    this.setFormValue(c ? this.value : null)
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { checked: c },
      bubbles: true,
      composed: true,
    }))
  }

  render() {
    const ariaChecked = this.indeterminate
      ? 'mixed'
      : (this.checked ? 'true' : 'false')
    return html`
      <label part="label">
        <span class="box" part="box">
          <input
            part="input"
            type="checkbox"
            .checked=${this.checked}
            ?disabled=${this.disabled}
            ?required=${this.required}
            name=${this.name}
            value=${this.value}
            aria-checked=${ariaChecked}
            aria-invalid=${this.invalid ? 'true' : 'false'}
            aria-label=${this.label || this.name || 'checkbox'}
            @change=${this.onChange}
          />
          <span class="glyph check" aria-hidden="true">
            <svg viewBox="0 0 16 16" fill="none">
              <path d="M3.5 8.5 L6.75 11.75 L12.5 4.75"></path>
            </svg>
          </span>
          <span class="glyph mixed" aria-hidden="true">
            <svg viewBox="0 0 16 16" fill="none">
              <line x1="4" y1="8" x2="12" y2="8"></line>
            </svg>
          </span>
        </span>
        <slot></slot>
      </label>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-checkbox')) {
  customElements.define('ship-checkbox', ShipCheckbox)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-checkbox': ShipCheckbox
  }
}
