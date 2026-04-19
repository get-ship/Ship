/**
 * <ship-textarea>
 *
 * Wraps a native `<textarea>`. Use `variant="code"` for monospace; resize
 * controlled via `resize` attribute.
 *
 * Attributes:
 *   - value, placeholder, name, required, disabled, readonly
 *   - rows: number (default 4)
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     (tints the focus ring; `invalid` overrides to destructive)
 *   - variant: default | code (default: default)
 *   - size: sm | md (default: md)
 *   - invalid: boolean — reflects aria-invalid + destructive border + ring
 *   - resize: none | vertical | horizontal | both (default: vertical)
 *
 * Parts:
 *   - textarea — the wrapped native element
 *
 * Events:
 *   - ship-input — CustomEvent { detail: { value } }
 */

import { LitElement, html, nothing } from 'lit'
import { shipTextareaStyles } from './ship-textarea.styles'
import type { Size, Tone } from './util'

export class ShipTextarea extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultValue = ''
  constructor() {
    super()
    this.internals = this.attachInternals()
  }
  static styles = shipTextareaStyles

  static properties = {
    value: { type: String },
    placeholder: { type: String },
    name: { type: String },
    rows: { type: Number },
    required: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    readonly: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    resize: { type: String },
    invalid: { type: Boolean, reflect: true },
    label: { type: String },
    describedBy: { type: String, attribute: 'described-by' },
  }

  value = ''
  placeholder = ''
  name = ''
  rows = 4
  required = false
  disabled = false
  readonly = false
  tone?: Tone
  variant: 'default' | 'code' = 'default'
  size: Size = 'md'
  resize: 'none' | 'vertical' | 'horizontal' | 'both' = 'vertical'
  invalid = false
  label = ''
  describedBy = ''
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
    this.value = (e.target as HTMLTextAreaElement).value
    this.setFormValue(this.value)
    this.dispatchEvent(new CustomEvent('ship-input', {
      detail: { value: this.value }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <textarea
        part="textarea"
        .value=${this.value}
        placeholder=${this.placeholder}
        name=${this.name}
        rows=${this.rows}
        ?required=${this.required}
        ?disabled=${this.disabled}
        ?readonly=${this.readonly}
        aria-invalid=${this.invalid ? 'true' : 'false'}
        aria-required=${this.required ? 'true' : 'false'}
        aria-label=${this.label || nothing}
        aria-describedby=${this.describedBy || nothing}
        style="resize: ${this.resize}"
        @input=${this.onInput}
      ></textarea>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-textarea')) {
  customElements.define('ship-textarea', ShipTextarea)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-textarea': ShipTextarea
  }
}
