/**
 * <ship-button>
 *
 * Wraps a native `<button>`. Keyboard, focus, and form participation
 * come for free from the native element.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - variant: solid | soft | outline | ghost (default: solid)
 *   - size: sm | md | lg (default: md)
 *   - disabled: boolean
 *   - loading: boolean — shows a spinner and disables the button
 *   - type: button | submit | reset (default: button)
 *
 * Slots:
 *   - (default) button label
 *   - leading — icon/adornment before the label
 *   - trailing — icon/adornment after the label
 *
 * Parts:
 *   - button — the wrapped native element
 *
 * Events:
 *   - (native `click` bubbles out of the host)
 */

import { LitElement, css, html } from 'lit'
import type { Size, Tone, Variant } from './util'

export class ShipButton extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
      vertical-align: middle;
    }
    :host([hidden]) { display: none; }

    button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      font-family: var(--font-body);
      font-weight: 500;
      line-height: 1;
      border-radius: 6px;
      cursor: pointer;
      border: 1px solid transparent;
      background: transparent;
      color: var(--foreground);
      transition: background-color 120ms ease, border-color 120ms ease, filter 120ms ease, color 120ms ease;
      white-space: nowrap;
    }
    button:focus-visible {
      outline: 2px solid var(--primary);
      outline-offset: 2px;
    }
    button:disabled { cursor: not-allowed; opacity: 0.6; }

    /* sizes — match specs .btn (sm) and a slightly larger md/lg for Studio use */
    :host([size='sm']) button { font-size: 11px; padding: 4px 8px; }
    :host(:not([size])) button,
    :host([size='md']) button { font-size: 12px; padding: 5px 10px; }
    :host([size='lg']) button { font-size: 13px; padding: 8px 14px; }

    /* default (tone=default): neutral bordered button matching specs .btn */
    :host(:not([tone])) button,
    :host([tone='default']) button {
      background: var(--background);
      border-color: var(--border);
      color: var(--foreground);
    }
    :host(:not([tone])) button:hover:not(:disabled),
    :host([tone='default']) button:hover:not(:disabled) {
      background: var(--muted);
    }

    /* solid variant — primary colored fill. Matches specs .btn-primary. */
    :host([variant='solid'][tone='default']) button,
    :host([variant='solid']:not([tone])) button {
      background: var(--primary);
      border-color: var(--primary);
      color: var(--primary-foreground);
    }
    :host([variant='solid'][tone='default']) button:hover:not(:disabled),
    :host([variant='solid']:not([tone])) button:hover:not(:disabled) {
      filter: brightness(1.05);
    }
    :host([variant='solid'][tone='destructive']) button {
      background: var(--destructive);
      border-color: var(--destructive);
      color: oklch(0.99 0.02 95);
    }
    :host([variant='solid'][tone='destructive']) button:hover:not(:disabled) {
      filter: brightness(1.05);
    }
    :host([variant='solid'][tone='success']) button {
      background: var(--success); border-color: var(--success); color: oklch(0.99 0.02 95);
    }
    :host([variant='solid'][tone='accent']) button {
      background: var(--accent); border-color: var(--accent); color: var(--accent-foreground);
    }

    /* outline variant — matches specs .btn-danger */
    :host([variant='outline']) button {
      background: transparent;
      border-color: var(--border);
      color: var(--foreground);
    }
    :host([variant='outline'][tone='destructive']) button {
      color: var(--destructive); border-color: var(--destructive);
    }
    :host([variant='outline']) button:hover:not(:disabled) { background: var(--muted); }

    /* ghost variant — transparent, hover tint */
    :host([variant='ghost']) button {
      background: transparent; border-color: transparent; color: var(--muted-foreground);
    }
    :host([variant='ghost']) button:hover:not(:disabled) {
      background: var(--muted); color: var(--foreground);
    }
    :host([variant='ghost'][tone='destructive']) button { color: var(--destructive); }

    /* soft variant — tinted surface, used for pill-like affirm buttons */
    :host([variant='soft']) button {
      background: color-mix(in oklch, var(--primary) 12%, transparent);
      border-color: color-mix(in oklch, var(--primary) 25%, transparent);
      color: var(--primary);
    }
    :host([variant='soft'][tone='destructive']) button {
      background: color-mix(in oklch, var(--destructive) 12%, transparent);
      border-color: color-mix(in oklch, var(--destructive) 28%, transparent);
      color: var(--destructive);
    }

    .spinner {
      width: 12px; height: 12px;
      border: 1.5px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
      display: inline-block;
    }
    @keyframes spin { to { transform: rotate(360deg); } }

    ::slotted([slot='leading']), ::slotted([slot='trailing']) {
      display: inline-flex; align-items: center;
    }
  `

  static properties = {
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
    loading: { type: Boolean, reflect: true },
    type: { type: String, reflect: true },
  }

  tone?: Tone
  variant: Variant = 'solid'
  size: Size = 'md'
  disabled = false
  loading = false
  type: 'button' | 'submit' | 'reset' = 'button'
  render() {
    return html`
      <button
        part="button"
        type=${this.type}
        ?disabled=${this.disabled || this.loading}
        aria-busy=${this.loading ? 'true' : 'false'}
      >
        ${this.loading
          ? html`<span class="spinner" aria-hidden="true"></span>`
          : html`<slot name="leading"></slot>`}
        <slot></slot>
        <slot name="trailing"></slot>
      </button>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-button')) {
  customElements.define('ship-button', ShipButton)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-button': ShipButton
  }
}
