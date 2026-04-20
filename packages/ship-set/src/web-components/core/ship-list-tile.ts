/**
 * <ship-list-tile>
 *
 * Generic row/card tile primitive. Absorbs the former ship-list-item,
 * ship-event-row, ship-log-line, ship-history-row, and ship-spec-row
 * under a single extended API.
 *
 * Attributes:
 *   - variant: 'row' | 'card' (default: 'row')
 *       row  — inline flex, hover background, thin divider.
 *       card — elevated surface, border-radius, subtle shadow.
 *   - interactive: boolean (default: false) — role=button, tabindex=0,
 *       cursor:pointer, Enter/Space dispatches `ship-select`.
 *   - selected: boolean (default: false, reflected) — reflects aria-selected
 *       and visual highlight.
 *   - disabled: boolean (default: false, reflected) — blocks interaction
 *       and dims the tile.
 *   - density: 'compact' | 'normal' | 'relaxed' (default: 'normal')
 *       padding: 6px / 10px / 16px vertical; horizontal pairs scale similarly.
 *   - tone: Tone (default: 'default') — tints accent (left border when a
 *       status-dot occupies the leading slot). Reuses the shared Tone type.
 *
 * Slots:
 *   - leading — leading icon/dot/avatar (~20px).
 *   - (default) — primary content (title/label). Required for a11y.
 *   - meta — secondary text below primary (legacy name preserved to avoid
 *       caller churn; functions as "secondary").
 *   - trailing — right-aligned content (badges, timestamps, chevrons).
 *
 * Events:
 *   - ship-select — CustomEvent<{}>, bubbles, composed. Dispatched when
 *       `interactive` is true and the user clicks or presses Enter/Space.
 *
 * CSS parts:
 *   - button — wrapper element when `interactive`.
 *   - row — flex container for leading + primary + trailing.
 *   - primary — the primary-text span.
 *   - secondary — the meta row below primary.
 *   - trailing — the trailing region wrapper.
 */

import { LitElement, css, html } from 'lit'
import { reducedMotion, toneVars } from './util'

type Variant = 'row' | 'card'
type Density = 'compact' | 'normal' | 'relaxed'

export class ShipListTile extends LitElement {
  static styles = [
    reducedMotion,
    toneVars,
    css`
      :host {
        display: block;
        --pad-y: 10px;
        --pad-x: 10px;
      }
      :host([density='compact'])  { --pad-y: 6px;  --pad-x: 8px; }
      :host([density='relaxed'])  { --pad-y: 16px; --pad-x: 14px; }

      .tile {
        width: 100%;
        text-align: left;
        background: transparent;
        border: 1px solid transparent;
        border-radius: var(--radius-md);
        padding: var(--pad-y) var(--pad-x);
        display: flex;
        flex-direction: column;
        gap: 4px;
        color: inherit;
        font: inherit;
        transition:
          background-color var(--duration-fast) var(--ease-out),
          border-color var(--duration-fast) var(--ease-out),
          box-shadow var(--duration-fast) var(--ease-out),
          transform var(--duration-fast) var(--ease-out);
      }

      /* variant=row (default) — thin hover background */
      :host([variant='row']) .tile:hover,
      :host(:not([variant])) .tile:hover {
        background: var(--muted);
        border-color: var(--border);
      }

      /* variant=card — elevated surface. Softer default border so the
       * resting card reads as a quiet surface; hover lifts it into focus
       * with a transform + stronger border + stronger shadow to match
       * shadcn/Radix Card-as-interactive-tile treatment. */
      :host([variant='card']) .tile {
        background: var(--card);
        border-color: color-mix(in oklch, var(--border) 60%, transparent);
        box-shadow: var(--shadow-xs);
      }
      :host([variant='card']) .tile:hover {
        border-color: color-mix(in oklch, var(--primary) 40%, transparent);
        box-shadow: var(--shadow-sm);
        transform: translateY(-1px);
      }
      :host([variant='card'][interactive]) .tile:active {
        transform: translateY(0);
        box-shadow: var(--shadow-xs);
        transition-duration: 60ms;
      }

      /* Focus ring when interactive */
      .tile:focus-visible {
        outline: none;
        box-shadow:
          0 0 0 var(--ring-offset) var(--background),
          0 0 0 calc(var(--ring-offset) + var(--ring-width))
            color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
      }
      @media (forced-colors: active) {
        .tile:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
      }

      /* Selected state */
      :host([selected]) .tile {
        background: var(--muted);
        border-color: var(--primary);
      }

      /* Disabled state */
      :host([disabled]) .tile {
        opacity: 0.6;
        cursor: not-allowed;
        pointer-events: none;
      }

      /* Interactive affordance */
      :host([interactive]) .tile { cursor: pointer; }

      /* Tone accent — left border tint when a tone is set and not default */
      :host([tone]:not([tone='default'])) .tile {
        border-left: 3px solid var(--tone-fg);
        padding-left: calc(var(--pad-x) - 2px);
      }

      .row { display: flex; align-items: center; gap: 8px; }
      .primary {
        flex: 1;
        min-width: 0;
        font-weight: var(--font-weight-medium);
        font-size: var(--text-base);
      }
      .secondary {
        display: flex;
        gap: 6px;
        align-items: center;
        font-size: 10px;
        color: var(--muted-foreground);
      }
      .secondary:empty { display: none; }

      ::slotted([slot='leading']),
      ::slotted([slot='trailing']) {
        display: inline-flex;
        align-items: center;
      }
    `,
  ]

  static properties = {
    variant: { type: String, reflect: true },
    interactive: { type: Boolean, reflect: true },
    selected: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    density: { type: String, reflect: true },
    tone: { type: String, reflect: true },
  }

  variant: Variant = 'row'
  interactive = false
  selected = false
  disabled = false
  density: Density = 'normal'
  tone = 'default'

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) {
      this.setAttribute('role', this.interactive ? 'button' : 'listitem')
    }
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('interactive')) {
      this.setAttribute('role', this.interactive ? 'button' : 'listitem')
    }
    if (changed.has('selected')) {
      this.setAttribute('aria-selected', this.selected ? 'true' : 'false')
    }
    if (changed.has('disabled')) {
      this.setAttribute('aria-disabled', this.disabled ? 'true' : 'false')
    }
  }

  private dispatchSelect() {
    if (this.disabled || !this.interactive) return
    this.dispatchEvent(
      new CustomEvent('ship-select', {
        detail: {},
        bubbles: true,
        composed: true,
      }),
    )
  }

  private onClick() {
    this.dispatchSelect()
  }

  private onKeyDown(e: KeyboardEvent) {
    if (!this.interactive || this.disabled) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this.dispatchSelect()
    }
  }

  render() {
    const tabindex = this.interactive && !this.disabled ? '0' : undefined
    return html`
      <div
        class="tile"
        part="button"
        tabindex=${tabindex as unknown as string}
        @click=${this.onClick}
        @keydown=${this.onKeyDown}
      >
        <div class="row" part="row">
          <slot name="leading"></slot>
          <span class="primary" part="primary"><slot></slot></span>
          <span part="trailing"><slot name="trailing"></slot></span>
        </div>
        <div class="secondary" part="secondary"><slot name="meta"></slot></div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-list-tile')) {
  customElements.define('ship-list-tile', ShipListTile)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-list-tile': ShipListTile
  }
}
