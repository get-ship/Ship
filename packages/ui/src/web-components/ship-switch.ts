/**
 * <ship-switch>
 *
 * Toggle switch backed by a native `<input type="checkbox">` for form
 * participation and keyboard access. Visually styled as a sliding pill.
 *
 * Attributes:
 *   - checked: boolean
 *   - disabled
 *   - name, value
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { checked } }
 */

import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

@customElement('ship-switch')
export class ShipSwitch extends LitElement {
  static styles = css`
    :host { display: inline-flex; align-items: center; gap: 8px; font-size: 13px; color: var(--foreground); }
    :host([disabled]) { opacity: 0.6; cursor: not-allowed; }
    label { display: inline-flex; align-items: center; gap: 8px; cursor: inherit; }
    .track {
      position: relative;
      display: inline-block;
      width: 30px; height: 18px;
      background: var(--muted);
      border: 1px solid var(--border);
      border-radius: 999px;
      transition: background 120ms, border-color 120ms;
    }
    .thumb {
      position: absolute;
      top: 1px; left: 1px;
      width: 14px; height: 14px;
      background: var(--background);
      border-radius: 50%;
      box-shadow: 0 1px 2px oklch(0.147 0.004 49.25 / 25%);
      transition: transform 140ms ease;
    }
    :host([checked]) .track {
      background: var(--primary);
      border-color: var(--primary);
    }
    :host([checked]) .thumb { transform: translateX(12px); }
    input {
      position: absolute; opacity: 0; width: 100%; height: 100%;
      margin: 0; cursor: inherit;
      inset: 0;
    }
    input:focus-visible + .thumb { outline: 2px solid var(--primary); outline-offset: 2px; }
  `

  @property({ type: Boolean, reflect: true }) checked = false
  @property({ type: Boolean, reflect: true }) disabled = false
  @property() name = ''
  @property() value = 'on'

  private onChange(e: Event) {
    const c = (e.target as HTMLInputElement).checked
    this.checked = c
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { checked: c }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <label>
        <span class="track" part="track">
          <input
            part="input"
            type="checkbox"
            role="switch"
            aria-checked=${this.checked ? 'true' : 'false'}
            .checked=${this.checked}
            ?disabled=${this.disabled}
            name=${this.name}
            value=${this.value}
            @change=${this.onChange}
          />
          <span class="thumb" part="thumb"></span>
        </span>
        <slot></slot>
      </label>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-switch': ShipSwitch
  }
}
