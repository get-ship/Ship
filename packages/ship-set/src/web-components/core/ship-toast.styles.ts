/**
 * Styles for <ship-toast>. Channel-based architecture shared with the other
 * polished primitives (ship-card, ship-dialog, ship-button):
 *   --tone-*    — set once via toneVars; consumed by surface/rail/progress.
 *   --toast-*   — size channel for px/py/fs/gap/min-w/max-w.
 *   Entrance animation uses the --toast-slide-x/y channel so the direction
 *   adapts to the parent container's position (data-slide-from). Exit is
 *   opt-in via [data-closing] so the sync .remove() path stays synchronous
 *   for existing unit tests.
 */

import { css } from 'lit'
import { reducedMotion, toneVars } from './util'

export const shipToastStyles = [
  reducedMotion,
  toneVars,
  css`
    :host {
      display: block;
      /* Size channel defaults (= md). Overridden per :host([size=…]) below. */
      --toast-px: 14px;
      --toast-py: 12px;
      --toast-fs: var(--text-base);
      --toast-title-fs: var(--text-base);
      --toast-gap: 10px;
      --toast-min-w: 280px;
      --toast-max-w: 420px;
      /* Default slide direction (right edge). Container sets
       * data-slide-from so positions on the left/top slide in from the
       * correct edge. */
      --toast-slide-x: 16px;
      --toast-slide-y: 0;
      pointer-events: auto;
      will-change: transform, opacity, max-height, margin;
    }
    :host([hidden]) { display: none; }

    :host([size='sm']) {
      --toast-px: 12px;
      --toast-py: 10px;
      --toast-fs: var(--text-sm);
      --toast-title-fs: var(--text-sm);
      --toast-gap: 8px;
      --toast-min-w: 240px;
      --toast-max-w: 360px;
    }
    :host([size='lg']) {
      --toast-px: 16px;
      --toast-py: 14px;
      --toast-fs: var(--text-md);
      --toast-title-fs: var(--text-md);
      --toast-gap: 12px;
      --toast-min-w: 320px;
      --toast-max-w: 460px;
    }

    /* Container tells us which edge to slide from via data-slide-from.
     * Matches the six container positions with a small set of transforms. */
    :host([data-slide-from='right'])  { --toast-slide-x: 16px;  --toast-slide-y: 0; }
    :host([data-slide-from='left'])   { --toast-slide-x: -16px; --toast-slide-y: 0; }
    :host([data-slide-from='top'])    { --toast-slide-x: 0;     --toast-slide-y: -12px; }
    :host([data-slide-from='bottom']) { --toast-slide-x: 0;     --toast-slide-y: 12px; }

    /* Entrance animation — runs once on mount. Uses the slide channel so
     * position-aware direction comes for free. */
    @keyframes ship-toast-in {
      from {
        opacity: 0;
        transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0)
          scale(0.98);
      }
      to {
        opacity: 1;
        transform: translate3d(0, 0, 0) scale(1);
      }
    }

    .toast {
      display: grid;
      grid-template-columns: auto 1fr auto;
      column-gap: var(--toast-gap);
      row-gap: 4px;
      align-items: start;
      padding: var(--toast-py) var(--toast-px);
      padding-right: calc(var(--toast-px) + 24px);
      min-width: var(--toast-min-w);
      max-width: var(--toast-max-w);
      border-radius: var(--radius-md);
      border: 1px solid var(--tone-border);
      background: var(--card);
      color: var(--card-foreground);
      box-shadow:
        0 0 0 1px color-mix(in oklch, black 3%, transparent),
        var(--shadow-md);
      font-family: var(--font-body);
      font-size: var(--toast-fs);
      line-height: var(--leading-normal);
      position: relative;
      overflow: hidden;
      isolation: isolate;
      animation: ship-toast-in var(--duration-medium) var(--ease-out) both;
    }

    /* Default (no tone) keeps the neutral card surface but drops a subtle
     * 2px accent rail on the leading edge so the toast reads as a card,
     * not a sticky note. */
    :host(:not([tone])) .toast,
    :host([tone='default']) .toast {
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
    }

    /* Tone-tinted variants — blend the tone wash with the card surface so
     * text keeps its contrast. The rail (::before) renders the tone at full
     * saturation for a crisp visual anchor. One combined selector covers
     * every semantic tone; the muted/default tones keep the neutral card. */
    :host([tone='info']) .toast, :host([tone='success']) .toast,
    :host([tone='warning']) .toast, :host([tone='destructive']) .toast,
    :host([tone='accent']) .toast {
      background: color-mix(in oklch, var(--tone-bg) 70%, var(--card));
      border-color: var(--tone-border);
    }

    /* Leading accent rail — 3px tinted stripe. Renders as ::before so the
     * rail follows the border-radius automatically. */
    .toast::before {
      content: '';
      position: absolute;
      inset-inline-start: 0;
      top: 0; bottom: 0;
      width: 3px;
      background: var(--tone-fg);
      opacity: 0;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    :host([tone='info']) .toast::before, :host([tone='success']) .toast::before,
    :host([tone='warning']) .toast::before, :host([tone='destructive']) .toast::before,
    :host([tone='accent']) .toast::before { opacity: 1; }

    /* Icon slot — sits top-left, sized to harmonize with the title line. */
    .icon {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 18px;
      height: 18px;
      flex: 0 0 auto;
      color: var(--tone-fg);
      margin-top: 1px;
    }
    :host(:not([tone])) .icon,
    :host([tone='default']) .icon { color: var(--muted-foreground); }
    .icon svg { width: 18px; height: 18px; display: block; }
    :host([size='sm']) .icon,
    :host([size='sm']) .icon svg { width: 16px; height: 16px; }
    :host([size='lg']) .icon,
    :host([size='lg']) .icon svg { width: 20px; height: 20px; }

    /* When the consumer doesn't slot an icon and we have no tone icon to
     * auto-fill (tone=default without data-has-icon), collapse the column
     * so the content flows to the leading edge. */
    :host([data-no-icon]) .toast {
      grid-template-columns: 1fr auto;
    }
    :host([data-no-icon]) .icon { display: none; }

    .content {
      display: flex;
      flex-direction: column;
      gap: 2px;
      min-width: 0;
      color: var(--card-foreground);
    }
    .title {
      font-weight: var(--font-weight-semibold);
      font-size: var(--toast-title-fs);
      letter-spacing: -0.005em;
      line-height: var(--leading-tight);
      color: var(--foreground);
    }
    .body {
      font-size: var(--toast-fs);
      color: var(--muted-foreground);
      line-height: var(--leading-normal);
      word-break: break-word;
    }
    /* When there's no title the body promotes to the stronger color so a
     * single-line toast still reads as the primary message. */
    :host([data-no-title]) .body { color: var(--card-foreground); }
    :host([data-no-title]) .title { display: none; }

    .actions {
      display: flex;
      gap: var(--space-2);
      margin-top: 6px;
      grid-column: 2;
    }
    :host([data-no-actions]) .actions { display: none; margin-top: 0; }

    /* Close button — top-right, ghost affordance that picks up tone on
     * hover. Matches the ship-dialog close in size and motion. */
    .close {
      position: absolute;
      top: 8px;
      inset-inline-end: 8px;
      width: 22px;
      height: 22px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--radius-sm);
      color: var(--muted-foreground);
      cursor: pointer;
      padding: 0;
      font: inherit;
      z-index: 1;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    .close:hover {
      background: color-mix(in oklch, var(--foreground) 8%, transparent);
      color: var(--foreground);
    }
    .close:active { transform: scale(0.94); }
    .close:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--card),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    .close svg { width: 12px; height: 12px; display: block; }
    :host(:not([dismissible])) .close { display: none; }

    /* Progress bar — 2px strip at the bottom that counts down from full to
     * empty over the configured duration. Opt-in via [show-progress].
     * Paused on host hover (CSS) and host-level data-paused (JS driven). */
    .progress {
      position: absolute;
      left: 0;
      right: 0;
      bottom: 0;
      height: 2px;
      background: var(--tone-fg);
      transform: scaleX(1);
      transform-origin: left center;
      animation: ship-toast-progress var(--toast-duration, 5000ms) linear forwards;
    }
    :host(:not([tone])) .progress,
    :host([tone='default']) .progress { background: var(--muted-foreground); }
    :host(:hover) .progress,
    :host([data-paused]) .progress { animation-play-state: paused; }
    @keyframes ship-toast-progress {
      from { transform: scaleX(1); }
      to   { transform: scaleX(0); }
    }
    :host(:not([show-progress])) .progress { display: none; }

    /* Closing state — opt-in exit animation. Consumers invoke .close() to
     * fade + collapse the toast before final removal. The sync dismiss()
     * path skips this so existing unit tests keep their synchronous
     * isConnected === false expectation. */
    :host([data-closing]) {
      animation: ship-toast-out var(--duration-medium) var(--ease-in) forwards;
    }
    @keyframes ship-toast-out {
      to {
        opacity: 0;
        transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0)
          scale(0.96);
      }
    }

    /* Forced-colors — restore a visible edge when color-mix is flattened. */
    @media (forced-colors: active) {
      .toast { border: 1px solid CanvasText; }
      .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
    }
  `,
]
