/**
 * <ship-toolbar>
 *
 * Horizontal flex row of actions. Matches specs `.footer` (action bar at
 * bottom, spacer + buttons right-aligned). A child with `slot="spacer"`
 * expands and pushes following children to the end; simpler callers can
 * just use `justify="end"`.
 *
 * Attributes:
 *   - justify: start | center | end | between (default: start)
 *   - align: center | start | end | baseline (default: center)
 *   - gap: sm | md | lg (default: md — 8px)
 *   - variant: default | bordered (bordered adds a top 1px border + padding)
 */

import { LitElement, css, html } from 'lit'
export class ShipToolbar extends LitElement {
  static styles = css`
    :host { display: flex; align-items: center; gap: 8px; }
    :host([gap='sm']) { gap: 4px; }
    :host([gap='lg']) { gap: 12px; }
    :host([justify='center']) { justify-content: center; }
    :host([justify='end']) { justify-content: flex-end; }
    :host([justify='between']) { justify-content: space-between; }
    :host([align='start']) { align-items: flex-start; }
    :host([align='end']) { align-items: flex-end; }
    :host([align='baseline']) { align-items: baseline; }
    :host([variant='bordered']) {
      padding: 12px 20px;
      border-top: 1px solid var(--border);
    }
    ::slotted([slot='spacer']) { flex: 1; }
  `

  static properties = {
    justify: { type: String, reflect: true },
    align: { type: String, reflect: true },
    gap: { type: String, reflect: true },
    variant: { type: String, reflect: true },
  }

  justify: 'start' | 'center' | 'end' | 'between' = 'start'
  align: 'center' | 'start' | 'end' | 'baseline' = 'center'
  gap: 'sm' | 'md' | 'lg' = 'md'
  variant: 'default' | 'bordered' = 'default'

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'toolbar')
    if (!this.hasAttribute('aria-orientation')) this.setAttribute('aria-orientation', 'horizontal')
    this.addEventListener('keydown', this.onKey)
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this.onKey)
  }

  private focusable(): HTMLElement[] {
    const slot = this.renderRoot.querySelector('slot:not([name])') as HTMLSlotElement | null
    const els = (slot?.assignedElements({ flatten: true }) ?? []) as HTMLElement[]
    return els.filter(el => !el.hasAttribute('disabled'))
  }

  private onKey = (e: KeyboardEvent) => {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
    const items = this.focusable()
    if (!items.length) return
    const active = (e.composedPath().find(n => items.includes(n as HTMLElement)) as HTMLElement) || null
    const i = active ? items.indexOf(active) : -1
    const next = e.key === 'ArrowRight' ? (i + 1) % items.length : (i - 1 + items.length) % items.length
    items[next].focus()
    e.preventDefault()
  }

  render() {
    return html`<slot name="spacer"></slot><slot></slot>`
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-toolbar')) {
  customElements.define('ship-toolbar', ShipToolbar)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-toolbar': ShipToolbar
  }
}
