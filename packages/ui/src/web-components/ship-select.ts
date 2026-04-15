/**
 * <ship-select>
 *
 * Wraps a native `<select>`. Options are declared as light-DOM `<option>`
 * children and projected into the shadow select via a slot-driven sync.
 *
 * Attributes:
 *   - value
 *   - name
 *   - required, disabled
 *   - size: sm | md (default: md)
 *
 * Slots:
 *   - (default) native <option> / <optgroup> children
 *
 * Parts:
 *   - select — wrapped native element
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { value } }
 */

import { LitElement, css, html, nothing } from 'lit'
export class ShipSelect extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultValue = ''
  constructor() {
    super()
    this.internals = this.attachInternals()
  }
  static styles = css`
    :host { display: inline-flex; }
    select {
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 5px 8px;
      font: inherit; font-size: 12px;
      background: var(--background);
      color: var(--foreground);
      cursor: pointer;
    }
    select:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }
    select:disabled { cursor: not-allowed; opacity: 0.6; }
    :host([size='sm']) select { font-size: 11px; padding: 3px 6px; }
  `

  static properties = {
    value: { type: String },
    name: { type: String },
    required: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    invalid: { type: Boolean, reflect: true },
    size: { type: String, reflect: true },
    label: { type: String },
    describedBy: { type: String, attribute: 'described-by' },
  }

  value = ''
  name = ''
  required = false
  disabled = false
  invalid = false
  size: 'sm' | 'md' = 'md'
  label = ''
  describedBy = ''
  get selectEl(): HTMLSelectElement { return this.renderRoot.querySelector('select')! as HTMLSelectElement }

  connectedCallback() {
    super.connectedCallback()
    this.defaultValue = this.value
  }
  firstUpdated() { this.syncOptions() }
  updated() {
    this.syncOptions()
    this.setFormValue(this.value)
  }
  formResetCallback() { this.value = this.defaultValue }
  formStateRestoreCallback(state: string | null) { this.value = state ?? '' }
  private setFormValue(v: string) {
    if (typeof this.internals.setFormValue === 'function') this.internals.setFormValue(v)
  }

  private syncOptions() {
    if (!this.selectEl) return
    const opts = Array.from(this.querySelectorAll(':scope > option, :scope > optgroup'))
    const fragment = document.createDocumentFragment()
    for (const o of opts) fragment.appendChild(o.cloneNode(true))
    this.selectEl.innerHTML = ''
    this.selectEl.appendChild(fragment)
    this.selectEl.value = this.value
  }

  private onChange(e: Event) {
    this.value = (e.target as HTMLSelectElement).value
    this.setFormValue(this.value)
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: this.value }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <select part="select"
        name=${this.name}
        ?required=${this.required}
        ?disabled=${this.disabled}
        aria-invalid=${this.invalid ? 'true' : 'false'}
        aria-label=${this.label || nothing}
        aria-describedby=${this.describedBy || nothing}
        @change=${this.onChange}
      ></select>
      <span hidden><slot @slotchange=${this.syncOptions}></slot></span>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-select')) {
  customElements.define('ship-select', ShipSelect)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-select': ShipSelect
  }
}
