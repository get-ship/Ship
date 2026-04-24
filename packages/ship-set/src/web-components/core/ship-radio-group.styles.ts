/**
 * Styles for <ship-radio-group> and <ship-radio>.
 *
 * Channel-driven tone x size. Circular indicator with dot fill on
 * selection. Matches shadcn/Radix Radio Group visual weight.
 *
 *   --tone — accent color for the selected dot + focus ring, set per
 *     :host([tone=…]) block on the group.
 *   --rd-size / --rd-dot — size channel on each radio.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipRadioGroupStyles = [
  reducedMotion,
  css`
    :host {
      display: flex;
      flex-direction: column;
      gap: var(--space-2);
      font-family: var(--font-body);
      color: var(--foreground);
    }
    :host([hidden]) { display: none; }
  `,
]

export const shipRadioStyles = [
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

      --tone: var(--primary);
    }
    :host([hidden]) { display: none; }

    /* Tone channel — inherited from group or set directly */
    :host(:not([tone])),
    :host([tone='default']) { --tone: var(--primary); }
    :host([tone='info'])        { --tone: var(--info); }
    :host([tone='success'])     { --tone: var(--success); }
    :host([tone='warning'])     { --tone: var(--warning); }
    :host([tone='destructive']) { --tone: var(--destructive); }
    :host([tone='accent'])      { --tone: var(--accent); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); }

    /* Size channel */
    :host(:not([size])),
    :host([size='md']) {
      --rd-size: 16px;
      --rd-dot: 8px;
    }
    :host([size='sm']) {
      --rd-size: 14px;
      --rd-dot: 6px;
    }
    :host([size='lg']) {
      --rd-size: 20px;
      --rd-dot: 10px;
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

    /* Circle */
    .circle {
      position: relative;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      width: var(--rd-size);
      height: var(--rd-size);
      background: var(--background);
      border: 1px solid color-mix(in oklch, var(--border) 160%, transparent);
      border-radius: 50%;
      box-sizing: border-box;
      transition:
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }

    /* Hover */
    :host(:not([disabled]):not([checked])) label:hover .circle {
      border-color: color-mix(in oklch, var(--border) 220%, transparent);
    }

    /* Checked — border takes tone color */
    :host([checked]) .circle {
      border-color: var(--tone);
    }

    /* Dot */
    .dot {
      width: var(--rd-dot);
      height: var(--rd-dot);
      border-radius: 50%;
      background: var(--tone);
      opacity: 0;
      transform: scale(0);
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([checked]) .dot {
      opacity: 1;
      transform: scale(1);
    }

    /* Native input — visually hidden */
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

    /* Focus ring */
    :host(:not([disabled])) label:has(input:focus-visible) .circle {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    @media (forced-colors: active) {
      .circle { border-color: CanvasText; }
      :host([checked]) .dot { background: Highlight; }
      :host(:not([disabled])) label:has(input:focus-visible) .circle {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }

    /* Disabled */
    :host([disabled]) {
      color: var(--muted-foreground);
    }
    :host([disabled]) .circle {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 70%, transparent);
      filter: saturate(0.6);
    }
    :host([disabled][checked]) .circle {
      border-color: color-mix(in oklch, var(--tone) 50%, var(--border));
    }
    :host([disabled][checked]) .dot {
      background: color-mix(in oklch, var(--tone) 55%, var(--muted));
    }

    ::slotted(*) {
      font: inherit;
      color: inherit;
      line-height: var(--leading-tight);
    }
  `,
]
