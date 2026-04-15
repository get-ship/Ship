/**
 * <ship-theme-toggle>
 *
 * Button that toggles the document between light and dark by flipping the
 * `.dark` class on documentElement. Persists the choice in localStorage
 * under `ship-theme`.
 *
 * Attributes:
 *   - theme: light | dark | auto — initial value (default auto, reads
 *     localStorage or prefers-color-scheme).
 *
 * Events:
 *   - ship-theme-change — CustomEvent { detail: { theme } }
 */

import { LitElement, css, html } from 'lit'
const STORAGE_KEY = 'ship-theme'

export class ShipThemeToggle extends LitElement {
  static styles = css`
    :host { display: inline-flex; }
    button {
      background: transparent;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 4px 8px;
      font: inherit;
      font-size: 12px;
      color: var(--foreground);
      cursor: pointer;
      display: inline-flex; align-items: center; gap: 6px;
    }
    button:hover { background: var(--muted); }
  `

  static properties = {
    theme: { type: String, reflect: true },
  }

  theme: 'light' | 'dark' | 'auto' = 'auto'
  connectedCallback() {
    super.connectedCallback()
    if (typeof window === 'undefined') return
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === 'light' || saved === 'dark') {
      this.theme = saved
    } else {
      const prefersDark = window.matchMedia?.('(prefers-color-scheme: dark)')?.matches
      this.theme = prefersDark ? 'dark' : 'light'
    }
    this.apply()
  }

  private apply() {
    const dark = this.theme === 'dark'
    document.documentElement.classList.toggle('dark', dark)
  }

  private toggle = () => {
    this.theme = this.theme === 'dark' ? 'light' : 'dark'
    try { localStorage.setItem(STORAGE_KEY, this.theme) } catch { /* quota */ }
    this.apply()
    this.dispatchEvent(new CustomEvent('ship-theme-change', {
      detail: { theme: this.theme }, bubbles: true, composed: true,
    }))
  }

  render() {
    const isDark = this.theme === 'dark'
    return html`
      <button
        @click=${this.toggle}
        aria-label="Toggle theme"
        aria-pressed=${isDark ? 'true' : 'false'}
        title="Toggle theme"
      >
        ${isDark ? '🌙' : '☀'} <slot>${isDark ? 'Dark' : 'Light'}</slot>
      </button>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-theme-toggle')) {
  customElements.define('ship-theme-toggle', ShipThemeToggle)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-theme-toggle': ShipThemeToggle
  }
}
