/**
 * <ship-input>
 *
 * Wraps a native `<input>`. Form-associated via the wrapped element; use
 * `part="input"` to style further.
 *
 * Attributes:
 *   - type: any native input type (default: text)
 *   - value, placeholder, name, required, disabled, readonly, autocomplete
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     (tints the focus ring; `invalid` overrides to destructive)
 *   - variant: default | ghost (default: default) — `ghost` removes border
 *     until focus, matching specs .title-input
 *   - size: sm | md | lg (default: md)
 *   - invalid: boolean — reflects aria-invalid + destructive border + ring
 *   - loading: boolean — shows an inline spinner in the trailing slot
 *
 * Slots:
 *   - leading — icon/adornment before the input
 *   - trailing — icon/adornment after the input (replaced by spinner when loading)
 *
 * Parts:
 *   - field — wrapper
 *   - input — native element
 *
 * Events:
 *   - `ship-input` — CustomEvent { detail: { value } } fired on native input
 *   - (native `input` and `change` also bubble out composed)
 */

import { LitElement, html, nothing } from 'lit'
import { shipInputStyles } from './ship-input.styles'
import type { Size, Tone, Variant } from './util'

export class ShipInput extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultValue = ''
  constructor() {
    super()
    this.internals = this.attachInternals()
  }
  static styles = shipInputStyles

  static properties = {
    type: { type: String },
    value: { type: String },
    placeholder: { type: String },
    name: { type: String },
    autocomplete: { type: String },
    required: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    readonly: { type: Boolean, reflect: true },
    invalid: { type: Boolean, reflect: true },
    loading: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    label: { type: String },
    describedBy: { type: String, attribute: 'described-by' },
    errorMessage: { type: String, attribute: 'error-message' },
  }

  type = 'text'
  value = ''
  placeholder = ''
  name = ''
  autocomplete = ''
  required = false
  disabled = false
  readonly = false
  invalid = false
  loading = false
  tone?: Tone
  variant: Variant | 'default' = 'default'
  size: Size = 'md'
  label = ''
  describedBy = ''
  errorMessage = ''
  private static _idSeq = 0
  private errId = `ship-input-err-${++ShipInput._idSeq}`

  connectedCallback() {
    super.connectedCallback()
    this.defaultValue = this.value
  }
  updated() { this.setFormValue(this.value) }
  formResetCallback() { this.value = this.defaultValue }
  formStateRestoreCallback(state: string | null) { this.value = state ?? '' }
  private setFormValue(v: string) {
    if (typeof this.internals.setFormValue === 'function') this.internals.setFormValue(v)
  }

  private onInput(e: Event) {
    const target = e.target as HTMLInputElement
    this.value = target.value
    this.setFormValue(this.value)
    this.dispatchEvent(new CustomEvent('ship-input', {
      detail: { value: this.value }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <div class="field" part="field">
        <slot name="leading"></slot>
        <input
          part="input"
          .value=${this.value}
          type=${this.type}
          placeholder=${this.placeholder}
          name=${this.name}
          autocomplete=${this.autocomplete || 'off'}
          ?required=${this.required}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          aria-invalid=${this.invalid ? 'true' : 'false'}
          aria-required=${this.required ? 'true' : 'false'}
          aria-busy=${this.loading ? 'true' : 'false'}
          aria-label=${this.label || nothing}
          aria-describedby=${this.invalid && this.errorMessage
            ? `${this.errId}${this.describedBy ? ' ' + this.describedBy : ''}`
            : (this.describedBy || nothing)}
          @input=${this.onInput}
        />
        ${this.loading
          ? html`<span class="spinner" aria-hidden="true"></span>`
          : html`<slot name="trailing"></slot>`}
      </div>
      ${this.invalid && this.errorMessage
        ? html`<span id=${this.errId} role="alert" style="display:none">${this.errorMessage}</span>`
        : nothing}
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-input')) {
  customElements.define('ship-input', ShipInput)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-input': ShipInput
  }
}
