/**
 * Styles for <ship-dialog>. Split out of ship-dialog.ts to keep the component
 * class under the 300-line ceiling while preserving the detail needed to
 * match shadcn/ui + Radix Dialog polish (size channel × backdrop × states).
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipDialogStyles = [reducedMotion, css`
  :host { display: contents; }
  :host([hidden]) { display: none; }

  /* Size channel — one source of truth for max-width + padding. Adding a
   * size = one :host([size=…]) block below. */
  :host(:not([size])),
  :host([size='md']) {
    --dlg-max-w: 500px;
    --dlg-px: 24px;
    --dlg-header-py: 18px;
    --dlg-body-py: 16px;
    --dlg-footer-py: 16px;
    --dlg-title-fs: var(--text-lg);
    --dlg-desc-fs: var(--text-base);
  }
  :host([size='sm']) {
    --dlg-max-w: 400px;
    --dlg-px: 20px;
    --dlg-header-py: 16px;
    --dlg-body-py: 14px;
    --dlg-footer-py: 14px;
    --dlg-title-fs: var(--text-md);
    --dlg-desc-fs: var(--text-sm);
  }
  :host([size='lg']) {
    --dlg-max-w: 700px;
    --dlg-px: 28px;
    --dlg-header-py: 20px;
    --dlg-body-py: 20px;
    --dlg-footer-py: 18px;
    --dlg-title-fs: var(--text-xl);
    --dlg-desc-fs: var(--text-base);
  }
  :host([size='full']) {
    --dlg-max-w: 90vw;
    --dlg-px: 28px;
    --dlg-header-py: 20px;
    --dlg-body-py: 20px;
    --dlg-footer-py: 18px;
    --dlg-title-fs: var(--text-xl);
    --dlg-desc-fs: var(--text-base);
  }

  /* The native <dialog>. Centered via dialog's default modal positioning
   * (margin:auto in UA sheet). We control the visual: surface, shadow,
   * rounded corners, and an entrance animation that respects reduced-motion
   * via the reducedMotion stylesheet above. */
  dialog {
    position: fixed;
    inset: 0;
    margin: auto;
    padding: 0;
    border: 1px solid color-mix(in oklch, var(--border) 140%, transparent);
    border-radius: var(--radius-lg);
    background: var(--card);
    color: var(--card-foreground);
    width: calc(100vw - 32px);
    max-width: var(--dlg-max-w);
    max-height: min(calc(100vh - 32px), 85vh);
    box-shadow:
      0 0 0 1px color-mix(in oklch, black 4%, transparent),
      var(--shadow-xl);
    backdrop-filter: var(--card-backdrop-filter);
    -webkit-backdrop-filter: var(--card-backdrop-filter);
    font-family: var(--font-body);
    color-scheme: light dark;
    overflow: visible;
    /* Start state — entrance runs on [open]. */
    opacity: 0;
    transform: translateY(8px) scale(0.96);
    transition:
      opacity var(--duration-medium) var(--ease-out),
      transform var(--duration-medium) var(--ease-out);
  }
  :host([size='full']) dialog {
    max-height: 90vh;
  }
  dialog[open] {
    opacity: 1;
    transform: translateY(0) scale(1);
  }

  /* Backdrop — semi-transparent overlay. Default variant has a subtle tint,
   * the 'blur' variant adds a backdrop-filter for a premium feel. Both fade
   * in with the dialog. */
  dialog::backdrop {
    background: var(--backdrop);
    opacity: 0;
    transition: opacity var(--duration-medium) var(--ease-out);
  }
  dialog[open]::backdrop {
    opacity: 1;
  }
  :host([backdrop='blur']) dialog::backdrop {
    background: var(--backdrop);
    backdrop-filter: blur(6px) saturate(1.1);
    -webkit-backdrop-filter: blur(6px) saturate(1.1);
  }
  :host([backdrop='solid']) dialog::backdrop {
    background: var(--backdrop);
  }

  /* Focus-trap sentinels — invisible, zero-size, but keyboard-tabbable.
   * Placed before and after .wrap so Tab at either edge bounces to the
   * other end via the sentinel focus handler. */
  .sentinel {
    position: fixed;
    width: 1px;
    height: 1px;
    pointer-events: none;
    opacity: 0;
  }

  /* Structural wrapper — flex column so the body area can scroll while the
   * header/footer stay pinned. */
  .wrap {
    display: flex;
    flex-direction: column;
    max-height: inherit;
    position: relative;
    isolation: isolate;
  }

  /* Header — title + optional description. Bordered from body so long
   * content below gets a visible scroll anchor. */
  .header {
    padding: var(--dlg-header-py) var(--dlg-px);
    padding-right: calc(var(--dlg-px) + 28px);
    border-bottom: 1px solid var(--border-strong);
    font-family: var(--heading-font);
    font-size: var(--dlg-title-fs);
    font-weight: var(--heading-weight);
    letter-spacing: var(--heading-tracking);
    line-height: var(--leading-tight);
    color: var(--foreground);
    flex: 0 0 auto;
  }
  :host([no-header]) .header { display: none; }

  /* Optional description — consumers wrap a second line in
   * <span class="ship-dialog-description">. Because slotted descendants
   * live in light DOM, the class is global and needs no ::slotted plumbing;
   * callers can style it directly or let the tokens.css surface a default.
   * Exposed custom properties let hosts theme from outside. */
  :host {
    --ship-dialog-description-color: var(--muted-foreground);
    --ship-dialog-description-size: var(--dlg-desc-fs);
  }

  .body {
    padding: var(--dlg-body-py) var(--dlg-px);
    overflow-y: auto;
    overscroll-behavior: contain;
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    color: var(--card-foreground);
    flex: 1 1 auto;
    min-height: 0;
    /* Subtle scrollbar — matches shadcn's scroll-area feel. */
    scrollbar-width: thin;
    scrollbar-color: color-mix(in oklch, var(--border) 160%, transparent) transparent;
  }
  .body::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  .body::-webkit-scrollbar-thumb {
    background: color-mix(in oklch, var(--border) 160%, transparent);
    border-radius: var(--radius-full);
  }
  .body::-webkit-scrollbar-track { background: transparent; }

  /* Footer — action row. Hidden when no footer slot content to avoid a
   * dangling empty band at the bottom. */
  .footer {
    padding: var(--dlg-footer-py) var(--dlg-px);
    border-top: 1px solid color-mix(in oklch, var(--border) 120%, transparent);
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
    align-items: center;
    flex: 0 0 auto;
    background: color-mix(in oklch, var(--muted) 25%, var(--card));
    border-bottom-left-radius: var(--radius-lg);
    border-bottom-right-radius: var(--radius-lg);
  }
  :host([no-footer]) .footer { display: none; }

  /* Close button — top-right, ghost styling that picks up tone on hover.
   * Sized so the target is comfortably 28px while the glyph stays crisp. */
  .close {
    position: absolute;
    top: 12px;
    inset-inline-end: 12px;
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 0;
    font: inherit;
    z-index: 1;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  .close:hover {
    background: var(--muted);
    color: var(--foreground);
    transform: scale(1.04);
  }
  .close:active {
    transform: scale(0.96);
  }
  .close:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--card),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  .close svg {
    width: 14px;
    height: 14px;
    display: block;
  }
  :host([no-close-button]) .close { display: none; }

  /* Forced-colors — ring + border legibility in high-contrast mode. */
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
    .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`]
