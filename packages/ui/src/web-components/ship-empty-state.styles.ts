/**
 * Styles for <ship-empty-state>.
 *
 * Extracted from ship-empty-state.ts to keep the component file under the
 * 300-line discipline cap and to share the visual language of the other
 * polished primitives (ship-button, ship-card, ship-input, ship-switch).
 *
 * Architecture — every selector below reads from one of:
 *
 *   --tone / --tone-fg / --tone-bg / --tone-border
 *     Applied via `toneVars` from util. Drives icon tint + focus chrome
 *     without per-variant rewrites. Adding a tone is one util-level change.
 *
 *   --es-pad-y / --es-pad-x / --es-gap
 *   --es-icon / --es-icon-pad / --es-title-fs / --es-desc-fs
 *     Size channel driven by `size=[sm|md|lg]` (default md). `md` reproduces
 *     the pre-polish 64px × 16px padding + 36px icon so every current caller
 *     keeps its exact visual weight.
 *
 *   --es-radius / --es-border / --es-bg
 *     Chrome channel set once per :host([variant=…]). Variant rules never
 *     touch padding / typography — only surface.
 *
 * Variants
 *   default      dashed border + card surface (the classic empty card)
 *   inline       horizontal compact row — icon beside title/description, no
 *                border, tight padding. Used for list-within-list empties
 *                or toolbar sections.
 *   placeholder  no border, transparent, softer — "nothing selected" vs the
 *                louder "no items at all" of default.
 *   connecting   kept for backward compatibility with existing SSE templates.
 *                Behaves like placeholder but sets aria-live=polite so screen
 *                readers announce the streaming state.
 *
 * The icon tile picks up a circular muted wash when shown at md/lg; at sm
 * (and on inline) it renders inline without the tile so it doesn't overpower
 * dense surrounding chrome. Fade-in on mount matches ship-alert's entry
 * transition; reduced-motion collapses it via the shared util mixin.
 */

import { css } from 'lit'
import { reducedMotion, toneVars } from './util'

