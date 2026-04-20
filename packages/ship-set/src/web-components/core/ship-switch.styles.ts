/**
 * Styles for <ship-switch>.
 *
 * Channel-driven so the track/thumb/tone/size matrix stays DRY. Matches
 * the visual weight of shadcn/ui's Switch + Radix Themes' Switch: pill
 * track, soft thumb with subtle shadow, tone-aware fill on checked,
 * stacked focus ring, hover lift on the track border.
 *
 *   --tone / --tone-on-fg  — accent color + readable thumb contrast, set
 *     once per :host([tone=…]) block; track/thumb rules read the channel.
 *   --sw-w / --sw-h / --sw-thumb
 *     Size channel. sm 24×14, md 28×16, lg 32×18 — matches shadcn's
 *     Switch dimensions so a row of label + switch lines up identically.
 *
 * The thumb uses absolute positioning with left:1px (off) and
 * left:calc(100% - thumb - 1px) (on) so the pellet always rides the
 * inside of the border regardless of size, with one transition property.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipSwitchStyles = [
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

      /* Tone channel — overridden per :host([tone=…]) below. Default is
       * primary so an attribute-less switch reads as the brand toggle. */
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

    /* Size channel. Each pair is (track W × H, thumb diameter). The
     * 1px border is absorbed via box-sizing so the visible pill reads
     * at the advertised dimensions. */
    :host(:not([size])),
    :host([size='md']) { --sw-w: 28px; --sw-h: 16px; --sw-thumb: 12px; }
    :host([size='sm']) { --sw-w: 24px; --sw-h: 14px; --sw-thumb: 10px; }
    :host([size='lg']) { --sw-w: 32px; --sw-h: 18px; --sw-thumb: 14px; }

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
     * Track — the pill. Border adds a crisp 1px edge that keeps the off
     * state legible against neutral backgrounds (shadcn does the same).
     * Transitions are token-driven so reduced-motion collapses them.
     * ------------------------------------------------------------------ */
    .track {
      position: relative;
      display: inline-block;
      flex-shrink: 0;
      width: var(--sw-w);
      height: var(--sw-h);
      background: var(--muted);
      border: 1px solid color-mix(in oklch, var(--border) 140%, transparent);
      border-radius: var(--radius-full);
      box-sizing: border-box;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }

    /* Hover — deepen the border + subtle shadow, matching ship-button's
     * quiet hover language. Skipped on disabled so users can tell when
     * the control is unreachable. */
    :host(:not([disabled]):not([checked])) label:hover .track {
      background: color-mix(in oklch, var(--muted) 70%, var(--foreground) 6%);
      border-color: color-mix(in oklch, var(--border) 180%, transparent);
    }

    /* Checked — fill with tone, top-highlight inset for the premium feel
     * (same recipe as ship-button solid). Border picks up a slightly
     * darker edge so the pill still reads crisply over light surfaces. */
    :host([checked]) .track {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent);
    }
    :host([checked]:not([disabled])) label:hover .track {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 26%, transparent),
        var(--shadow-xs);
    }

    /* ------------------------------------------------------------------
     * Thumb — sliding pellet. Sits 1px inside the border in both states;
     * the left calc flips between those anchors so sizes stay in sync.
     * ------------------------------------------------------------------ */
    .thumb {
      position: absolute;
      top: 50%;
      left: 1px;
      width: var(--sw-thumb);
      height: var(--sw-thumb);
      background: var(--background);
      border-radius: 50%;
      box-shadow:
        0 1px 2px 0 oklch(0 0 0 / 18%),
        0 1px 1px 0 oklch(0 0 0 / 10%);
      transform: translateY(-50%);
      transition:
        left var(--duration-fast) var(--ease-out),
        background-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
      pointer-events: none;
    }
    :host([checked]) .thumb {
      left: calc(100% - var(--sw-thumb) - 3px);
      background: var(--tone-on-fg);
    }

    /* ------------------------------------------------------------------
     * Native input — visually hidden but preserved for form submission,
     * keyboard toggle (Space), and a11y tools. Stretches over the track
     * so hit-testing matches the visible affordance.
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

    /* Focus ring — stacked box-shadow on the track so the ring sits
     * outside the pill without reserving layout. Uses :has() so the
     * ring travels with the native input's :focus-visible, keeping
     * keyboard semantics identical to ship-button. */
    :host(:not([disabled])) label:has(input:focus-visible) .track {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    :host([checked]:not([disabled])) label:has(input:focus-visible) .track {
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }

    /* Forced colors — drop the color-mix ring in favor of a CanvasText
     * outline that high-contrast modes can render reliably. */
    @media (forced-colors: active) {
      :host(:not([disabled])) label:has(input:focus-visible) .track {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
      .track { border-color: CanvasText; }
      :host([checked]) .track { background: Highlight; }
    }

    /* ------------------------------------------------------------------
     * Disabled — muted + desaturated, no interactions. Distinct from
     * loading (which this component doesn't have) so the unreachable
     * state is unambiguous.
     * ------------------------------------------------------------------ */
    :host([disabled]) {
      color: var(--muted-foreground);
    }
    :host([disabled]) .track {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 70%, transparent);
      filter: saturate(0.6);
      box-shadow: none;
    }
    :host([disabled][checked]) .track {
      background: color-mix(in oklch, var(--tone) 55%, var(--muted));
      border-color: color-mix(in oklch, var(--tone) 40%, var(--border));
    }
    :host([disabled]) .thumb {
      box-shadow: 0 1px 1px 0 oklch(0 0 0 / 10%);
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
