/**
 * Styles for <ship-badge>.
 *
 * Static status label — small, dense, non-interactive. Reads at a
 * glance in dense table cells and row metadata. Matches shadcn/ui's
 * Badge visual weight while preserving the existing Ship pipelines /
 * specs treatment (uppercase micro-label, tight letter-spacing).
 *
 * Channels — every selector below reads from exactly one of:
 *   --tone / --tone-fg / --tone-contrast
 *     Set once per :host([tone=…]) block. Variant rules stay
 *     tone-agnostic so new tones land with one selector.
 *   --bdg-h / --bdg-px / --bdg-fs / --bdg-gap / --bdg-icon
 *     Size channel. sm 18px, md 22px, lg 26px heights — matches shadcn
 *     Badge while keeping micro-label proportions (uppercase + 0.04em
 *     tracking) aligned with the existing pipelines/specs usage.
 *   --bdg-radius
 *     Shape channel. Default is --radius-sm (square corners) for visual
 *     parity with the pre-polish badge used across tables; [pill] flips
 *     to --radius-full for the rounded shadcn look.
 *
 * Interaction: none. Badges do not hover, do not focus, do not click —
 * if any of those are needed, the caller wants <ship-chip>. This is an
 * explicit constraint: badges exist to be read quickly inside other
 * chrome (rows, cards, headers) and an interactive-looking badge would
 * mis-cue users.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipBadgeStyles = [
  reducedMotion,
  css`
    :host {
      display: inline-flex;
      vertical-align: middle;
      /* Tone channel — overridden per :host([tone=…]). Default reads as
       * muted so a bare <ship-badge> sits quietly until given meaning. */
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }
    :host([hidden]) { display: none; }

    /* ------------------------------------------------------------------
     * Tone channel. Contrast colors picked so solid fills stay legible
     * in both light and dark themes (matches ship-button's tone table).
     * ------------------------------------------------------------------ */
    :host(:not([tone])),
    :host([tone='muted']) {
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }
    :host([tone='default']) {
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
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

    /* ------------------------------------------------------------------
     * Size channel. Heights chosen to slot cleanly into the common
     * contexts: sm inside dense table cells next to a monospace cell,
     * md for inline status after a title, lg for hero/summary cards.
     * Uppercase + 0.04em tracking matches the pre-polish pipelines
     * status pill so table rows don't re-flow.
     * ------------------------------------------------------------------ */
    :host(:not([size])),
    :host([size='md']) {
      --bdg-h: 22px;
      --bdg-px: 8px;
      --bdg-fs: 10.5px;
      --bdg-gap: 4px;
      --bdg-icon: 10px;
    }
    :host([size='sm']) {
      --bdg-h: 18px;
      --bdg-px: 6px;
      --bdg-fs: 9.5px;
      --bdg-gap: 3px;
      --bdg-icon: 8px;
    }
    :host([size='lg']) {
      --bdg-h: 26px;
      --bdg-px: 10px;
      --bdg-fs: 11.5px;
      --bdg-gap: 5px;
      --bdg-icon: 12px;
    }

    /* Shape channel — square by default (visual parity with pre-polish
     * badge), pill when opted-in. Both shapes share the same height &
     * padding so swapping shapes doesn't re-layout surrounding text. */
    :host { --bdg-radius: var(--radius-sm); }
    :host([pill]) { --bdg-radius: var(--radius-full); }

    /* ------------------------------------------------------------------
     * Core badge chrome. Non-interactive — no cursor, no hover, no
     * focus ring. Padding block is 0 so the explicit min-height owns
     * the vertical rhythm; inline padding follows the size channel.
     * ------------------------------------------------------------------ */
    .badge {
      display: inline-flex;
      align-items: center;
      gap: var(--bdg-gap);
      min-height: var(--bdg-h);
      padding: 0 var(--bdg-px);
      font-family: var(--font-body);
      font-size: var(--bdg-fs);
      font-weight: var(--font-weight-semibold);
      line-height: 1;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      border-radius: var(--bdg-radius);
      /* Base styling — acts as the soft variant so any unknown or missing
       * variant attribute still renders a legible pill. Known variants
       * below re-declare these three properties to override. */
      background: color-mix(in oklch, var(--tone) 16%, transparent);
      border: 1px solid color-mix(in oklch, var(--tone) 24%, transparent);
      color: var(--tone-fg);
      white-space: nowrap;
      -webkit-font-smoothing: antialiased;
      font-variant-numeric: tabular-nums;
    }

    /* ------------------------------------------------------------------
     * Variant matrix. Reads only from --tone / --tone-contrast so the
     * matrix stays DRY across tones. Default (no variant attr) is the
     * soft look — matches the pre-polish badge and is the context
     * where badges sit most often (inline row metadata).
     * ------------------------------------------------------------------ */

    /* soft — tone-tinted surface, faint border, tone text. Default is
     * already applied by .badge above; this explicit selector exists
     * so callers that pass variant=soft target the same rules as
     * variant-less badges. */
    :host([variant='soft']) .badge {
      background: color-mix(in oklch, var(--tone) 16%, transparent);
      border-color: color-mix(in oklch, var(--tone) 24%, transparent);
      color: var(--tone-fg);
    }

    /* solid — filled tone, contrast text, subtle inset highlight for
     * the premium feel (same recipe as ship-button solid). */
    :host([variant='solid']) .badge {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
    }
    :host([variant='solid'][tone='muted']) .badge,
    :host([variant='solid']:not([tone])) .badge {
      background: var(--muted-foreground);
      border-color: color-mix(in oklch, var(--muted-foreground) 100%, black 6%);
      color: var(--background);
    }

    /* outline — transparent with tone border + tone text. The slightly
     * deeper border keeps the chip legible against textured backgrounds. */
    :host([variant='outline']) .badge {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 50%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']:not([tone])) .badge,
    :host([variant='outline'][tone='muted']) .badge {
      border-color: var(--border);
      color: var(--muted-foreground);
    }

    /* ------------------------------------------------------------------
     * Leading slot — icon / status dot. Sized from the icon channel so
     * the glyph always matches the label's cap-height optically. Uses
     * currentColor so an <ship-status-dot> or <ship-icon> tints with
     * the badge's tone-fg without extra wiring.
     * ------------------------------------------------------------------ */
    .leading {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: var(--bdg-icon);
      height: var(--bdg-icon);
      flex-shrink: 0;
      color: currentColor;
    }
    /* Hide leading wrapper when no content is slotted into it.
     * :empty doesn't work because <slot> is always a child. Use
     * :not(:has(::slotted(*))) for modern browsers. */
    .leading { display: none; }
    .leading:has(::slotted(*)) { display: inline-flex; }
    ::slotted([slot='leading']) {
      display: inline-flex;
      align-items: center;
      color: currentColor;
      width: 100%;
      height: 100%;
    }

    /* Forced-colors — fall back to system colors so high-contrast
     * modes render the badge border reliably. */
    @media (forced-colors: active) {
      .badge { border-color: CanvasText; color: CanvasText; }
      :host([variant='solid']) .badge {
        background: Highlight;
        color: HighlightText;
        border-color: Highlight;
      }
    }
  `,
]
