/**
 * <ship-timeline> + <ship-timeline-item>
 *
 * Vertical event timeline with left-gutter dots and connector line.
 *
 * <ship-timeline>
 *   Slots:
 *     - (default) <ship-timeline-item> children
 *
 * <ship-timeline-item>
 *   Attributes:
 *     - tone: default | info | success | warning | destructive | accent | muted
 *     - at: ISO or relative label
 *     - interactive: boolean — activate with click/Enter/Space
 *
 *   Events:
 *     - ship-select (when interactive)
 *
 *   Slots:
 *     - title
 *     - (default) body
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from './util'
import type { Tone } from './util'

export class ShipTimeline extends LitElement {
  static styles = css`
    :host { display: block; position: relative; }
    .list { display: flex; flex-direction: column; gap: 4px; }
  `

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'list')
  }
  render() {
    return html`<div class="list" part="list"><slot></slot></div>`
  }
}

export class ShipTimelineItem extends LitElement {
  static styles = [
    toneVars,
    css`
      :host {
        display: grid;
        grid-template-columns: 20px 1fr;
        column-gap: 10px;
        padding: 6px 0;
        position: relative;
      }
      .gutter {
        position: relative;
        display: flex;
        justify-content: center;
      }
      .gutter::before {
        content: '';
        position: absolute;
        top: 0; bottom: 0;
        width: 1px;
        background: var(--border);
      }
      :host(:first-of-type) .gutter::before { top: 10px; }
      :host(:last-of-type) .gutter::before { bottom: calc(100% - 10px); }
      .dot {
        position: relative;
        width: 10px; height: 10px;
        border-radius: var(--radius-full);
        background: var(--tone-fg);
        border: 2px solid var(--background);
        margin-top: 4px;
        box-shadow: var(--shadow-xs);
      }
      .body { min-width: 0; }
      .head {
        display: flex;
        align-items: baseline;
        gap: 8px;
        font-family: var(--font-body);
        font-size: var(--text-sm);
        font-weight: var(--font-weight-medium);
        color: var(--foreground);
      }
      .at { color: var(--muted-foreground); font-size: var(--text-xs); margin-left: auto; }
      .content {
        font-size: var(--text-sm);
        color: var(--muted-foreground);
        line-height: var(--leading-normal);
        margin-top: 2px;
      }
      :host([interactive]) { cursor: pointer; border-radius: var(--radius-sm); outline: none; }
      :host([interactive]:hover) { background: var(--muted); }
      :host([interactive]:focus-visible) {
        box-shadow:
          0 0 0 var(--ring-offset) var(--background),
          0 0 0 calc(var(--ring-offset) + var(--ring-width))
            color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
      }
      @media (forced-colors: active) {
        :host([interactive]:focus-visible) { outline: 2px solid CanvasText; outline-offset: 2px; }
      }
    `,
  ]

  static properties = {
    tone: { type: String, reflect: true },
    at: { type: String, reflect: true },
    interactive: { type: Boolean, reflect: true },
  }

  tone: Tone = 'default'
  at?: string
  interactive = false

  connectedCallback(): void {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'listitem')
    this.addEventListener('keydown', this._onKey)
    this.addEventListener('click', this._onClick)
  }

  disconnectedCallback(): void {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this._onKey)
    this.removeEventListener('click', this._onClick)
  }

  private _emit() {
    this.dispatchEvent(new CustomEvent('ship-select', { bubbles: true, composed: true, detail: {} }))
  }
  private _onClick = () => { if (this.interactive) this._emit() }
  private _onKey = (e: KeyboardEvent) => {
    if (!this.interactive) return
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); this._emit() }
  }

  updated() {
    if (this.interactive) {
      this.setAttribute('role', 'button')
      if (!this.hasAttribute('tabindex')) this.setAttribute('tabindex', '0')
    }
  }

  render() {
    return html`
      <div class="gutter"><span class="dot" part="dot"></span></div>
      <div class="body" part="body">
        <div class="head">
          <slot name="title"></slot>
          ${this.at ? html`<span class="at" part="at">${this.at}</span>` : null}
        </div>
        <div class="content"><slot></slot></div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined') {
  if (!customElements.get('ship-timeline')) customElements.define('ship-timeline', ShipTimeline)
  if (!customElements.get('ship-timeline-item')) customElements.define('ship-timeline-item', ShipTimelineItem)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-timeline': ShipTimeline
    'ship-timeline-item': ShipTimelineItem
  }
}
