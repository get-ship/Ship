/**
 * Styles for <ship-select>. Split out of ship-select.ts to keep the
 * component class under the 300-line ceiling while preserving the detail
 * needed to match shadcn/ui polish (tone channel × size channel × states).
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipSelectStyles = [reducedMotion, css`
  :host {
    display: inline-flex;
    vertical-align: middle;
    /* Per-host tone channel — mirrors ship-button's approach so the chrome
     * below can stay tone-agnostic and compose freely. */
    --tone: var(--foreground);
    --tone-fg: var(--foreground);
  }
  :host([hidden]) { display: none; }

  /* Tone channel. Default tone leaves the control neutral; explicit tones
   * colorize the border + focus ring. Foreground text stays readable. */
  :host([tone='info']) {
    --tone: var(--info);
    --tone-fg: var(--info);
  }
  :host([tone='success']) {
    --tone: var(--success);
    --tone-fg: var(--success);
  }
  :host([tone='warning']) {
    --tone: var(--warning);
    --tone-fg: var(--warning);
  }
  :host([tone='destructive']) {
    --tone: var(--destructive);
    --tone-fg: var(--destructive);
  }
  :host([tone='accent']) {
    --tone: var(--accent);
    --tone-fg: var(--accent);
  }
  :host([tone='muted']) {
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
  }

  /* Size channel — one source of truth for height/padding/type scale.
   * Matches ship-button visual weight. Values are hardcoded (not pulled
   * from --space-* tokens) because some runtime hosts ship a tokens.css
   * without the spacing scale, and a select that loses its padding is
   * worse than a select that ignores a token the host didn't define. */
  :host(:not([size])),
  :host([size='md']) {
    --sel-h: 34px;
    --sel-px: 12px;
    --sel-fs: 13px;
    --sel-gap: 8px;
    --sel-chev: 14px;
  }
  :host([size='sm']) {
    --sel-h: 28px;
    --sel-px: 10px;
    --sel-fs: 12px;
    --sel-gap: 6px;
    --sel-chev: 12px;
  }
  :host([size='lg']) {
    --sel-h: 40px;
    --sel-px: 16px;
    --sel-fs: 14px;
    --sel-gap: 8px;
    --sel-chev: 16px;
  }

  /* Field wrapper — this is the visible chrome. The real <select> sits
   * on top, fully transparent, so clicks/keyboard/native popup still work
   * without us re-implementing a menu. */
  .field {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: var(--sel-gap);
    min-height: var(--sel-h);
    padding-inline: var(--sel-px);
    padding-inline-end: calc(var(--sel-px) + var(--sel-chev) + 6px);
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-xs);
    font-family: var(--font-body);
    font-size: var(--sel-fs);
    font-weight: var(--font-weight-medium);
    line-height: 1;
    letter-spacing: -0.005em;
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  /* Hover — subtle muted wash + border crispening, matching ship-button's
   * default (no-variant) treatment so the two sit next to each other
   * without either looking out of place. */
  :host(:not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--muted) 70%, var(--background));
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  /* Toned border when a tone is set (except default). Keeps the surface
   * neutral while still signaling intent on the edge. */
  :host([tone]:not([tone='default'])) .field {
    border-color: color-mix(in oklch, var(--tone) 38%, var(--border));
  }
  :host([tone]:not([tone='default']):not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--tone) 8%, var(--background));
    border-color: color-mix(in oklch, var(--tone) 55%, var(--border));
  }

  /* The real <select> — layered invisibly over the chrome. We use
   * appearance:none to strip OS chrome, color:transparent to hide the
   * rendered glyph, and pointer events stay enabled so the native popup
   * fires. */
  select {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    padding-inline: var(--sel-px);
    padding-inline-end: calc(var(--sel-px) + var(--sel-chev) + 6px);
    font: inherit;
    color: transparent;
    background: transparent;
    border: 0;
    outline: 0;
    appearance: none;
    -webkit-appearance: none;
    cursor: inherit;
  }
  /* Keep the chosen option legible in the native popup (OS-chromed). */
  select option { color: var(--foreground); background: var(--background); }

  /* Visible label layer — mirrors the current value on top of the chrome.
   * This is what the user actually reads; the native <select> underneath
   * provides the menu. Sized to content so the host's intrinsic width
   * tracks the current value (matches shadcn's trigger-hugs-label feel). */
  .value {
    display: inline-block;
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
  }
  :host([data-placeholder]) .value {
    color: var(--muted-foreground);
  }

  /* Chevron — SVG in shadow so it always matches currentColor + size.
   * Positioned absolutely so it never contends with the text layout. */
  .chevron {
    position: absolute;
    inset-block: 0;
    inset-inline-end: var(--sel-px);
    margin-block: auto;
    width: var(--sel-chev);
    height: var(--sel-chev);
    color: var(--muted-foreground);
    pointer-events: none;
    transition:
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-medium) var(--ease-out);
  }
  :host(:not([disabled])) .field:hover .chevron {
    color: var(--foreground);
  }
  :host([tone]:not([tone='default'])) .chevron {
    color: color-mix(in oklch, var(--tone-fg) 75%, var(--muted-foreground));
  }

  /* Focus ring — rendered on the wrapper via stacked box-shadows. The
   * native select sits inside, so :focus-within on the host flips this on
   * reliably whether the user tabs in or clicks. */
  :host(:focus-within) .field {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 60%, var(--border));
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--tone) 55%, transparent);
  }
  :host(:not([tone]):focus-within) .field,
  :host([tone='default']:focus-within) .field {
    border-color: color-mix(in oklch, var(--ring-color) 55%, var(--border));
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }

  /* Invalid — destructive border + destructive ring, overrides tone so
   * validation always reads first. */
  :host([invalid]) .field {
    border-color: var(--destructive);
  }
  :host([invalid]:focus-within) .field {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }

  /* Disabled — muted surface, desaturated, not-allowed. 60% opacity on
   * the whole host matches ship-button's disabled affordance. */
  :host([disabled]) {
    opacity: 0.6;
    cursor: not-allowed;
  }
  :host([disabled]) .field {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 80%, transparent);
    box-shadow: none;
    cursor: not-allowed;
    filter: saturate(0.6);
  }
  :host([disabled]) select {
    cursor: not-allowed;
  }

  /* Forced-colors — make sure the ring is still visible. */
  @media (forced-colors: active) {
    :host(:focus-within) .field { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`]
