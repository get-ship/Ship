/**
 * Styles for <ship-accordion> and <ship-accordion-item>.
 *
 * Smooth height animation via CSS grid trick. Separator borders between
 * items. Chevron rotates on expand. Focus ring on trigger buttons.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipAccordionStyles = [
  reducedMotion,
  css`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--foreground);
    }
    :host([hidden]) { display: none; }
  `,
]

export const shipAccordionItemStyles = [
  reducedMotion,
  css`
    :host {
      display: block;
      border-bottom: 1px solid var(--border);
    }
    :host([hidden]) { display: none; }

    .trigger {
      display: flex;
      align-items: center;
      justify-content: space-between;
      width: 100%;
      padding: var(--space-4) 0;
      background: none;
      border: none;
      font: inherit;
      font-size: var(--text-base);
      font-weight: 500;
      color: var(--foreground);
      cursor: pointer;
      text-align: left;
      line-height: var(--leading-normal);
      transition: color var(--duration-fast) var(--ease-out);
    }
    .trigger:hover {
      text-decoration: underline;
      text-underline-offset: 4px;
    }

    /* Focus ring */
    .trigger:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
      border-radius: var(--radius-sm);
    }
    @media (forced-colors: active) {
      .trigger:focus-visible {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }

    :host([disabled]) .trigger {
      cursor: not-allowed;
      color: var(--muted-foreground);
    }
    :host([disabled]) .trigger:hover {
      text-decoration: none;
    }

    /* Chevron */
    .chevron {
      flex-shrink: 0;
      width: 16px;
      height: 16px;
      color: var(--muted-foreground);
      transition: transform var(--duration-medium) var(--ease-out);
    }
    :host([open]) .chevron {
      transform: rotate(180deg);
    }

    /* Content area — CSS grid height animation */
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
    .content-body {
      padding-bottom: var(--space-4);
      font-size: var(--text-sm);
      color: var(--muted-foreground);
      line-height: var(--leading-relaxed);
    }
  `,
]
