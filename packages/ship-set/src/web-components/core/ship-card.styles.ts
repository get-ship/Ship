/**
 * Styles for <ship-card>.
 *
 * Extracted from ship-card.ts to keep the component file under the 300-line
 * discipline cap and to give the same visual language the other polished
 * primitives share (ship-button, ship-input, ship-select).
 *
 * Architecture — three channels read by the chrome below:
 *   --tone / --tone-fg / --tone-bg / --tone-border
 *     Set once per :host([tone=…]) block via `toneVars` from util. The
 *     interactive/focus/rail/accent-bar rules read only from these so new
 *     tones land by adding one selector.
 *   --card-px / --card-py / --card-gap
 *     Density channel driven by `padding=[none|sm|md|lg]`. `md` is the
 *     default and matches the pre-polish values so existing callers don't
 *     shift a pixel.
 *   --card-radius
 *     Radius channel. Uses --radius-lg to match shadcn/ui's Card.
 *
 * The header/footer separators, hover lift, focus ring, and tone-tinted
 * rail all read from these channels — no per-variant rewrites.
 */

import { css } from 'lit'
import { reducedMotion, toneVars } from './util'

export const shipCardStyles = [
  reducedMotion,
  toneVars,
  css`
    :host {
      display: block;
      position: relative;
      background: var(--card);
      color: var(--card-foreground);
      border: 1px solid var(--border);
      border-radius: var(--card-radius);
      box-shadow: var(--shadow-card-rest);
      backdrop-filter: var(--card-backdrop-filter);
      -webkit-backdrop-filter: var(--card-backdrop-filter);
      /* Density channel — overridden per :host([padding=…]) below.
       * Defaults match pre-polish 16px/20px so existing composites
       * (ship-pipeline-definition, ship-pipeline-run, settings cards)
       * keep their visual weight. */
      --card-px: 20px;
      --card-py: 16px;
      --card-gap: var(--space-3);
      --card-radius: var(--radius-lg);
      padding: var(--card-py) var(--card-px);
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([hidden]) { display: none; }

    /* Density channel. 'none' is useful for nested composites that want
     * total control over internal spacing. */
    :host([padding='none']) {
      --card-px: 0px;
      --card-py: 0px;
      --card-gap: 0px;
    }
    :host([padding='sm']) {
      --card-px: var(--space-3);
      --card-py: var(--space-2);
      --card-gap: var(--space-2);
    }
    :host([padding='lg']) {
      --card-px: var(--space-6);
      --card-py: var(--space-5);
      --card-gap: var(--space-4);
    }

    /* Variant: outline drops the solid fill in favor of the border alone —
     * used for secondary surfaces that sit inside another card/panel. */
    :host([variant='outline']) {
      background: transparent;
      box-shadow: none;
    }

    /* Toned border when a tone is set (but not default). Keeps the surface
     * neutral; the tone lives on the edge. Applied on the non-fill path so
     * it doesn't fight the wash. */
    :host([tone]:not([tone='default']):not([fill])) {
      border-color: var(--tone-border);
    }

    /* fill — tone-tinted background wash. Pairs with status-rail or tone
     * for an alert-style card. Border picks up tone-border so the edge
     * follows the wash instead of clashing with the neutral token. */
    :host([fill]) {
      background: var(--tone-bg);
      border-color: var(--tone-border);
    }

    /* status-rail — 3px left border tinted by tone. Preserves padding by
     * offsetting the inline-start inset so inner text doesn't jump when
     * the rail toggles on. */
    :host([status-rail]) {
      border-left: 3px solid var(--tone-fg);
      padding-left: calc(var(--card-px) - 3px);
    }

    /* accent-bar — 2-stop gradient top-bar that fades in on hover.
     * Rendered as a pseudo-element so the bar scrolls with the card and
     * follows the radius curve without needing clip-path. */
    :host([accent-bar])::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 2px;
      background: linear-gradient(90deg, var(--primary), var(--accent));
      opacity: 0;
      border-radius: var(--card-radius) var(--card-radius) 0 0;
      transition: opacity var(--duration-medium) var(--ease-out);
      pointer-events: none;
    }
    :host([accent-bar]:hover)::before,
    :host([accent-bar][interactive]:focus-visible)::before {
      opacity: 1;
    }

    /* ------------------------------------------------------------------
     * Interactive — hover lift + active press + focus-visible ring.
     * Non-interactive cards stay static; this block is the only place the
     * card picks up pointer affordances. Mirrors ship-button's motion so
     * the two read as the same design system when sitting side-by-side.
     * ------------------------------------------------------------------ */
    :host([interactive]) {
      cursor: pointer;
      -webkit-tap-highlight-color: transparent;
    }
    :host([interactive]:hover) {
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      box-shadow: var(--shadow-hover);
      transform: translateY(-1px);
    }
    @media (prefers-reduced-motion: reduce) {
      :host([interactive]:hover) { transform: none; }
    }
    :host([interactive][tone]:not([tone='default']):hover) {
      border-color: color-mix(in oklch, var(--tone-fg) 55%, var(--border));
    }
    :host([interactive]:active) {
      box-shadow: var(--shadow-card-rest);
      transform: translateY(0);
      transition-duration: 60ms;
    }

    /* Focus ring — only for keyboard focus on interactive cards. Stacked
     * box-shadow so the ring sits outside the border without reserving
     * layout space (matches ship-button / ship-select / ship-input). */
    :host([interactive]:focus-visible) {
      outline: none;
      border-color: color-mix(in oklch, var(--ring-color) 55%, var(--border));
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([interactive][tone]:not([tone='default']):focus-visible) {
      border-color: color-mix(in oklch, var(--tone-fg) 60%, var(--border));
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone-fg) var(--ring-opacity), transparent);
    }

    /* ------------------------------------------------------------------
     * Slots — header/content/footer with subtle separators. Separators
     * render only when the corresponding slot has projected content,
     * detected via the [data-filled] attribute set from slotchange.
     * ------------------------------------------------------------------ */
    .header,
    .footer {
      display: block;
    }
    .header[data-filled] {
      margin: calc(var(--card-py) * -1) calc(var(--card-px) * -1) var(--card-gap);
      padding: var(--card-py) var(--card-px);
      border-bottom: 1px solid
        var(--border-subtle);
      font-family: var(--heading-font);
      font-weight: var(--heading-weight);
      letter-spacing: var(--heading-tracking);
    }
    .footer[data-filled] {
      margin: var(--card-gap) calc(var(--card-px) * -1) calc(var(--card-py) * -1);
      padding: var(--card-py) var(--card-px);
      border-top: 1px solid
        var(--border-subtle);
    }
    /* When there's no inner padding there's nothing to negative-margin
     * against; the separator becomes a plain rule instead. */
    :host([padding='none']) .header[data-filled] {
      margin: 0 0 var(--space-3);
      padding: 0 0 var(--space-3);
    }
    :host([padding='none']) .footer[data-filled] {
      margin: var(--space-3) 0 0;
      padding: var(--space-3) 0 0;
    }

    /* status-rail narrows the left margin by the 3px rail so the header
     * separator doesn't sit over the colored rail. */
    :host([status-rail]) .header[data-filled] {
      margin-left: calc(var(--card-px) * -1 + 3px);
    }
    :host([status-rail]) .footer[data-filled] {
      margin-left: calc(var(--card-px) * -1 + 3px);
    }

    /* Forced-colors — keep the focus ring visible in high-contrast mode
     * where color-mix'd shadows collapse to the system palette. */
    @media (forced-colors: active) {
      :host([interactive]:focus-visible) {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }
  `,
]
