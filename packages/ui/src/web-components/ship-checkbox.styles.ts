/**
 * Styles for <ship-checkbox>.
 *
 * Channel-driven. Matches shadcn/ui's Checkbox + Radix Themes: rounded
 * square, tone-aware fill on check/indeterminate, white glyph, stacked
 * focus ring, muted border hover. The glyph is an inline SVG with
 * currentColor so it always picks up the readable contrast foreground
 * regardless of tone.
 *
 *   --tone / --tone-on-fg  — fill color + readable glyph contrast, set
 *     once per :host([tone=…]) block; box/glyph rules read the channel.
 *   --cb-size / --cb-radius / --cb-stroke
 *     Size channel. sm=14, md=16, lg=18 — matches shadcn's Checkbox.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipCheckboxStyles = [
  reducedMotion,
  css`
    :host {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      vertical-align: middle;
      font-family: var(--font-body);
      font-size: var(--text-base);
      color: var(--foreground);
      -webkit-tap-highlight-color: transparent;

      /* Tone channel — default is primary so an attribute-less checkbox
       * reads as the brand fill (matches shadcn/Radix defaults). */
      --tone: var(--primary);
      --tone-on-fg: var(--primary-foreground);
    }
    :host([hidden]) { display: none; }

    :host(:not([tone])),
    :host([tone='default']) {
      --tone: var(--primary);
      --tone-on-fg: var(--primary-foreground);
    }
    :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: oklch(0.99 0.02 260); }
    :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: oklch(0.99 0.02 150); }
    :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: oklch(0.2 0.04 75); }
    :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: oklch(0.99 0.02 25); }
    :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }

    /* Size channel. Stroke width scales with size so the check glyph
     * maintains the same optical weight from sm through lg. */
    :host(:not([size])),
    :host([size='md']) {
      --cb-size: 16px;
      --cb-radius: var(--radius-sm);
      --cb-stroke: 2px;
    }
    :host([size='sm']) {
      --cb-size: 14px;
      --cb-radius: 3px;
      --cb-stroke: 1.75px;
    }
    :host([size='lg']) {
      --cb-size: 18px;
      --cb-radius: var(--radius-sm);
      --cb-stroke: 2.25px;
    }

    label {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      cursor: pointer;
      user-select: none;
      -webkit-user-select: none;
    }
    :host([disabled]) label { cursor: not-allowed; }

    /* ------------------------------------------------------------------
     * Box — the square. Border adds a crisp edge; box-sizing absorbs the
     * border so the visible square reads at the advertised size.
     * ------------------------------------------------------------------ */
    .box {
      position: relative;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      width: var(--cb-size);
      height: var(--cb-size);
      background: var(--background);
      border: 1px solid color-mix(in oklch, var(--border) 160%, transparent);
      border-radius: var(--cb-radius);
      box-sizing: border-box;
      box-shadow: var(--shadow-xs);
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }

    /* Hover — quiet border darkening + subtle shadow bump. Matches
     * ship-input's at-rest-to-hover language so a form row reads as
     * one cohesive surface. */
    :host(:not([disabled]):not([checked]):not([indeterminate])) label:hover .box {
      border-color: color-mix(in oklch, var(--border) 220%, transparent);
      box-shadow: var(--shadow-sm);
    }

    /* Invalid — destructive border regardless of tone, so an error on
     * an info/success-toned checkbox is unmissable. Mirrors ship-input. */
    :host([invalid]) .box {
      border-color: color-mix(in oklch, var(--destructive) 70%, var(--border));
    }
    :host([invalid]:not([disabled])) label:hover .box {
      border-color: color-mix(in oklch, var(--destructive) 85%, var(--border));
    }

    /* Checked / indeterminate — fill with tone, top-highlight inset for
     * the premium feel (same recipe as ship-button solid). Border picks
     * up a slightly darker edge so the square still reads crisply. */
    :host([checked]) .box,
    :host([indeterminate]) .box {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent),
        var(--shadow-xs);
    }
    :host([checked]:not([disabled])) label:hover .box,
    :host([indeterminate]:not([disabled])) label:hover .box {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
        var(--shadow-sm);
    }

    /* ------------------------------------------------------------------
     * Glyphs — check + indeterminate. Rendered in currentColor on
     * --tone-on-fg so the glyph is always legible against the fill.
     * Opacity/scale transitions give a tiny snap when toggling — the
     * kind of detail that sells the component as production-tier.
     * ------------------------------------------------------------------ */
    .glyph {
      position: absolute;
      inset: 0;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      color: var(--tone-on-fg);
      opacity: 0;
      transform: scale(0.6);
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
      pointer-events: none;
    }
    :host([checked]:not([indeterminate])) .glyph.check,
    :host([indeterminate]) .glyph.mixed {
      opacity: 1;
      transform: scale(1);
    }

    .glyph svg {
      width: 100%;
      height: 100%;
      display: block;
    }
    .glyph svg path,
    .glyph svg line {
      stroke: currentColor;
      stroke-width: var(--cb-stroke);
      stroke-linecap: round;
      stroke-linejoin: round;
      fill: none;
    }

    /* ------------------------------------------------------------------
     * Native input — visually hidden but preserved for form submission,
     * keyboard toggle (Space), and a11y tools. Stretches over the box so
     * hit-testing matches the visible affordance.
     * ------------------------------------------------------------------ */
    input {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      margin: 0;
      padding: 0;
      opacity: 0;
      cursor: inherit;
      appearance: none;
      -webkit-appearance: none;
    }

    /* Focus ring — stacked box-shadow on the box so the ring sits
     * outside the square without reserving layout. Uses :has() so the
     * ring travels with the native input's :focus-visible, keeping
     * keyboard semantics identical to ship-button / ship-switch. */
    :host(:not([disabled])) label:has(input:focus-visible) .box {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    :host([invalid]:not([disabled])) label:has(input:focus-visible) .box {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
    }

    /* Forced colors — drop the color-mix ring in favor of a CanvasText
     * outline that high-contrast modes can render reliably. */
    @media (forced-colors: active) {
      .box { border-color: CanvasText; }
      :host([checked]) .box,
      :host([indeterminate]) .box { background: Highlight; }
      .glyph { color: HighlightText; }
      :host(:not([disabled])) label:has(input:focus-visible) .box {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }

    /* ------------------------------------------------------------------
     * Disabled — muted + desaturated, no interactions. Distinct from
     * invalid/loading so the unreachable state is unambiguous.
     * ------------------------------------------------------------------ */
    :host([disabled]) {
      color: var(--muted-foreground);
    }
    :host([disabled]) .box {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 70%, transparent);
      filter: saturate(0.6);
      box-shadow: none;
    }
    :host([disabled][checked]) .box,
    :host([disabled][indeterminate]) .box {
      background: color-mix(in oklch, var(--tone) 55%, var(--muted));
      border-color: color-mix(in oklch, var(--tone) 40%, var(--border));
    }

    /* Slotted label — inherits host color + size so callers don't need
     * to restyle. Wraps naturally when the row gets tight. */
    ::slotted(*) {
      font: inherit;
      color: inherit;
      line-height: var(--leading-tight);
    }
  `,
]
