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

import { LitElement, css, html } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'

@customElement('ship-select')
export class ShipSelect extends LitElement {
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

  @property() value = ''
  @property() name = ''
  @property({ type: Boolean, reflect: true }) required = false
  @property({ type: Boolean, reflect: true }) disabled = false
  @property({ reflect: true }) size: 'sm' | 'md' = 'md'

  @query('select') private selectEl!: HTMLSelectElement

  firstUpdated() { this.syncOptions() }
  updated() { this.syncOptions() }

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
        @change=${this.onChange}
      ></select>
      <span hidden><slot @slotchange=${this.syncOptions}></slot></span>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-select': ShipSelect
  }
}
