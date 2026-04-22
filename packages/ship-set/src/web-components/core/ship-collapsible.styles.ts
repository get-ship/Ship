/**
 * Styles for <ship-collapsible>.
 *
 * Smooth height animation on expand/collapse using CSS grid trick
 * (grid-template-rows: 0fr -> 1fr). Respects prefers-reduced-motion.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipCollapsibleStyles = [
  reducedMotion,
  css`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--foreground);
    }
    :host([hidden]) { display: none; }

    .content-wrap {
      display: grid;
      grid-template-rows: 0fr;
      transition: grid-template-rows var(--duration-medium) var(--ease-out);
    }
    :host([open]) .content-wrap {
      grid-template-rows: 1fr;
    }

    .content-inner {
      overflow: hidden;
    }

    ::slotted([slot='trigger']) {
      cursor: pointer;
      user-select: none;
      -webkit-user-select: none;
    }
    :host([disabled]) ::slotted([slot='trigger']) {
      cursor: not-allowed;
      opacity: 0.5;
    }
  `,
]
