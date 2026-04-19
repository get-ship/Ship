/**
 * Styles for <ship-input>.
 *
 * Extracted from the component file to keep ship-input.ts under the 300-line
 * discipline cap. All tone/size/variant logic is channel-driven:
 *   --tone / --tone-fg  — set once per :host([tone=…]) block
 *   --in-h / --in-px / --in-fs / …  — set once per :host([size=…]) block
 * The field + variant + state selectors read only from these channels, so
 * adding a tone or size is a single `:host([…])` addition, not a matrix edit.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipInputStyles = [reducedMotion, css`
  :host {
    display: inline-flex;
    width: 100%;
    --tone: var(--primary);
    --tone-fg: var(--primary);
  }
  :host([hidden]) { display: none; }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--ring-color);
    --tone-fg: var(--foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);     --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) {
    --in-h: 34px;
    --in-px: 10px;
    --in-gap: var(--space-2);
    --in-fs: var(--text-base);
    --in-lh: 22px;
    --in-fw: var(--font-weight-normal);
    --in-icon: 14px;
    --in-radius: var(--radius-md);
  }
  :host([size='sm']) {
    --in-h: 28px;
    --in-px: 8px;
    --in-gap: 6px;
    --in-fs: var(--text-sm);
    --in-lh: 18px;
    --in-fw: var(--font-weight-normal);
    --in-icon: 12px;
    --in-radius: var(--radius-sm);
  }
  :host([size='lg']) {
    --in-h: 40px;
    --in-px: var(--space-3);
    --in-gap: var(--space-2);
    --in-fs: var(--text-lg);
    --in-lh: 24px;
    --in-fw: var(--font-weight-medium);
    --in-icon: 16px;
    --in-radius: var(--radius-md);
  }

  .field {
    display: inline-flex;
    align-items: center;
    gap: var(--in-gap);
    width: 100%;
    min-height: var(--in-h);
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--in-radius);
    padding: 0 var(--in-px);
    box-shadow: var(--shadow-xs);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  /* Hover — subtle border darkening + surface lift. Mirrors shadcn's
   * quiet-input feel: the field acknowledges the pointer without shouting. */
  :host(:not([disabled])) .field:hover {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  /* Focus ring — stacked box-shadow so the ring sits outside the border
   * without reserving layout. Matches ship-button's technique. */
  .field:focus-within {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
  }

  /* Invalid — destructive tint on border + ring. Wins over tone so users
   * can't miss an error state even on info/success-toned inputs. */
  :host([invalid]) .field {
    border-color: color-mix(in oklch, var(--destructive) 65%, var(--border));
  }
  :host([invalid]) .field:hover {
    border-color: color-mix(in oklch, var(--destructive) 80%, var(--border));
  }
  :host([invalid]) .field:focus-within {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }

  /* Ghost — no chrome at rest, materialize on focus. Keeps the title-input
   * pattern (click-to-edit) without the old border-jump. */
  :host([variant='ghost']) .field {
    background: transparent;
    border-color: transparent;
    box-shadow: none;
  }
  :host([variant='ghost']:not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--muted) 50%, transparent);
    border-color: transparent;
    box-shadow: none;
  }
  :host([variant='ghost']) .field:focus-within {
    background: var(--background);
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
  }

  /* Native element — transparent so the field chrome is the surface of
   * truth. font-size is driven off the size channel, never hardcoded. */
  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: inherit;
    font-family: var(--font-body);
    font-size: var(--in-fs);
    font-weight: var(--in-fw);
    line-height: var(--in-lh);
    letter-spacing: -0.005em;
    padding: 0;
  }
  input::placeholder {
    color: var(--muted-foreground);
    opacity: 1;
  }
  input:-webkit-autofill {
    -webkit-box-shadow: 0 0 0 1000px var(--background) inset;
    -webkit-text-fill-color: var(--foreground);
    caret-color: var(--foreground);
  }

  /* Disabled — dim the whole field and kill pointer affordances. */
  :host([disabled]) .field,
  :host([readonly]) .field { cursor: not-allowed; }
  :host([disabled]) .field {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    box-shadow: none;
    opacity: 0.6;
  }
  input:disabled {
    cursor: not-allowed;
    color: var(--muted-foreground);
  }
  input:read-only { cursor: default; }

  /* Loading spinner — sized off the adornment channel so it sits on the
   * same optical line as a slotted icon. */
  .spinner {
    width: var(--in-icon);
    height: var(--in-icon);
    border: 1.5px solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
    flex-shrink: 0;
    color: var(--muted-foreground);
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Slotted adornments — constrain icons to the size channel so a bare
   * <svg> drops in at the right visual weight without caller intervention. */
  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
    color: var(--muted-foreground);
    font-size: var(--in-fs);
    line-height: 1;
  }
  ::slotted(svg) {
    width: var(--in-icon);
    height: var(--in-icon);
    display: block;
  }

  @media (forced-colors: active) {
    .field:focus-within { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`]
