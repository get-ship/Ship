/**
 * <ship-input>
 *
 * Wraps a native `<input>`. Form-associated via the wrapped element; use
 * `part="input"` to style further.
 *
 * Attributes:
 *   - type: any native input type (default: text)
 *   - value, placeholder, name, required, disabled, readonly, autocomplete
 *   - variant: default | ghost (default: default) — `ghost` removes border
 *     until focus, matching specs .title-input
 *   - size: sm | md | lg (default: md)
 *   - invalid: boolean — reflects aria-invalid + destructive border
 *
 * Slots:
 *   - leading — icon/adornment before the input
 *   - trailing — icon/adornment after the input
 *
 * Parts:
 *   - field — wrapper
 *   - input — native element
 *
 * Events:
 *   - `ship-input` — CustomEvent { detail: { value } } fired on native input
 *   - (native `input` and `change` also bubble out composed)
 */

import { LitElement, css, html } from 'lit'
import type { Size, Variant } from './util'

export class ShipInput extends LitElement {
  static styles = css`
    :host { display: inline-flex; width: 100%; }
    .field {
      display: inline-flex; align-items: center; gap: 6px;
      width: 100%;
      background: var(--background);
      color: var(--foreground);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 0 10px;
      transition: border-color 120ms, box-shadow 120ms;
    }
    .field:focus-within {
      outline: 2px solid var(--primary);
      outline-offset: -1px;
      border-color: transparent;
    }
    :host([variant='ghost']) .field {
      background: transparent;
      border-color: transparent;
    }
    :host([variant='ghost']) .field:focus-within {
      background: var(--background);
      border-color: var(--border);
      outline: none;
    }
    :host([invalid]) .field { border-color: var(--destructive); }

    input {
      flex: 1;
      border: 0; outline: 0; background: transparent; color: inherit;
      font: inherit;
      padding: 0;
      min-width: 0;
    }
    input::placeholder { color: var(--muted-foreground); }
    input:disabled { cursor: not-allowed; opacity: 0.6; }

    :host([size='sm']) .field { padding: 2px 8px; }
    :host([size='sm']) input { font-size: 12px; line-height: 20px; }
    :host(:not([size])) .field,
    :host([size='md']) .field { padding: 4px 10px; }
    :host(:not([size])) input,
    :host([size='md']) input { font-size: 13px; line-height: 22px; }
    :host([size='lg']) .field { padding: 6px 12px; }
    :host([size='lg']) input { font-size: 16px; line-height: 28px; font-weight: 600; }
  `

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
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
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
  variant: Variant | 'default' = 'default'
  size: Size = 'md'
  private onInput(e: Event) {
    const target = e.target as HTMLInputElement
    this.value = target.value
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
          @input=${this.onInput}
        />
        <slot name="trailing"></slot>
      </div>
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
