/**
 * Shadow DOM styles for <ship-code-editor>.
 *
 * Keeps the host layout and sizing rules separate from the CodeMirror
 * theme (which is applied inside the editor instance itself).
 */

import { css } from 'lit'
import { MONO } from './util'

export const codeEditorStyles = css`
  :host {
    display: block;
    position: relative;
    font-family: ${MONO};
    font-size: var(--text-base);
    line-height: 1.5;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--card);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  :host(:focus-within) {
    border-color: var(--primary);
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }

  :host([tone='info']:focus-within) {
    border-color: var(--info);
    box-shadow: 0 0 0 var(--ring-width)
      color-mix(in oklch, var(--info) var(--ring-opacity), transparent);
  }
  :host([tone='success']:focus-within) {
    border-color: var(--success);
    box-shadow: 0 0 0 var(--ring-width)
      color-mix(in oklch, var(--success) var(--ring-opacity), transparent);
  }
  :host([tone='destructive']:focus-within) {
    border-color: var(--destructive);
    box-shadow: 0 0 0 var(--ring-width)
      color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }
  :host([tone='warning']:focus-within) {
    border-color: var(--warning);
    box-shadow: 0 0 0 var(--ring-width)
      color-mix(in oklch, var(--warning) var(--ring-opacity), transparent);
  }

  :host([readonly]) {
    opacity: 0.8;
  }

  /* Size variants */
  :host([size='sm']) {
    font-size: var(--text-sm);
    line-height: 1.4;
  }
  :host([size='lg']) {
    font-size: var(--text-lg);
    line-height: 1.6;
  }

  .editor-root {
    min-height: 2em;
  }

  /* Ensure CodeMirror fills the container */
  .editor-root :global(.cm-editor) {
    height: 100%;
  }
`
