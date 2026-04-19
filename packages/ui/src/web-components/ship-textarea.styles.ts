/**
 * Styles for <ship-textarea>.
 *
 * Shares the visual language of <ship-input> — same tone/size channel
 * architecture, same focus ring recipe, same hover/disabled treatment.
 * Lives in its own file to respect the 300-line discipline cap on
 * ship-textarea.ts and to let the two components evolve in lockstep
 * without duplicating rules inline.
 */

import { css } from 'lit'
import { MONO, reducedMotion } from './util'

export const shipTextareaStyles = [reducedMotion, css`
  :host {
    display: block;
    width: 100%;
    --tone: var(--ring-color);
    --tone-fg: var(--foreground);
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
    --ta-px: 12px;
    --ta-py: 10px;
    --ta-fs: var(--text-base);
    --ta-lh: 1.5;
    --ta-radius: var(--radius-md);
  }
  :host([size='sm']) {
    --ta-px: 10px;
    --ta-py: 8px;
    --ta-fs: var(--text-sm);
    --ta-lh: 1.45;
    --ta-radius: var(--radius-sm);
  }

  textarea {
    display: block;
    width: 100%;
    box-sizing: border-box;
    border: 1px solid var(--border);
    border-radius: var(--ta-radius);
    padding: var(--ta-py) var(--ta-px);
    background: var(--background);
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--ta-fs);
    line-height: var(--ta-lh);
    letter-spacing: -0.005em;
    box-shadow: var(--shadow-xs);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  :host(:not([disabled])) textarea:hover {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  textarea:focus-visible,
  textarea:focus {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
  }

  :host([invalid]) textarea {
    border-color: color-mix(in oklch, var(--destructive) 65%, var(--border));
  }
  :host([invalid]) textarea:hover {
    border-color: color-mix(in oklch, var(--destructive) 80%, var(--border));
  }
  :host([invalid]) textarea:focus-visible,
  :host([invalid]) textarea:focus {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }

  textarea::placeholder {
    color: var(--muted-foreground);
    opacity: 1;
  }
  textarea:disabled {
    cursor: not-allowed;
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    color: var(--muted-foreground);
    box-shadow: none;
    opacity: 0.6;
  }
  textarea:read-only { cursor: default; }

  :host([variant='code']) textarea {
    font-family: ${MONO};
    font-size: var(--text-base);
    line-height: 1.55;
    letter-spacing: 0;
    tab-size: 2;
  }

  @media (forced-colors: active) {
    textarea:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`]
