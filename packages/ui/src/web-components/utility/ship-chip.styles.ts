/**
 * Styles for <ship-chip>.
 *
 * Interactive filter / selection pill — the counterpart to <ship-badge>.
 * Where badge is a static inline label, chip is a Radix Toggle: hoverable,
 * focusable, selectable, optionally removable. Used as filter rails,
 * multi-select tag inputs, and enum-cell markers in dense tables.
 *
 * Channel architecture — every selector below reads from exactly one of:
 *   --tone / --tone-fg / --tone-contrast
 *     Accent / text / solid-contrast. Set once per :host([tone=…]) block.
 *   --chip-h / --chip-px / --chip-fs / --chip-gap / --chip-icon / --chip-remove
 *     Size channel — sm 22px, md 26px, lg 30px. sm is the Ship filter-rail
 *     default so existing templates keep their weight; md/lg scale up to
 *     shadcn Badge proportions for denser tag inputs / hero selection.
 *   --chip-radius
 *     Fully rounded by default (Radix Toggle / shadcn Chip convention).
 *
 * States are visually distinct:
 *   - unselected + interactive  : soft tone surface, hover deepens tint
 *   - selected                  : solid tone fill with contrast text
 *   - disabled                  : muted bg, desaturated, cursor forbidden
 *   - :focus-visible            : stacked box-shadow ring (tone-mixed)
 */

import { css } from 'lit'
import { reducedMotion } from '../util'

