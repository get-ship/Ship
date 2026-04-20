/**
 * <ship-button>
 *
 * Wraps a native `<button>`. Keyboard, focus, and form participation
 * come for free from the native element.
 *
 * Attributes:
 *   - tone: default | primary | info | success | warning | destructive | accent | muted
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

import { LitElement, css, html, nothing } from 'lit'
import { reducedMotion } from './util'
import type { Size, Tone, Variant } from './util'

export class ShipButton extends LitElement {
  static styles = [reducedMotion, css`
    :host {
      display: inline-flex;
      vertical-align: middle;
      /* Per-host tone channel. Overridden per :host([tone=...]) below so
       * the variant selectors can stay tone-agnostic and compose freely. */
      --tone: var(--primary);
      --tone-fg: var(--primary-foreground);
      --tone-contrast: oklch(0.99 0.02 95);
    }
    :host([hidden]) { display: none; }

    /* Tone channel — each tone supplies its own accent color + a readable
     * contrast foreground for solid fills. The rest of the styling is driven
     * purely off --tone / --tone-contrast so variants stay DRY. */
    :host(:not([tone])),
    :host([tone='default']) {
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
    }
    :host([tone='primary']) {
      --tone: var(--primary);
      --tone-fg: var(--primary);
      --tone-contrast: var(--primary-foreground);
    }
    :host([tone='info']) {
      --tone: var(--info);
      --tone-fg: var(--info);
      --tone-contrast: oklch(0.99 0.02 260);
    }
    :host([tone='success']) {
      --tone: var(--success);
      --tone-fg: var(--success);
      --tone-contrast: oklch(0.99 0.02 150);
    }
    :host([tone='warning']) {
      --tone: var(--warning);
      --tone-fg: var(--warning);
      --tone-contrast: oklch(0.2 0.04 75);
    }
    :host([tone='destructive']) {
      --tone: var(--destructive);
      --tone-fg: var(--destructive);
      --tone-contrast: oklch(0.99 0.02 25);
    }
    :host([tone='accent']) {
      --tone: var(--accent);
      --tone-fg: var(--accent);
      --tone-contrast: var(--accent-foreground);
    }
    :host([tone='muted']) {
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }

    button {
      position: relative;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--space-2);
      font-family: var(--font-body);
      font-weight: var(--font-weight-medium);
      letter-spacing: -0.005em;
      line-height: 1;
      border-radius: var(--radius-md);
      cursor: pointer;
      border: 1px solid transparent;
      background: transparent;
      color: var(--foreground);
      isolation: isolate;
      user-select: none;
      -webkit-user-select: none;
      -webkit-tap-highlight-color: transparent;
      /* Optical alignment for g/y descenders — nudge content up by 0.5px so
       * the label sits visually centered between cap-height and baseline. */
      padding-block-start: calc(var(--btn-py) - 0.5px);
      padding-block-end: calc(var(--btn-py) + 0.5px);
      padding-inline: var(--btn-px);
      transform: translateZ(0);
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out),
        filter var(--duration-fast) var(--ease-out);
      white-space: nowrap;
    }

    /* Size tokens — drive padding/height/typography from a single --btn-*
     * channel so the matrix below never needs to repeat sizing logic. */
    :host(:not([size])),
    :host([size='md']) {
      --btn-h: 34px;
      --btn-px: var(--space-3);
      --btn-py: 0px;
      --btn-fs: var(--text-base);
      --btn-fw: var(--font-weight-semibold);
      --btn-gap: var(--space-2);
      --btn-icon: 14px;
    }
    :host([size='sm']) {
      --btn-h: 28px;
      --btn-px: var(--space-3);
      --btn-py: 0px;
      --btn-fs: var(--text-sm);
      --btn-fw: var(--font-weight-medium);
      --btn-gap: 6px;
      --btn-icon: 12px;
    }
    :host([size='lg']) {
      --btn-h: 40px;
      --btn-px: var(--space-4);
      --btn-py: 0px;
      --btn-fs: var(--text-md);
      --btn-fw: var(--font-weight-semibold);
      --btn-gap: var(--space-2);
      --btn-icon: 16px;
    }

    button {
      min-height: var(--btn-h);
      font-size: var(--btn-fs);
      font-weight: var(--btn-fw);
      gap: var(--btn-gap);
    }

    /* Icon-only — detected via the default slot being empty of visible text.
     * When active, collapse horizontal padding so the button renders as a
     * perfectly square affordance for icon adornments. */
    :host([data-icon-only]) button {
      padding-inline: 0;
      inline-size: var(--btn-h);
      min-inline-size: var(--btn-h);
    }

    /* ──────────────────────────────────────────────────────────────────
     * Variant matrix — each rule reads only from --tone/--tone-contrast,
     * so adding a tone is a single :host([tone=…]) channel override above.
     * ────────────────────────────────────────────────────────────────── */

    /* solid — tone fill, contrast text, subtle top-highlight for premium feel */
    :host([variant='solid']) button {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent),
        var(--shadow-xs);
    }
    :host([variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        var(--shadow-sm);
      transform: translateY(-1px);
    }
    :host([variant='solid']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, black 8%);
      box-shadow:
        inset 0 1px 2px 0 color-mix(in oklch, black 18%, transparent),
        var(--shadow-xs);
      transform: translateY(0);
    }

    /* Solid default tone: neutral surface that still feels like a filled
     * button (matches shadcn's default variant). */
    :host(:not([tone])[variant='solid']) button,
    :host([tone='default'][variant='solid']) button {
      background: var(--foreground);
      border-color: var(--foreground);
      color: var(--background);
    }
    :host(:not([tone])[variant='solid']) button:hover:not(:disabled),
    :host([tone='default'][variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--foreground) 88%, var(--background));
    }

    /* Default (no variant attr) is the neutral bordered button — matches
     * the existing view styling and shadcn's outline feel. */
    :host(:not([variant])) button {
      background: var(--background);
      border-color: var(--border);
      color: var(--foreground);
      box-shadow: var(--shadow-xs);
    }
    :host(:not([variant])) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--muted) 70%, var(--background));
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      box-shadow: var(--shadow-sm);
      transform: translateY(-1px);
    }
    :host(:not([variant])) button:active:not(:disabled) {
      background: var(--muted);
      box-shadow: var(--shadow-xs);
      transform: translateY(0);
    }
    /* Default button with a tone: colorize text + border subtly so tone is
     * legible without overpowering the neutral surface. */
    :host(:not([variant])[tone]:not([tone='default']):not([tone='muted'])) button {
      color: var(--tone-fg);
      border-color: color-mix(in oklch, var(--tone) 38%, var(--border));
    }
    :host(:not([variant])[tone]:not([tone='default']):not([tone='muted'])) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 8%, var(--background));
      border-color: color-mix(in oklch, var(--tone) 55%, var(--border));
    }

    /* soft — tone-tinted surface with a faint border for definition */
    :host([variant='soft']) button {
      background: color-mix(in oklch, var(--tone) 14%, transparent);
      border-color: color-mix(in oklch, var(--tone) 22%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='soft']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 22%, transparent);
      border-color: color-mix(in oklch, var(--tone) 32%, transparent);
      transform: translateY(-1px);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='soft']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 28%, transparent);
      transform: translateY(0);
      box-shadow: none;
    }

    /* outline — transparent with tone border + text */
    :host([variant='outline']) button {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 55%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']:not([tone])) button,
    :host([variant='outline'][tone='default']) button {
      border-color: var(--border);
      color: var(--foreground);
    }
    :host([variant='outline']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 10%, transparent);
      border-color: color-mix(in oklch, var(--tone) 75%, transparent);
      transform: translateY(-1px);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='outline']:not([tone])) button:hover:not(:disabled),
    :host([variant='outline'][tone='default']) button:hover:not(:disabled) {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
    }
    :host([variant='outline']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 16%, transparent);
      transform: translateY(0);
      box-shadow: none;
    }

    /* ghost — no chrome, tone tint on hover */
    :host([variant='ghost']) button {
      background: transparent;
      border-color: transparent;
      color: var(--tone-fg);
    }
    :host([variant='ghost']:not([tone])) button,
    :host([variant='ghost'][tone='default']) button {
      color: var(--muted-foreground);
    }
    :host([variant='ghost']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 12%, transparent);
    }
    :host([variant='ghost']:not([tone])) button:hover:not(:disabled),
    :host([variant='ghost'][tone='default']) button:hover:not(:disabled) {
      background: var(--muted);
      color: var(--foreground);
    }
    :host([variant='ghost']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 18%, transparent);
    }

    /* Premium treatment for the primary CTA (solid + accent): a subtle
     * diagonal gradient + faint colored glow on hover. Keeps the rest of
     * the matrix honest while giving the hero button some presence. */
    :host([variant='solid'][tone='accent']) button {
      background:
        linear-gradient(
          180deg,
          color-mix(in oklch, var(--accent) 100%, white 6%) 0%,
          var(--accent) 100%
        );
    }
    :host([variant='solid'][tone='accent']) button:hover:not(:disabled) {
      background:
        linear-gradient(
          180deg,
          color-mix(in oklch, var(--accent) 100%, white 10%) 0%,
          color-mix(in oklch, var(--accent) 95%, white 5%) 100%
        );
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
        var(--shadow-sm),
        0 0 0 1px color-mix(in oklch, var(--accent) 40%, transparent),
        0 6px 20px -8px color-mix(in oklch, var(--accent) 70%, transparent);
    }

    /* Destructive solid: bump contrast so it visibly demands attention. */
    :host([variant='solid'][tone='destructive']) button {
      font-weight: var(--font-weight-semibold);
    }
    :host([variant='solid'][tone='destructive']) button:hover:not(:disabled) {
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
        var(--shadow-sm),
        0 4px 14px -4px color-mix(in oklch, var(--destructive) 55%, transparent);
    }

    /* Focus ring — :focus-visible only, rendered via stacked box-shadows
     * so the ring sits outside the border without reserving layout space. */
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) 55%, transparent);
    }
    :host(:not([tone])) button:focus-visible,
    :host([tone='default']) button:focus-visible {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    @media (forced-colors: active) {
      button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
    }

    /* Disabled — muted, no pointer, no motion. Distinct enough from hover
     * state that users can tell it's unreachable, not just loading. */
    button:disabled {
      cursor: not-allowed;
      color: color-mix(in oklch, var(--muted-foreground) 65%, transparent);
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 80%, transparent);
      box-shadow: none;
      transform: none;
      filter: saturate(0.6);
    }
    :host([variant='ghost']) button:disabled,
    :host([variant='outline']) button:disabled {
      background: transparent;
    }
    :host([variant='outline']) button:disabled {
      border-color: color-mix(in oklch, var(--border) 60%, transparent);
    }

    /* Loading — fade the label out while preserving layout, overlay spinner
     * in the same color as the text would be (currentColor on the button). */
    :host([loading]) button { cursor: progress; }
    :host([loading]) .content {
      opacity: 0;
      transform: scale(0.96);
    }
    .content {
      display: inline-flex;
      align-items: center;
      gap: var(--btn-gap);
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }

    .spinner-wrap {
      position: absolute;
      inset: 0;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      pointer-events: none;
    }
    .spinner {
      width: var(--btn-icon);
      height: var(--btn-icon);
      border: 1.5px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: spin 0.7s linear infinite;
      display: inline-block;
    }
    @keyframes spin { to { transform: rotate(360deg); } }

    /* Slotted icons — constrain to the size channel so icons visually match
     * the typographic scale without each caller having to set a size. */
    ::slotted([slot='leading']),
    ::slotted([slot='trailing']) {
      display: inline-flex;
      align-items: center;
      flex: 0 0 auto;
    }
    ::slotted(svg) {
      width: var(--btn-icon);
      height: var(--btn-icon);
      display: block;
    }
  `]

  static properties = {
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
    loading: { type: Boolean, reflect: true },
    type: { type: String, reflect: true },
    label: { type: String },
    pressed: { type: String, reflect: true },
    expanded: { type: String, reflect: true },
  }

  tone?: Tone
  variant: Variant = 'solid'
  size: Size = 'md'
  disabled = false
  loading = false
  type: 'button' | 'submit' | 'reset' = 'button'
  label = ''
  pressed: 'true' | 'false' | '' = ''
  expanded: 'true' | 'false' | '' = ''

  /** Detect an empty default slot so we can render a square icon-only button
   * without requiring callers to opt in with a new attribute. */
  private handleDefaultSlotChange = (e: Event): void => {
    const slot = e.target as HTMLSlotElement
    const nodes = slot.assignedNodes({ flatten: true })
    const hasText = nodes.some((n) => {
      if (n.nodeType === Node.TEXT_NODE) return (n.textContent || '').trim().length > 0
      if (n.nodeType === Node.ELEMENT_NODE) {
        const el = n as Element
        return (el.textContent || '').trim().length > 0 || el.childElementCount > 0
      }
      return false
    })
    if (hasText) this.removeAttribute('data-icon-only')
    else this.setAttribute('data-icon-only', '')
  }

  render() {
    return html`
      <button
        part="button"
        type=${this.type}
        ?disabled=${this.disabled || this.loading}
        aria-busy=${this.loading ? 'true' : 'false'}
        aria-disabled=${(this.disabled || this.loading) ? 'true' : 'false'}
        aria-label=${this.label || nothing}
        aria-pressed=${this.pressed || nothing}
        aria-expanded=${this.expanded || nothing}
      >
        <span class="content">
          <slot name="leading"></slot>
          <slot @slotchange=${this.handleDefaultSlotChange}></slot>
          <slot name="trailing"></slot>
        </span>
        ${this.loading
          ? html`<span class="spinner-wrap" aria-hidden="true"><span class="spinner"></span></span>`
          : nothing}
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
