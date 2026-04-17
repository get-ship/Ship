/**
 * Styles for <ship-terminal>.
 *
 * The xterm.js canvas renders inside a container div; we style the host
 * and wrapper to integrate with Ship's visual language. The xterm.js
 * CSS is imported by the component at runtime from the xterm package.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipTerminalStyles = [reducedMotion, css`
  :host {
    display: block;
    width: 100%;
    position: relative;
    overflow: hidden;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--background);
  }
  :host([hidden]) { display: none; }

  .terminal-wrap {
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  /* xterm.js injects its own styles on the container; we ensure the
     wrapper fills the host and remove any extra padding xterm adds. */
  .terminal-wrap .xterm {
    padding: var(--space-2) var(--space-3);
  }

  .terminal-wrap .xterm-viewport {
    overflow-y: auto;
  }

  /* Search overlay — positioned top-right inside the terminal */
  .search-bar {
    position: absolute;
    top: var(--space-2);
    right: var(--space-3);
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-sm);
    z-index: var(--z-elevated);
    opacity: 0;
    pointer-events: none;
    transform: translateY(-4px);
    transition:
      opacity var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  .search-bar[data-open='true'] {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0);
  }
  .search-bar input {
    font-family: inherit;
    font-size: var(--text-sm);
    padding: 2px var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--background);
    color: var(--foreground);
    outline: none;
    width: 180px;
  }
  .search-bar input:focus {
    border-color: var(--primary);
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  .search-bar button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--muted-foreground);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--text-sm);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .search-bar button:hover {
    background: var(--muted);
    color: var(--foreground);
  }

  @media (forced-colors: active) {
    :host {
      outline: 1px solid CanvasText;
    }
    .search-bar input:focus {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
  }
`]
