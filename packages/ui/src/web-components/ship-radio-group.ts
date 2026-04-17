/**
 * <ship-radio-group> + <ship-radio>
 *
 * Radio group matching Radix Radio Group behavior. Group manages
 * selection; radios are composed as children. Arrow keys navigate
 * within the group.
 *
 * Attributes on <ship-radio-group>:
 *   - value: string — currently selected radio value
 *   - disabled: boolean — disables all radios
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - size: sm | md | lg
 *   - name: string — form field name
 *   - label: string — aria-label for the group
 *
 * Attributes on <ship-radio>:
 *   - value: string (required)
 *   - disabled: boolean
 *   - checked: boolean (reflect, managed by group)
 *   - tone: optional override (inherits group tone)
 *   - size: optional override (inherits group size)
 *
 * Slots on <ship-radio>:
 *   - (default) — label content
 *
 * Parts on <ship-radio>:
 *   - circle — the outer ring
 *   - dot — the inner filled dot
 *   - input — the native radio input
 *
 * Events on <ship-radio-group>:
 *   - ship-change — CustomEvent { detail: { value } }, bubbles, composed.
 */

import { LitElement, html } from 'lit'
import {
  shipRadioGroupStyles,
  shipRadioStyles,
} from './ship-radio-group.styles'
import type { Size, Tone } from './util'

export class ShipRadio extends LitElement {
  static styles = shipRadioStyles

  static properties = {
    value: { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
    checked: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
  }

  value = ''
  disabled = false
  checked = false
  tone?: Tone
  size?: Size

  /** Focus the native input from the group's keyboard handler. */
  focusInput() {
    const input = this.renderRoot.querySelector('input') as HTMLInputElement | null
    input?.focus()
  }

  private onClick = () => {
    if (this.disabled) return
    this.dispatchEvent(new CustomEvent('ship-radio-select', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  private onKeydown = (e: KeyboardEvent) => {
    const nav = ['ArrowDown', 'ArrowRight', 'ArrowUp', 'ArrowLeft']
    if (nav.includes(e.key)) {
      e.preventDefault()
      const forward = e.key === 'ArrowDown' || e.key === 'ArrowRight'
      this.dispatchEvent(new CustomEvent('ship-radio-nav', {
        detail: { direction: forward ? 'next' : 'prev' },
        bubbles: true,
        composed: true,
      }))
    }
  }

  render() {
    return html`
      <label>
        <span class="circle" part="circle">
          <input
            part="input"
            type="radio"
            role="radio"
            .checked=${this.checked}
            ?disabled=${this.disabled}
            aria-checked=${this.checked ? 'true' : 'false'}
            aria-label=${this.value}
            tabindex=${this.checked ? '0' : '-1'}
            @click=${this.onClick}
            @keydown=${this.onKeydown}
          />
          <span class="dot" part="dot" aria-hidden="true"></span>
        </span>
        <slot></slot>
      </label>
    `
  }
}

export class ShipRadioGroup extends LitElement {
  static styles = shipRadioGroupStyles

  static properties = {
    value: { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    name: { type: String },
    label: { type: String },
  }

  value = ''
  disabled = false
  tone?: Tone
  size?: Size
  name = ''
  label = ''

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'radiogroup')
    if (this.label) this.setAttribute('aria-label', this.label)
    this.addEventListener('ship-radio-select', this.onRadioSelect as EventListener)
    this.addEventListener('ship-radio-nav', this.onRadioNav as EventListener)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('ship-radio-select', this.onRadioSelect as EventListener)
    this.removeEventListener('ship-radio-nav', this.onRadioNav as EventListener)
  }

  private get radios(): ShipRadio[] {
    return Array.from(this.querySelectorAll(':scope > ship-radio')) as ShipRadio[]
  }

  private get enabledRadios(): ShipRadio[] {
    return this.radios.filter(r => !r.disabled && !this.disabled)
  }

  updated() {
    this.syncRadios()
  }

  private syncRadios() {
    for (const radio of this.radios) {
      radio.checked = radio.value === this.value
      // Inherit group-level tone/size if the radio doesn't override
      if (this.tone && !radio.hasAttribute('tone')) {
        radio.tone = this.tone
      }
      if (this.size && !radio.hasAttribute('size')) {
        radio.size = this.size
      }
      if (this.disabled) {
        radio.disabled = true
      }
    }
  }

  private onRadioSelect = (e: CustomEvent) => {
    e.stopPropagation()
    const newValue = e.detail.value as string
    if (newValue === this.value) return
    this.value = newValue
    this.syncRadios()
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  private onRadioNav = (e: CustomEvent) => {
    e.stopPropagation()
    const enabled = this.enabledRadios
    if (!enabled.length) return

    const current = (e.target as HTMLElement).closest('ship-radio') as ShipRadio | null
    const idx = current ? enabled.indexOf(current) : -1
    let next: ShipRadio

    if (e.detail.direction === 'next') {
      next = enabled[(idx + 1) % enabled.length]
    } else {
      next = enabled[(idx - 1 + enabled.length) % enabled.length]
    }

    // Select + focus the target radio
    this.value = next.value
    this.syncRadios()
    next.focusInput()
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  render() {
    return html`<slot></slot>`
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-radio')) {
    customElements.define('ship-radio', ShipRadio)
  }
  if (!customElements.get('ship-radio-group')) {
    customElements.define('ship-radio-group', ShipRadioGroup)
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-radio': ShipRadio
    'ship-radio-group': ShipRadioGroup
  }
}
