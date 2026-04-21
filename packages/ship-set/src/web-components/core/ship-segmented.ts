/**
 * <ship-segmented> + <ship-segmented-option>
 *
 * Generic segmented control backed by a signal value. Mirrors iOS
 * UISegmentedControl / Linear segmented button groups. Replaces
 * view-local col-tabs / col-density segmented patterns.
 *
 * Attributes on <ship-segmented>:
 *   - value (string, reflected) — selected option value
 *   - size  — sm | md (default: md)
 *   - disabled — boolean, blocks all interaction
 *
 * Attributes on <ship-segmented-option>:
 *   - value (required) — option value
 *   - icon  — ship-icon name (optional, renders before label)
 *   - disabled — boolean, blocks this option
 *
 * Slots on <ship-segmented-option>:
 *   - (default) — label text
 *   - leading   — override icon slot content
 *
 * Events on <ship-segmented>:
 *   - ship-change — CustomEvent { detail: { value: string } }, bubbles, composed
 *
 * Accessibility:
 *   - role="radiogroup" on host
 *   - role="radio" + aria-checked on each option button
 *   - Arrow key navigation, Enter/Space activation
 */

import { LitElement, css, html } from 'lit'
import type { Size } from './util'

export class ShipSegmentedOption extends LitElement {
  static styles = css`:host { display: none; }`
  static properties = {
    value: { type: String, reflect: true },
    icon:  { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
  }
  value = ''
  icon = ''
  disabled = false
  render() { return html`` }
}

export class ShipSegmented extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
    }

    .container {
      display: inline-flex;
      padding: 2px;
      background: var(--muted);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      gap: 1px;
    }

    :host([size='sm']) .container { padding: 2px; }

    button {
      appearance: none;
      border: 0;
      background: transparent;
      border-radius: var(--radius-sm);
      font-family: var(--font-body);
      color: var(--muted-foreground);
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: var(--space-1);
      transition:
        background var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
      white-space: nowrap;
      line-height: 1;
    }

    :host(:not([size='sm'])) button {
      padding: 5px var(--space-3);
      font-size: var(--text-sm);
    }
    :host([size='sm']) button {
      padding: 4px var(--space-2);
      font-size: var(--text-xs);
    }

    button[aria-checked='true'] {
      background: var(--card);
      color: var(--foreground);
      box-shadow: var(--shadow-xs);
    }

    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 2px var(--background),
        0 0 0 4px color-mix(in oklch, var(--primary) 60%, transparent);
    }

    button[disabled],
    :host([disabled]) button {
      opacity: 0.5;
      cursor: not-allowed;
      pointer-events: none;
    }

    .btn-icon {
      display: inline-flex;
      align-items: center;
      width: 14px;
      height: 14px;
    }
    :host(:not([size='sm'])) .btn-icon {
      width: 16px;
      height: 16px;
    }
  `

  static properties = {
    value:    { type: String, reflect: true },
    size:     { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
  }

  value = ''
  size: Size = 'md'
  disabled = false

  private _mutationObserver?: MutationObserver

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'radiogroup')
    this._mutationObserver = new MutationObserver(() => this.requestUpdate())
    this._mutationObserver.observe(this, { childList: true, subtree: true, attributes: true })
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this._mutationObserver?.disconnect()
  }

  private get options(): ShipSegmentedOption[] {
    return Array.from(this.querySelectorAll(':scope > ship-segmented-option')) as ShipSegmentedOption[]
  }

  private select(value: string) {
    if (this.disabled) return
    if (this.value === value) return
    this.value = value
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { value },
      bubbles: true,
      composed: true,
    }))
  }

  private onKeyDown(e: KeyboardEvent, options: ShipSegmentedOption[]) {
    const enabled = options.filter(o => !o.disabled)
    if (!enabled.length) return
    const values = enabled.map(o => o.value)
    const i = Math.max(0, values.indexOf(this.value))
    let next = i
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault()
      next = (i + 1) % values.length
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault()
      next = (i - 1 + values.length) % values.length
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      // activate whatever button currently has focus
      const focused = this.renderRoot.activeElement as HTMLButtonElement | null
      if (focused?.dataset.value) this.select(focused.dataset.value)
      return
    } else {
      return
    }
    const targetValue = values[next]
    this.select(targetValue)
    queueMicrotask(() => {
      const btn = this.renderRoot.querySelector<HTMLButtonElement>(
        `button[data-value="${CSS.escape(targetValue)}"]`,
      )
      btn?.focus()
    })
  }

  render() {
    const options = this.options
    return html`
      <div class="container" part="container">
        ${options.map(opt => html`
          <button
            role="radio"
            data-value=${opt.value}
            aria-checked=${opt.value === this.value ? 'true' : 'false'}
            aria-disabled=${(this.disabled || opt.disabled) ? 'true' : 'false'}
            ?disabled=${this.disabled || opt.disabled}
            tabindex=${opt.value === this.value ? '0' : '-1'}
            @click=${() => this.select(opt.value)}
            @keydown=${(e: KeyboardEvent) => this.onKeyDown(e, options)}
          >
            ${opt.icon ? html`
              <span class="btn-icon" aria-hidden="true">
                <ship-icon name=${opt.icon} size="sm"></ship-icon>
              </span>
            ` : null}
            <slot name=${'opt-' + opt.value}>${opt.textContent?.trim()}</slot>
          </button>
        `)}
      </div>
    `
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-segmented-option'))
    customElements.define('ship-segmented-option', ShipSegmentedOption)
  if (!customElements.get('ship-segmented'))
    customElements.define('ship-segmented', ShipSegmented)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-segmented': ShipSegmented
    'ship-segmented-option': ShipSegmentedOption
  }
}
