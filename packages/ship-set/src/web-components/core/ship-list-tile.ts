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
 *   - navigable: string (optional) — when set, the tile root renders as
 *       an `<a href>` so cmd-click, middle-click, and open-in-new-tab
 *       work as native anchor semantics. `target` and `rel` on the host
 *       element pass through to the anchor (e.g.
 *       `navigable="/x" target="_blank" rel="noopener"`). The anchor's
 *       default navigation happens unless a consumer calls
 *       `preventDefault` on the `ship-select` event (when `interactive`
 *       is also set). When `navigable` is set, role becomes `link`.
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
 *       When `navigable` is also set, calling `preventDefault` on this
 *       event also prevents the anchor's default navigation.
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

/**
 * Document-level side effect: installs a single CSS rule that draws a 1px
 * top border between adjacent row-variant tiles. Shadow DOM cannot style
 * sibling host elements from inside the shadow root, so this rule is
 * registered on the owning document via CSSStyleSheet / adoptedStyleSheets
 * (with a <style> fallback for environments that don't support the API).
 *
 * Rule suppression: when the current tile has the `selected` attribute the
 * tile's own border takes visual priority and the divider is hidden.
 */
const DIVIDER_CSS = `
ship-list-tile[variant='row'] + ship-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
ship-list-tile[variant='row'][selected] + ship-list-tile[variant='row'],
ship-list-tile[variant='row'] + ship-list-tile[variant='row'][selected] {
  border-top: none;
}
`

const _dividerInstalled = new WeakSet<Document>()
function installDividerStyles(doc: Document) {
  if (_dividerInstalled.has(doc)) return
  _dividerInstalled.add(doc)
  try {
    if (
      typeof CSSStyleSheet !== 'undefined' &&
      'replaceSync' in CSSStyleSheet.prototype
    ) {
      const sheet = new CSSStyleSheet()
      sheet.replaceSync(DIVIDER_CSS)
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet]
      return
    }
  } catch {
    /* fall through */
  }
  const style = doc.createElement('style')
  style.setAttribute('data-ship-list-tile-dividers', '')
  style.textContent = DIVIDER_CSS
  doc.head.appendChild(style)
}

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
        text-decoration: none;
        transition:
          background-color var(--duration-fast) var(--ease-out),
          border-color var(--duration-fast) var(--ease-out),
          box-shadow var(--duration-fast) var(--ease-out),
          transform var(--duration-fast) var(--ease-out);
      }
      a.tile { display: flex; }

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
        border-color: var(--border-subtle);
        box-shadow: var(--shadow-xs);
      }
      :host([variant='card']) .tile:hover {
        border-color: var(--tone-primary-border);
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
    navigable: { type: String, reflect: true },
    selected: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    density: { type: String, reflect: true },
    tone: { type: String, reflect: true },
  }

  variant: Variant = 'row'
  interactive = false
  navigable: string | null = null
  selected = false
  disabled = false
  density: Density = 'normal'
  tone = 'default'

  connectedCallback() {
    super.connectedCallback()
    installDividerStyles(this.ownerDocument)
    if (!this.hasAttribute('role')) {
      this.setAttribute('role', this.resolveRole())
    }
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('interactive') || changed.has('navigable')) {
      this.setAttribute('role', this.resolveRole())
    }
    if (changed.has('selected')) {
      this.setAttribute('aria-selected', this.selected ? 'true' : 'false')
    }
    if (changed.has('disabled')) {
      this.setAttribute('aria-disabled', this.disabled ? 'true' : 'false')
    }
  }

  private resolveRole(): string {
    // Link wins for a11y when navigable is set — a link with enhanced
    // keyboard semantics via interactive still announces as a link.
    if (this.navigable != null) return 'link'
    if (this.interactive) return 'button'
    return 'listitem'
  }

  /** Dispatch ship-select. Returns whether default was prevented. */
  private dispatchSelect(): boolean {
    if (this.disabled || !this.interactive) return false
    const ev = new CustomEvent('ship-select', {
      detail: {},
      bubbles: true,
      composed: true,
      cancelable: true,
    })
    this.dispatchEvent(ev)
    return ev.defaultPrevented
  }

  private onClick(e: MouseEvent) {
    // Only modify navigation when interactive. A navigable-only tile
    // acts as a pure anchor — no ship-select dispatch.
    if (!this.interactive) return
    const prevented = this.dispatchSelect()
    // When combined with navigable, consumers can preventDefault on
    // ship-select to also block the anchor's native navigation.
    if (prevented && this.navigable != null) {
      e.preventDefault()
    }
  }

  private onKeyDown(e: KeyboardEvent) {
    if (!this.interactive || this.disabled) return
    // Anchors already handle Enter natively. Skip to avoid double-firing
    // navigation, but still let Space trigger selection for parity with
    // the button variant.
    if (this.navigable != null && e.key === 'Enter') return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this.dispatchSelect()
    }
  }

  render() {
    const tabindex = this.interactive && !this.disabled ? '0' : undefined
    const inner = html`
      <div class="row" part="row">
        <slot name="leading"></slot>
        <span class="primary" part="primary"><slot></slot></span>
        <span part="trailing"><slot name="trailing"></slot></span>
      </div>
      <div class="secondary" part="secondary"><slot name="meta"></slot></div>
    `

    if (this.navigable != null) {
      // Pass-through target/rel from the host element so callers can
      // write `<ship-list-tile navigable="/x" target="_blank" rel="noopener">`.
      const target = this.getAttribute('target') ?? undefined
      const rel = this.getAttribute('rel') ?? undefined
      return html`
        <a
          class="tile"
          part="button"
          href=${this.navigable}
          target=${target as unknown as string}
          rel=${rel as unknown as string}
          tabindex=${tabindex as unknown as string}
          @click=${this.onClick}
          @keydown=${this.onKeyDown}
        >
          ${inner}
        </a>
      `
    }

    return html`
      <div
        class="tile"
        part="button"
        tabindex=${tabindex as unknown as string}
        @click=${this.onClick}
        @keydown=${this.onKeyDown}
      >
        ${inner}
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
