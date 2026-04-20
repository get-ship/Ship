/**
 * <ship-timestamp>
 *
 * Renders a point-in-time. Shows a relative label by default with the
 * absolute ISO string exposed via `title` for hover. Updates automatically
 * once per minute while mounted.
 *
 * Attributes:
 *   - value: ISO date string (required)
 *   - format: relative | absolute | both (default: relative)
 */

import { LitElement, css, html } from 'lit'

type Format = 'relative' | 'absolute' | 'both'

const MINUTE = 60_000
const HOUR = 3_600_000
const DAY = 86_400_000

export function formatRelative(iso: string, now = Date.now()): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return iso
  const diff = now - t
  const abs = Math.abs(diff)
  const suffix = diff >= 0 ? 'ago' : 'from now'
  if (abs < MINUTE) return `just now`
  if (abs < HOUR) return `${Math.floor(abs / MINUTE)}m ${suffix}`
  if (abs < DAY) return `${Math.floor(abs / HOUR)}h ${suffix}`
  const days = Math.floor(abs / DAY)
  if (days < 30) return `${days}d ${suffix}`
  if (days < 365) return `${Math.floor(days / 30)}mo ${suffix}`
  return `${Math.floor(days / 365)}y ${suffix}`
}

export class ShipTimestamp extends LitElement {
  static styles = css`
    :host {
      display: inline;
      font-family: var(--font-body);
      font-size: inherit;
      color: inherit;
    }
    .abs { color: var(--muted-foreground); margin-left: 4px; }
  `

  static properties = {
    value: { type: String, reflect: true },
    format: { type: String, reflect: true },
  }

  value = ''
  format: Format = 'relative'
  private _timer?: ReturnType<typeof setInterval>

  connectedCallback(): void {
    super.connectedCallback()
    this._timer = setInterval(() => this.requestUpdate(), MINUTE)
  }

  disconnectedCallback(): void {
    super.disconnectedCallback()
    if (this._timer) clearInterval(this._timer)
  }

  render() {
    if (!this.value) return html`<span></span>`
    const iso = this.value
    const rel = formatRelative(iso)
    const parsed = Date.parse(iso)
    const abs = Number.isNaN(parsed) ? iso : new Date(parsed).toISOString()
    this.title = abs
    if (this.format === 'absolute') return html`<time datetime=${iso}>${abs}</time>`
    if (this.format === 'both')
      return html`<time datetime=${iso}>${rel}</time><span class="abs">(${abs})</span>`
    return html`<time datetime=${iso}>${rel}</time>`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-timestamp')) {
  customElements.define('ship-timestamp', ShipTimestamp)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-timestamp': ShipTimestamp
  }
}
