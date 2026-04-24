/**
 * Styles for <ship-slider>.
 *
 * Channel-driven tone x size. Horizontal track with filled portion and
 * a draggable thumb. Matches shadcn/Radix Slider visual weight.
 *
 *   --tone — accent color for the filled track + thumb, set per
 *     :host([tone=…]) block.
 *   --sl-h / --sl-thumb — size channel.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipSliderStyles = [
  reducedMotion,
  css`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--foreground);
      -webkit-tap-highlight-color: transparent;
      --tone: var(--primary);
    }
    :host([hidden]) { display: none; }

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
      --sl-h: 6px;
      --sl-thumb: 16px;
    }
    :host([size='sm']) {
      --sl-h: 4px;
      --sl-thumb: 12px;
    }
    :host([size='lg']) {
      --sl-h: 8px;
      --sl-thumb: 20px;
    }

    .container {
      position: relative;
      display: flex;
      align-items: center;
      width: 100%;
      height: var(--sl-thumb);
      cursor: pointer;
      touch-action: none;
    }
    :host([disabled]) .container {
      cursor: not-allowed;
      opacity: 0.5;
    }

    /* Track */
    .track {
      position: relative;
      width: 100%;
      height: var(--sl-h);
      background: var(--muted);
      border-radius: var(--radius-full);
      overflow: hidden;
    }

    /* Filled portion */
    .fill {
      position: absolute;
      top: 0;
      left: 0;
      height: 100%;
      background: var(--tone);
      border-radius: inherit;
      transition: width 0ms;
    }

    /* Thumb */
    .thumb {
      position: absolute;
      top: 50%;
      width: var(--sl-thumb);
      height: var(--sl-thumb);
      background: var(--background);
      border: 2px solid var(--tone);
      border-radius: 50%;
      box-shadow: var(--shadow-sm);
      transform: translate(-50%, -50%);
      transition:
        box-shadow var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
      box-sizing: border-box;
    }
    .thumb:hover {
      box-shadow: var(--shadow-md);
      transform: translate(-50%, -50%) scale(1.05);
    }
    .thumb:active {
      transform: translate(-50%, -50%) scale(0.95);
    }

    /* Focus ring */
    :host(:not([disabled])) .thumb:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    @media (forced-colors: active) {
      .thumb:focus-visible {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
      .thumb { border-color: CanvasText; }
      .fill { background: Highlight; }
    }
  `,
]
