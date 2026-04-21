/**
 * <ship-avatar>
 *
 * Avatar tile: image when `src` loads, otherwise initials derived from
 * `name` (or supplied explicitly via `initials`). Single-word names ->
 * 1-char initial; multi-word -> first char of first and last word,
 * uppercased.
 *
 * Attributes:
 *   - src: image URL. Falls back to initials on error or when absent.
 *   - name: source for initials and aria-label.
 *   - initials: explicit initials override (e.g. "AB"). Takes precedence
 *       over derivation from `name` when set. Uppercased for display.
 *   - size: sm | md | lg | xl (default md = 32px)
 *       sm = 28px — bumped from 24px so compact list-tile rows with
 *         density="compact" have a visible gradient tile. The previous
 *         24px had zero production usage.
 *       md = 32px (default)
 *       lg = 40px
 *       xl = 56px
 *   - shape: round (default) | square. `square` uses `--radius-sm` so
 *       the tile reads as a rounded square — the pre-migration
 *       treatment for agent avatars. Round keeps the circular mask.
 *   - tone: background tone. Respected in the initials path too —
 *       `tone="primary"` yields a solid `--primary` background with
 *       `--primary-foreground` text. Unset / `default` / `muted` keep
 *       the primary→accent gradient fallback so anonymous avatars stay
 *       distinctive without any per-site configuration.
 *
 * Combining `tone="primary" shape="square"` reproduces the pre-migration
 * orange square avatar treatment exactly.
 */

import { LitElement, css, html } from 'lit'
import { reducedMotion, toneVars } from './util'
import type { Tone } from './util'

export type AvatarSize = 'sm' | 'md' | 'lg' | 'xl'
export type AvatarShape = 'round' | 'square'

export function initialsFromName(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 0) return ''
  if (parts.length === 1) return parts[0].charAt(0).toUpperCase()
  return (parts[0].charAt(0) + parts[parts.length - 1].charAt(0)).toUpperCase()
}

export class ShipAvatar extends LitElement {
  static styles = [
    reducedMotion,
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
      :host([size='sm']) { width: 28px; height: 28px; font-size: var(--text-xs); }
      :host([size='lg']) { width: 40px; height: 40px; font-size: var(--text-md); }
      :host([size='xl']) { width: 56px; height: 56px; font-size: var(--text-lg); }

      /* Square shape — rounded-square tile. Used for branded/product
       * avatars (agents) where the round circle reads as "user photo".
       * Round stays the default. */
      :host([shape='square']) { border-radius: var(--radius-sm); }

      /* Rich initials tile — gradient background, bold uppercase glyph.
       * Applied when the host is rendering initials (i.e. no loaded image).
       * Uses [data-initials] so image and initials paths share the same
       * host shell without cascading into the image. Anonymous case —
       * no explicit tone or tone=default/muted — renders the
       * primary→accent gradient. Explicit tones short-circuit below. */
      :host([data-initials]) {
        background:
          linear-gradient(
            135deg,
            color-mix(in oklch, var(--primary) 70%, transparent),
            color-mix(in oklch, var(--accent) 70%, transparent)
          );
        color: var(--foreground);
      }
      /* Explicit tone="primary" → solid primary background + primary
       * foreground text. Reproduces the pre-migration orange square. */
      :host([data-initials][tone='primary']) {
        background: var(--primary);
        color: var(--primary-foreground);
      }
      /* Explicit tone="accent" → solid accent. */
      :host([data-initials][tone='accent']) {
        background: var(--accent);
        color: var(--accent-foreground);
      }
      /* Other explicit tones (info/success/warning/destructive) use the
       * existing tone wash from toneVars — soft tint + fg. Muted and
       * default fall through to the gradient above. */
      :host([data-initials][tone='info']),
      :host([data-initials][tone='success']),
      :host([data-initials][tone='warning']),
      :host([data-initials][tone='destructive']) {
        background: var(--tone-bg);
        color: var(--tone-fg);
      }

      .wrap {
        width: 100%; height: 100%;
        display: inline-flex; align-items: center; justify-content: center;
      }
      .initials {
        text-transform: uppercase;
        font-weight: var(--font-weight-bold);
        letter-spacing: 0.02em;
        line-height: 1;
      }
      img { width: 100%; height: 100%; object-fit: cover; display: block; }
    `,
  ]

  static properties = {
    src: { type: String },
    name: { type: String },
    initials: { type: String },
    size: { type: String, reflect: true },
    shape: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    _errored: { state: true },
  }

  src = ''
  name = ''
  initials = ''
  size: AvatarSize = 'md'
  shape: AvatarShape = 'round'
  tone: Tone = 'muted'
  private _errored = false

  private onError = () => {
    this._errored = true
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('src')) this._errored = false
    // Reflect a [data-initials] marker attribute when we're rendering the
    // initials path so CSS can apply the rich tile treatment without
    // leaking those styles onto the image variant.
    const showImage = Boolean(this.src) && !this._errored
    if (showImage) this.removeAttribute('data-initials')
    else this.setAttribute('data-initials', '')
  }

  private resolvedInitials(): string {
    const explicit = (this.initials || '').trim()
    if (explicit) return explicit.toUpperCase()
    return initialsFromName(this.name)
  }

  render() {
    const showImage = this.src && !this._errored
    const initials = this.resolvedInitials()
    return html`
      <span class="wrap" role="img" aria-label=${this.name || initials || 'avatar'} part="wrap">
        ${showImage
          ? html`<img src=${this.src} alt=${this.name} @error=${this.onError} />`
          : html`<span class="initials" part="initials">${initials}</span>`}
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
