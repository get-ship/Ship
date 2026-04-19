/**
 * <ship-slider>
 *
 * Horizontal range slider matching Radix Slider behavior. Track with
 * filled portion and draggable thumb. Supports keyboard, pointer drag,
 * and click-on-track interaction.
 *
 * Attributes:
 *   - value: number (default 0)
 *   - min: number (default 0)
 *   - max: number (default 100)
 *   - step: number (default 1)
 *   - disabled: boolean
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - size: sm | md | lg (default md)
 *   - label: string — aria-label for the thumb
 *
 * Parts:
 *   - track — the background rail
 *   - fill — the filled portion
 *   - thumb — the draggable handle
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { value } }, bubbles, composed.
 */

import { LitElement, html } from 'lit'
import { shipSliderStyles } from './ship-slider.styles'
import type { Size, Tone } from './util'

export class ShipSlider extends LitElement {
  static styles = shipSliderStyles

  static properties = {
    value: { type: Number, reflect: true },
    min: { type: Number },
    max: { type: Number },
    step: { type: Number },
    disabled: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    label: { type: String },
  }

  value = 0
  min = 0
  max = 100
  step = 1
  disabled = false
  tone?: Tone
  size: Size = 'md'
  label = ''

  private dragging = false

  private get pct(): number {
    const range = this.max - this.min
    if (range <= 0) return 0
    return ((this.value - this.min) / range) * 100
  }

  private clamp(v: number): number {
    const stepped = Math.round((v - this.min) / this.step) * this.step + this.min
    return Math.max(this.min, Math.min(this.max, stepped))
  }

  private setValue(v: number) {
    const clamped = this.clamp(v)
    if (clamped === this.value) return
    this.value = clamped
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value: this.value },
      bubbles: true,
      composed: true,
    }))
  }

  private valueFromPointer(clientX: number): number {
    const track = this.renderRoot.querySelector('.track') as HTMLElement | null
    if (!track) return this.value
    const rect = track.getBoundingClientRect()
    const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
    return this.min + pct * (this.max - this.min)
  }

  private onPointerDown = (e: PointerEvent) => {
    if (this.disabled) return
    e.preventDefault()
    this.dragging = true
    this.setValue(this.valueFromPointer(e.clientX))
    const thumb = this.renderRoot.querySelector('.thumb') as HTMLElement
    thumb?.focus()

    const onMove = (ev: PointerEvent) => {
      if (!this.dragging) return
      this.setValue(this.valueFromPointer(ev.clientX))
    }
    const onUp = () => {
      this.dragging = false
      document.removeEventListener('pointermove', onMove)
      document.removeEventListener('pointerup', onUp)
    }
    document.addEventListener('pointermove', onMove)
    document.addEventListener('pointerup', onUp)
  }

  private onKeydown = (e: KeyboardEvent) => {
    if (this.disabled) return
    let handled = true
    switch (e.key) {
      case 'ArrowRight':
      case 'ArrowUp':
        this.setValue(this.value + this.step)
        break
      case 'ArrowLeft':
      case 'ArrowDown':
        this.setValue(this.value - this.step)
        break
      case 'Home':
        this.setValue(this.min)
        break
      case 'End':
        this.setValue(this.max)
        break
      case 'PageUp':
        this.setValue(this.value + this.step * 10)
        break
      case 'PageDown':
        this.setValue(this.value - this.step * 10)
        break
      default:
        handled = false
    }
    if (handled) e.preventDefault()
  }

  render() {
    const pct = this.pct
    return html`
      <div class="container" @pointerdown=${this.onPointerDown}>
        <div class="track" part="track">
          <div class="fill" part="fill" style="width: ${pct}%"></div>
        </div>
        <div
          class="thumb"
          part="thumb"
          role="slider"
          tabindex=${this.disabled ? '-1' : '0'}
          aria-valuemin=${this.min}
          aria-valuemax=${this.max}
          aria-valuenow=${this.value}
          aria-label=${this.label || 'slider'}
          aria-disabled=${this.disabled ? 'true' : 'false'}
          style="left: ${pct}%"
          @keydown=${this.onKeydown}
        ></div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-slider')) {
  customElements.define('ship-slider', ShipSlider)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-slider': ShipSlider
  }
}
