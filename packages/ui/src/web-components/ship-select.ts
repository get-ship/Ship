/**
 * <ship-select>
 *
 * Wraps a native `<select>`. Options are declared as light-DOM `<option>`
 * children and projected into the shadow select via a slot-driven sync.
 *
 * The native popup is OS-chrome (we accept that) — but the closed control
 * is skinned to shadcn/ui-equivalent polish: custom chevron, hover surface,
 * stacked focus ring, tone-aware border, size channel, disabled fade,
 * placeholder dimming, and motion that respects `prefers-reduced-motion`.
 *
 * Attributes:
 *   - value
 *   - name
 *   - required, disabled, invalid
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - size: sm | md | lg (default: md)
 *   - label, described-by
 *
 * Slots:
 *   - (default) native <option> / <optgroup> children
 *
 * Parts:
 *   - field — the chrome wrapper (holds chevron + native select)
 *   - select — the wrapped native element
 *   - chevron — the custom dropdown icon
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { value } }
 */

import { LitElement, html, nothing } from 'lit'
import { shipSelectStyles } from './ship-select.styles'
import type { Size, Tone } from './util'

export class ShipSelect extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultValue = ''
  constructor() {
    super()
    this.internals = this.attachInternals()
  }
  static styles = shipSelectStyles

  static properties = {
    value: { type: String },
    name: { type: String },
    required: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    invalid: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    label: { type: String },
    describedBy: { type: String, attribute: 'described-by' },
  }

  value = ''
  name = ''
  required = false
  disabled = false
  invalid = false
  tone?: Tone
  size: Size = 'md'
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
    this.updatePlaceholderState()
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

  /** Pick the label for the current value. We always read from light-DOM
   * options (the source of truth) so the label is available on every render,
   * including the first one before `syncOptions()` has cloned them into
   * shadow. Falls back to the raw value so the chrome never renders empty. */
  private currentLabel(): string {
    const lightOpts = Array.from(
      this.querySelectorAll(':scope > option, :scope > optgroup > option')
    ) as HTMLOptionElement[]
    if (lightOpts.length === 0) return this.value
    const match =
      lightOpts.find((o) => o.value === this.value) ??
      lightOpts.find((o) => o.hasAttribute('selected')) ??
      lightOpts[0]
    const text = (match?.textContent || '').trim()
    return text || match?.value || this.value
  }

  /** Mirror `data-placeholder` on the host when the current option has an
   * empty value — lets the visible label dim to `--muted-foreground`. */
  private updatePlaceholderState() {
    const sel = this.selectEl
    if (!sel) {
      // Pre-sync: look at light-DOM.
      const lightOpts = Array.from(
        this.querySelectorAll(':scope > option, :scope > optgroup > option')
      ) as HTMLOptionElement[]
      const match =
        lightOpts.find((o) => o.value === this.value) ??
        lightOpts.find((o) => o.hasAttribute('selected')) ??
        lightOpts[0]
      if (!match || match.value === '') this.setAttribute('data-placeholder', '')
      else this.removeAttribute('data-placeholder')
      return
    }
    const opt = sel.selectedOptions?.[0] ?? sel.options[sel.selectedIndex]
    const isPlaceholder = !opt || opt.value === ''
    if (isPlaceholder) this.setAttribute('data-placeholder', '')
    else this.removeAttribute('data-placeholder')
  }

  private onChange(e: Event) {
    this.value = (e.target as HTMLSelectElement).value
    this.setFormValue(this.value)
    this.updatePlaceholderState()
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: this.value }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <div class="field" part="field">
        <span class="value" aria-hidden="true">${this.currentLabel()}</span>
        <svg class="chevron" part="chevron" viewBox="0 0 16 16"
          aria-hidden="true" focusable="false" fill="none"
          stroke="currentColor" stroke-width="1.75"
          stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 6l4 4 4-4"></path>
        </svg>
        <select part="select"
          name=${this.name}
          ?required=${this.required}
          ?disabled=${this.disabled}
          aria-invalid=${this.invalid ? 'true' : 'false'}
          aria-label=${this.label || nothing}
          aria-describedby=${this.describedBy || nothing}
          @change=${this.onChange}
        ></select>
      </div>
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
