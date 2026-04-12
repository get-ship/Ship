/**
 * <ship-checkbox>
 *
 * Wraps a native `<input type="checkbox">`. Label content goes in the
 * default slot.
 *
 * Attributes:
 *   - checked: boolean
 *   - name, value
 *   - disabled, required
 *   - indeterminate: boolean
 *
 * Parts:
 *   - input — native element
 *   - label — text wrapper
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { checked } }
 */

import { LitElement, css, html } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'

@customElement('ship-checkbox')
export class ShipCheckbox extends LitElement {
  static styles = css`
    :host { display: inline-flex; align-items: center; gap: 8px; cursor: pointer; font: inherit; font-size: 13px; color: var(--foreground); }
    :host([disabled]) { cursor: not-allowed; opacity: 0.6; }
    label { display: inline-flex; align-items: center; gap: 8px; cursor: inherit; }
    input { margin: 0; accent-color: var(--primary); width: 14px; height: 14px; }
    input:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
  `

  @property({ type: Boolean, reflect: true }) checked = false
  @property({ type: Boolean }) indeterminate = false
  @property() name = ''
  @property() value = 'on'
  @property({ type: Boolean, reflect: true }) disabled = false
  @property({ type: Boolean, reflect: true }) required = false

  @query('input') private input!: HTMLInputElement

  updated() {
    if (this.input) this.input.indeterminate = this.indeterminate
  }

  private onChange(e: Event) {
    const c = (e.target as HTMLInputElement).checked
    this.checked = c
    this.indeterminate = false
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { checked: c }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <label part="label">
        <input
          part="input"
          type="checkbox"
          .checked=${this.checked}
          name=${this.name}
          value=${this.value}
          ?disabled=${this.disabled}
          ?required=${this.required}
          @change=${this.onChange}
        />
        <slot></slot>
      </label>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-checkbox': ShipCheckbox
  }
}
