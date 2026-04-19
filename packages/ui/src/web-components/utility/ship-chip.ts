/**
 * <ship-chip>
 *
 * Interactive filter / selection pill. The counterpart to
 * <ship-badge> — where badge is a static inline label, chip is a Radix
 * Toggle: hoverable, focusable, optionally selectable, optionally
 * removable. Used across Ship as filter rails (events, skills, pipelines)
 * and as enum-cell tags in the data view.
 *
 * The host itself is the interactive surface — no inner <button> — so
 * Datastar's `data-on:click` on the host fires identically whether the
 * user hits the label, padding, or icon. Keyboard activation (Enter /
 * Space) dispatches a synthetic click on the host so existing templates
 * that listen for click bubbling keep working.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     (default: muted)
 *   - variant: solid | soft | outline (default: soft)
 *   - size: sm | md | lg (default: md; sm is the Ship rail convention)
 *   - selected: boolean — filter-pill "on" state; forces a solid tone
 *     fill with contrast text regardless of variant
 *   - removable: boolean — renders a × button that emits `ship-remove`
 *   - disabled: boolean — non-interactive, muted+desaturated
 *
 * Slots:
 *   - leading — icon / status dot / avatar before the label
 *   - (default) label text
 *
 * Parts:
 *   - chip — the outer pill element
 *   - leading — the leading-icon slot container
 *   - remove — the × button (only present when `removable`)
 *
 * Events:
 *   - ship-remove — fires when × is clicked or Backspace/Delete is
 *     pressed with the chip focused. `{ detail: {} }`, bubbles across
 *     the shadow boundary.
 */

import { LitElement, html, nothing } from 'lit'
import { shipChipStyles } from './ship-chip.styles'
import type { Size, Tone } from '../util'

type ChipVariant = 'solid' | 'soft' | 'outline'

export class ShipChip extends LitElement {
  static styles = shipChipStyles

  static properties = {
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    selected: { type: Boolean, reflect: true },
    removable: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
  }

  tone: Tone
  variant: ChipVariant | null
  size: Size
  selected: boolean
  removable: boolean
  disabled: boolean

  constructor() {
    super()
    this.tone = 'muted'
    this.variant = null
    this.size = 'md'
    this.selected = false
    this.removable = false
    this.disabled = false
  }

  connectedCallback() {
    super.connectedCallback()
    // The host owns interaction. Make it focusable unless the caller
    // explicitly opts out. An existing tabindex wins so authors can
    // still render chips as non-focusable decorations.
    if (!this.hasAttribute('tabindex') && !this.disabled) {
      this.setAttribute('tabindex', '0')
    }
    if (!this.hasAttribute('role')) {
      this.setAttribute('role', 'button')
    }
    this.addEventListener('keydown', this.onKeydown)
  }

  disconnectedCallback() {
    this.removeEventListener('keydown', this.onKeydown)
    super.disconnectedCallback()
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('disabled')) {
      if (this.disabled) {
        this.setAttribute('aria-disabled', 'true')
        this.setAttribute('tabindex', '-1')
      } else {
        this.removeAttribute('aria-disabled')
        if (this.getAttribute('tabindex') === '-1') {
          this.setAttribute('tabindex', '0')
        }
      }
    }
    if (changed.has('selected')) {
      this.setAttribute('aria-pressed', this.selected ? 'true' : 'false')
    }
  }

  private onKeydown = (e: KeyboardEvent) => {
    if (this.disabled) return
    // Enter / Space → synthesize a click on the host. Native click
    // bubbles, so Datastar `data-on:click` handlers fire identically
    // to a pointer click. We don't preventDefault on Enter so forms
    // still submit if this chip sits inside one.
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this.click()
      return
    }
    // Backspace / Delete → remove, if removable. Matches the Radix
    // Toggle / GitHub tag-input convention so keyboard users can
    // clear filters without mousing to the × glyph.
    if (this.removable && (e.key === 'Backspace' || e.key === 'Delete')) {
      e.preventDefault()
      this.dispatchRemove()
    }
  }

  private onRemoveClick = (e: MouseEvent) => {
    e.stopPropagation()
    if (this.disabled) return
    this.dispatchRemove()
  }

  private dispatchRemove() {
    this.dispatchEvent(new CustomEvent('ship-remove', {
      detail: {},
      bubbles: true,
      composed: true,
    }))
  }

  render() {
    return html`
      <span class="chip" part="chip">
        <span class="leading" part="leading"><slot name="leading"></slot></span>
        <slot></slot>
        ${this.removable
          ? html`
              <button
                class="remove"
                part="remove"
                type="button"
                aria-label="Remove"
                tabindex="-1"
                ?disabled=${this.disabled}
                @click=${this.onRemoveClick}
              >
                <svg viewBox="0 0 10 10" aria-hidden="true">
                  <path d="M2 2 L8 8 M8 2 L2 8" />
                </svg>
              </button>
            `
          : nothing}
      </span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-chip')) {
  customElements.define('ship-chip', ShipChip)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-chip': ShipChip
  }
}
