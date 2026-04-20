/**
 * <ship-avatar>
 *
 * Circular avatar: image when `src` loads, otherwise initials derived
 * from `name`. Single-word names -> 1-char initial; multi-word -> first
 * char of first and last word, uppercased.
 *
 * Attributes:
 *   - src: image URL. Falls back to initials on error or when absent.
 *   - name: source for initials and aria-label.
 *   - size: sm | md | lg | xl (default md = 32px)
 *   - tone: background tone for the initials fallback
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from './util'
import type { Tone } from './util'

export type AvatarSize = 'sm' | 'md' | 'lg' | 'xl'

export function initialsFromName(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 0) return ''
  if (parts.length === 1) return parts[0].charAt(0).toUpperCase()
  return (parts[0].charAt(0) + parts[parts.length - 1].charAt(0)).toUpperCase()
}

export class ShipAvatar extends LitElement {
  static styles = [
    toneVars,
    css`
      :host {
        display: inline-flex;
        width: 32px; height: 32px;
        border-radius: var(--radius-full);
        background: var(--tone-bg);
        color: var(--tone-fg);
        font-family: var(--font-body);
        font-weight: var(--font-weight-semibold);
        font-size: var(--text-sm);
        overflow: hidden;
        user-select: none;
      }
      :host([size='sm']) { width: 24px; height: 24px; font-size: var(--text-xs); }
      :host([size='lg']) { width: 40px; height: 40px; font-size: var(--text-md); }
      :host([size='xl']) { width: 56px; height: 56px; font-size: var(--text-lg); }
      .wrap {
        width: 100%; height: 100%;
        display: inline-flex; align-items: center; justify-content: center;
      }
      img { width: 100%; height: 100%; object-fit: cover; display: block; }
    `,
  ]

  static properties = {
    src: { type: String },
    name: { type: String },
    size: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    _errored: { state: true },
  }

  src = ''
  name = ''
  size: AvatarSize = 'md'
  tone: Tone = 'muted'
  private _errored = false

  private onError = () => {
    this._errored = true
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('src')) this._errored = false
  }

  render() {
    const showImage = this.src && !this._errored
    const initials = initialsFromName(this.name)
    return html`
      <span class="wrap" role="img" aria-label=${this.name || 'avatar'} part="wrap">
        ${showImage
          ? html`<img src=${this.src} alt=${this.name} @error=${this.onError} />`
          : html`<span part="initials">${initials}</span>`}
      </span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-avatar')) {
  customElements.define('ship-avatar', ShipAvatar)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-avatar': ShipAvatar
  }
}