export const shipEmptyStateStyles = [
  reducedMotion,
  toneVars,
  css`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--muted-foreground);

      /* Size channel — overridden per :host([size=…]) below. md is the
       * default and matches the pre-polish 64px/16px card so every existing
       * caller keeps the same visual weight. --es-icon sizes the inner
       * glyph; --es-tile sizes the circular wrapper around it. Keeping
       * those on separate channels means the tile can stay generous while
       * the glyph itself reads at the same optical size as the copy. */
      --es-pad-y: 64px;
      --es-pad-x: 16px;
      --es-gap: var(--space-3);
      --es-icon: 22px;
      --es-tile: 48px;
      --es-title-fs: var(--text-md);
      --es-desc-fs: var(--text-sm);
      --es-title-weight: var(--font-weight-semibold);

      /* Variant channel — default is the dashed card; inline/placeholder/
       * connecting override below. */
      --es-radius: var(--radius-lg);
      --es-border: 1px dashed var(--border);
      --es-bg: var(--card);
    }
    :host([hidden]) { display: none; }

    /* ------------------------------------------------------------------
     * Size channel. Padding scales roughly 1.4× between steps so lg feels
     * hero-sized without dwarfing its container; sm tightens to fit list
     * sub-sections. Icon + type sizes travel together so the composition
     * stays optically balanced across sizes.
     * ------------------------------------------------------------------ */
    :host([size='sm']) {
      --es-pad-y: var(--space-7);
      --es-pad-x: var(--space-4);
      --es-gap: var(--space-2);
      --es-icon: 16px;
      --es-tile: 36px;
      --es-title-fs: var(--text-base);
      --es-desc-fs: var(--text-sm);
    }
    :host([size='lg']) {
      --es-pad-y: 96px;
      --es-pad-x: var(--space-6);
      --es-gap: var(--space-4);
      --es-icon: 28px;
      --es-tile: 64px;
      --es-title-fs: var(--text-lg);
      --es-desc-fs: var(--text-base);
    }

    /* ------------------------------------------------------------------
     * Variant channel. Every variant only touches surface + radius +
     * border + internal padding density. Typography and icon sizing stay
     * controlled by the size channel, so size=lg variant=placeholder and
     * size=lg variant=default share the exact same type scale.
     * ------------------------------------------------------------------ */
    :host([variant='placeholder']),
    :host([variant='connecting']) {
      --es-border: 0;
      --es-bg: transparent;
      --es-radius: 0;
      --es-pad-y: var(--space-6);
    }
    :host([variant='placeholder'][size='sm']),
    :host([variant='connecting'][size='sm']) {
      --es-pad-y: var(--space-4);
    }
    :host([variant='placeholder'][size='lg']),
    :host([variant='connecting'][size='lg']) {
      --es-pad-y: var(--space-8);
    }

    :host([variant='inline']) {
      --es-border: 1px dashed var(--border);
      --es-bg: color-mix(in oklch, var(--muted) 55%, transparent);
      --es-radius: var(--radius-md);
      --es-pad-y: var(--space-3);
      --es-pad-x: var(--space-4);
      --es-gap: var(--space-3);
    }

    /* ------------------------------------------------------------------
     * Box — the outer chrome. Flex column by default; inline flips to a
     * horizontal row with left-aligned text so it slots cleanly into list
     * sub-sections without forcing a full-bleed empty card.
     * ------------------------------------------------------------------ */
    .box {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: var(--es-gap);
      padding: var(--es-pad-y) var(--es-pad-x);
      text-align: center;
      background: var(--es-bg);
      border: var(--es-border);
      border-radius: var(--es-radius);
      animation: es-in var(--duration-medium) var(--ease-out) both;
    }
    :host([variant='inline']) .box {
      flex-direction: row;
      align-items: center;
      justify-content: flex-start;
      text-align: left;
      gap: var(--space-3);
    }

    @keyframes es-in {
      from { opacity: 0; transform: translateY(4px); }
      to   { opacity: 1; transform: translateY(0); }
    }

    /* ------------------------------------------------------------------
     * Icon tile. The tile is a muted circle wrapping the glyph — reads
     * as deliberate, not just "missing content". sm uses a 36px tile,
     * md 48px, lg 64px. Inline variant drops the tile so the icon sits
     * bare beside the title in dense row chrome.
     * ------------------------------------------------------------------ */
    .icon {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      width: var(--es-tile);
      height: var(--es-tile);
      border-radius: var(--radius-full);
      /* Tile wash reads deeper than a plain muted surface so the tone is
       * legible without overpowering the surrounding card. Neutral tones
       * fall back to plain muted via the explicit selector below. */
      background: color-mix(in oklch, var(--tone-fg) 18%, var(--muted));
      color: var(--tone-fg);
      line-height: 1;
    }
    :host(:not([tone])) .icon,
    :host([tone='default']) .icon {
      background: var(--muted);
      color: var(--muted-foreground);
    }
    .icon:empty,
    .icon[data-empty] { display: none; }
    /* Inline compact row drops the tile — icon sits bare at the icon-channel
     * size so dense row chrome doesn't get a full-height circle. */
    :host([variant='inline']) .icon {
      width: var(--es-icon);
      height: var(--es-icon);
      background: transparent;
    }
    /* Built-in glyph — inline SVG rendered by the component when the
     * icon attr is set and no slot content is present. Sized off the
     * icon channel so it scales with size=[sm|md|lg]. */
    .glyph {
      width: var(--es-icon);
      height: var(--es-icon);
      display: block;
    }
    /* Slotted glyphs get a display + color nudge. We can't reliably set
     * width/height on a slotted <ship-icon> (its internal SVG is pinned
     * to ship-icon's own size attr), so callers using ship-icon as a
     * slot should set size="lg" when pairing with an md/lg empty state.
     * Raw <svg slot="icon"> scales to the icon channel via the direct
     * selector below. */
    ::slotted([slot='icon']) {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      color: currentColor;
    }
    ::slotted(svg[slot='icon']) {
      width: var(--es-icon);
      height: var(--es-icon);
    }

    /* ------------------------------------------------------------------
     * Typography. Title sits foreground-weight, description picks up
     * muted-foreground + an ~40ch max-width so long copy wraps at a
     * comfortable reading measure rather than stretching to the card.
     * ------------------------------------------------------------------ */
    .copy {
      display: flex;
      flex-direction: column;
      gap: 4px;
      align-items: center;
      min-width: 0;
    }
    :host([variant='inline']) .copy {
      align-items: flex-start;
    }
    .title {
      font-size: var(--es-title-fs);
      font-weight: var(--es-title-weight);
      color: var(--foreground);
      line-height: var(--leading-tight);
      letter-spacing: -0.005em;
    }
    .title:empty { display: none; }

    .desc {
      font-size: var(--es-desc-fs);
      line-height: var(--leading-normal);
      max-width: 40ch;
      color: var(--muted-foreground);
    }
    :host([variant='inline']) .desc {
      max-width: 60ch;
    }
    .desc:empty { display: none; }

    /* connecting / placeholder soften the title weight slightly so these
     * quieter states don't compete with actual content headers. */
    :host([variant='connecting']) .title,
    :host([variant='placeholder']) .title {
      font-weight: var(--font-weight-medium);
      color: var(--muted-foreground);
    }

    /* ------------------------------------------------------------------
     * Actions row. Centered below the copy; on inline, pushed to the
     * far right of the row so a CTA slots naturally on the trailing edge.
     * ------------------------------------------------------------------ */
    .actions {
      display: flex;
      gap: var(--space-2);
      justify-content: center;
      margin-top: 4px;
      flex-wrap: wrap;
    }
    .actions:empty { display: none; }
    :host([variant='inline']) .actions {
      margin-top: 0;
      margin-left: auto;
      justify-content: flex-end;
    }

    /* Forced-colors — keep the dashed border visible in high-contrast
     * modes where color-mix'd tokens collapse. */
    @media (forced-colors: active) {
      .box { border-color: CanvasText; }
      .icon { background: Canvas; color: CanvasText; }
    }
  `,
]