export const shipChipStyles = [
  reducedMotion,
  css`
    :host {
      display: inline-flex;
      vertical-align: middle;
      -webkit-tap-highlight-color: transparent;
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }
    :host([hidden]) { display: none; }

    /* Tone channel — same palette as ship-button so a row of mixed
     * chips + buttons share tone semantics. One-liner per tone keeps
     * additions cheap. */
    :host(:not([tone])),
    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
    :host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
    :host([tone='info'])        { --tone: var(--info);              --tone-fg: var(--info);              --tone-contrast: oklch(0.99 0.02 260); }
    :host([tone='success'])     { --tone: var(--success);           --tone-fg: var(--success);           --tone-contrast: oklch(0.99 0.02 150); }
    :host([tone='warning'])     { --tone: var(--warning);           --tone-fg: var(--warning);           --tone-contrast: oklch(0.2 0.04 75); }
    :host([tone='destructive']) { --tone: var(--destructive);       --tone-fg: var(--destructive);       --tone-contrast: oklch(0.99 0.02 25); }
    :host([tone='accent'])      { --tone: var(--accent);            --tone-fg: var(--accent);            --tone-contrast: var(--accent-foreground); }

    /* Size channel — sm matches pre-polish filter rails; md/lg step up
     * proportionally. Heights +4px per step so sibling controls align. */
    :host(:not([size])),
    :host([size='md']) { --chip-h: 26px; --chip-px: 10px; --chip-fs: var(--text-sm);   --chip-gap: 6px; --chip-icon: 12px; --chip-remove: 16px; }
    :host([size='sm']) { --chip-h: 22px; --chip-px: 8px;  --chip-fs: var(--text-xs);   --chip-gap: 4px; --chip-icon: 10px; --chip-remove: 14px; }
    :host([size='lg']) { --chip-h: 30px; --chip-px: 12px; --chip-fs: var(--text-base); --chip-gap: 6px; --chip-icon: 14px; --chip-remove: 18px; }

    :host { --chip-radius: var(--radius-full); }

    /* ------------------------------------------------------------------
     * Core chip chrome. The host owns interaction — no inner <button>
     * — so Datastar's data-on:click on the host fires identically
     * whether the user hits the label, padding, or icon. Keyboard
     * activation (Enter / Space) synthesizes a host click in TS.
     * ------------------------------------------------------------------ */
    .chip {
      display: inline-flex;
      align-items: center;
      gap: var(--chip-gap);
      min-height: var(--chip-h);
      padding: 0 var(--chip-px);
      font-family: var(--font-body);
      font-size: var(--chip-fs);
      font-weight: var(--font-weight-medium);
      line-height: 1;
      letter-spacing: -0.005em;
      color: var(--tone-fg);
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--chip-radius);
      cursor: pointer;
      user-select: none;
      -webkit-user-select: none;
      white-space: nowrap;
      isolation: isolate;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        filter var(--duration-fast) var(--ease-out);
    }

    /* ------------------------------------------------------------------
     * Variant matrix — solid | soft | outline. Default (no variant
     * attr) is soft so filter rails stay legible against the page bg.
     * ------------------------------------------------------------------ */

    :host(:not([variant])) .chip,
    :host([variant='soft']) .chip {
      background: color-mix(in oklch, var(--tone) 14%, transparent);
      border-color: color-mix(in oklch, var(--tone) 22%, transparent);
      color: var(--tone-fg);
    }
    :host(:not([variant]):not([disabled])) .chip:hover,
    :host([variant='soft']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 22%, transparent);
      border-color: color-mix(in oklch, var(--tone) 32%, transparent);
    }
    :host(:not([variant]):not([disabled])) .chip:active,
    :host([variant='soft']:not([disabled])) .chip:active {
      background: color-mix(in oklch, var(--tone) 28%, transparent);
    }

    :host([variant='outline']) .chip {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 45%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']:not([tone])) .chip,
    :host([variant='outline'][tone='muted']) .chip {
      border-color: var(--border);
      color: var(--muted-foreground);
    }
    :host([variant='outline']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 10%, transparent);
      border-color: color-mix(in oklch, var(--tone) 70%, transparent);
    }
    :host([variant='outline']:not([tone]):not([disabled])) .chip:hover,
    :host([variant='outline'][tone='muted']:not([disabled])) .chip:hover {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      color: var(--foreground);
    }

    :host([variant='solid']) .chip {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
    }
    :host([variant='solid']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        var(--shadow-xs);
    }

    /* ------------------------------------------------------------------
     * Selected state — Radix Toggle pressed look. Promotes any variant
     * to a solid tone fill so [soft][soft][selected][soft] reads the
     * active one unambiguously. Selected + muted uses --foreground as
     * the fill so the pill asserts neutrally instead of washing out.
     * ------------------------------------------------------------------ */
    :host([selected]) .chip {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 10%);
      color: var(--tone-contrast);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent),
        var(--shadow-xs);
      font-weight: var(--font-weight-semibold);
    }
    :host([selected]:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
        var(--shadow-sm);
    }
    :host([selected]:not([tone])) .chip,
    :host([selected][tone='muted']) .chip {
      background: var(--foreground);
      border-color: var(--foreground);
      color: var(--background);
    }
    :host([selected]:not([tone]):not([disabled])) .chip:hover,
    :host([selected][tone='muted']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--foreground) 88%, var(--background));
    }

    /* Focus ring — stacked box-shadow so the ring sits outside the pill
     * without reserving layout. Fires on :host(:focus-visible) so it
     * travels with Tab navigation and programmatic focus identically. */
    :host(:focus-visible) { outline: none; }
    :host(:focus-visible) .chip {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    :host([selected]:focus-visible) .chip {
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }

    /* Disabled — muted, desaturated, uninteractive. Distinct from the
     * unselected look so users can tell when a chip is unreachable. */
    :host([disabled]) .chip {
      cursor: not-allowed;
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 80%, transparent);
      color: color-mix(in oklch, var(--muted-foreground) 70%, transparent);
      filter: saturate(0.65);
      box-shadow: none;
    }

    /* ------------------------------------------------------------------
     * Leading slot — icon / status dot / avatar. Sized from the icon
     * channel so glyphs optically match cap-height. currentColor flows
     * so <ship-status-dot> tints with the chip tone for free.
     * ------------------------------------------------------------------ */
    .leading {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: var(--chip-icon);
      height: var(--chip-icon);
      flex-shrink: 0;
      color: currentColor;
    }
    .leading { display: none; }
    .leading:has(::slotted(*)) { display: inline-flex; }
    ::slotted([slot='leading']) {
      display: inline-flex;
      align-items: center;
      color: currentColor;
      width: 100%;
      height: 100%;
    }

    /* ------------------------------------------------------------------
     * Remove button — × affordance on the trailing edge. Not focusable
     * separately from the chip (tabindex=-1 in TS); clicking stops
     * propagation so the chip's click doesn't also fire. Keyboard
     * removal: Backspace / Delete on the focused chip (handled in TS).
     * ------------------------------------------------------------------ */
    .remove {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: var(--chip-remove);
      height: var(--chip-remove);
      margin-inline: 2px -4px;
      padding: 0;
      border: 0;
      background: transparent;
      color: inherit;
      cursor: pointer;
      border-radius: var(--radius-full);
      flex-shrink: 0;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out);
      -webkit-tap-highlight-color: transparent;
    }
    .remove svg {
      width: 70%;
      height: 70%;
      display: block;
      stroke: currentColor;
      fill: none;
      stroke-width: 1.75;
      stroke-linecap: round;
    }
    .remove:hover:not(:disabled) {
      background: color-mix(in oklch, currentColor 18%, transparent);
    }
    :host([selected]) .remove:hover:not(:disabled) {
      background: color-mix(in oklch, white 24%, transparent);
    }
    .remove:focus-visible {
      outline: none;
      box-shadow: 0 0 0 2px color-mix(in oklch, var(--tone) 55%, transparent);
    }
    :host([disabled]) .remove { cursor: not-allowed; }

    /* Forced-colors — system colors for high-contrast mode. */
    @media (forced-colors: active) {
      .chip { border-color: CanvasText; color: CanvasText; }
      :host([selected]) .chip {
        background: Highlight;
        color: HighlightText;
        border-color: Highlight;
      }
      :host(:focus-visible) .chip {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }
  `,
]
