/**
 * Styles for <ship-tabs> + <ship-tab-panel>.
 *
 * Channel architecture — every selector below reads from one of:
 *   --tab-h / --tab-px / --tab-fs / --tab-gap / --tab-radius
 *     Size channel. md matches pre-polish pipelines/agents values so
 *     existing templates keep their weight; sm/lg derive from the same
 *     scale as ship-button so sibling controls align optically.
 *   --tone / --tone-fg
 *     Active-tab accent. Defaults to --primary; tone=info|success|warning|
 *     destructive|accent|muted colorizes the underline + count chip on
 *     the active tab without affecting inactive tabs.
 *   --underline-x / --underline-w (horizontal) or --underline-y/h (vertical)
 *     Written from the component on active-tab change. A single ::after
 *     pseudo-element on .tablist translates along the bar via transform
 *     so the indicator moves without triggering layout.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipTabsStyles = [reducedMotion, css`
  :host {
    display: block;
    font-family: var(--font-body);
    --tone: var(--primary);
    --tone-fg: var(--foreground);
  }
  :host([hidden]) { display: none; }

  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  /* Size channel — matches ship-button's sm/md/lg height scale so a tab
   * sitting next to a button lines up optically. */
  :host(:not([size])),
  :host([size='md']) {
    --tab-h: 36px;
    --tab-px: 14px;
    --tab-fs: var(--text-base);
    --tab-fw: var(--font-weight-medium);
    --tab-active-fw: var(--font-weight-semibold);
    --tab-gap: var(--space-2);
    --tab-radius: var(--radius-md);
    --tab-count-fs: 10.5px;
    --tab-count-px: 7px;
  }
  :host([size='sm']) {
    --tab-h: 30px;
    --tab-px: 10px;
    --tab-fs: var(--text-sm);
    --tab-gap: 6px;
    --tab-radius: var(--radius-sm);
    --tab-count-fs: 9.5px;
    --tab-count-px: 6px;
  }
  :host([size='lg']) {
    --tab-h: 42px;
    --tab-px: var(--space-4);
    --tab-fs: var(--text-md);
    --tab-gap: var(--space-2);
    --tab-radius: var(--radius-md);
    --tab-count-fs: 11px;
    --tab-count-px: 8px;
  }

  /* Shared tablist + tab chrome. Variants reshape from here via their
   * class hook (.underline / .segmented) — no per-variant duplication. */
  .tablist {
    position: relative;
    display: flex;
    align-items: stretch;
    gap: 2px;
  }
  .tab {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--tab-gap);
    height: var(--tab-h);
    padding: 0 var(--tab-px);
    background: transparent;
    border: 0;
    color: var(--muted-foreground);
    font-family: inherit;
    font-size: var(--tab-fs);
    font-weight: var(--tab-fw, var(--font-weight-medium));
    line-height: 1;
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    -webkit-user-select: none;
    -webkit-tap-highlight-color: transparent;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  .tab:hover:not(:disabled):not([aria-selected='true']) {
    color: var(--foreground);
  }
  .tab[aria-selected='true'] {
    color: var(--tone-fg);
    font-weight: var(--tab-active-fw, var(--font-weight-semibold));
  }
  .tab:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    border-radius: var(--tab-radius);
    z-index: 1;
  }
  .tab:disabled {
    cursor: not-allowed;
    color: color-mix(in oklch, var(--muted-foreground) 55%, transparent);
    filter: saturate(0.6);
  }

  /* Underline variant — indicator is an ::after pseudo on .tablist
   * positioned via --underline-x/--underline-w; width transition drives
   * the tab-to-tab slide. Pre-measurement opacity hides the indicator
   * until the component writes the geometry on first update. */
  .tablist.underline {
    border-bottom: 1px solid var(--border);
    margin-bottom: 20px;
    gap: 0;
  }
  .tablist.underline::after {
    content: '';
    position: absolute;
    left: 0;
    bottom: -1px;
    height: 2px;
    width: var(--underline-w, 0px);
    transform: translateX(var(--underline-x, 0px));
    background: var(--tone);
    border-radius: 2px 2px 0 0;
    pointer-events: none;
    transition:
      transform var(--duration-medium) var(--ease-out),
      width var(--duration-medium) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }
  :host(:not([data-underline-ready])) .tablist.underline::after {
    opacity: 0;
  }
  .tablist.underline .tab { border-radius: 0; margin-bottom: -1px; }
  .tablist.underline .tab:hover:not(:disabled):not([aria-selected='true']) {
    background: color-mix(in oklch, var(--muted) 60%, transparent);
  }
  :host([sticky]) .tablist.underline {
    position: sticky;
    top: 0;
    background: var(--background);
    z-index: var(--z-sticky);
    padding-top: 4px;
    margin-top: -4px;
  }

  /* Segmented variant — pill group. Active tab elevates onto --background
   * with a soft shadow so it reads as a chip lifted off the muted track. */
  .tablist.segmented {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    background: var(--muted);
    border: 1px solid color-mix(in oklch, var(--border) 80%, transparent);
    border-radius: calc(var(--tab-radius) + 3px);
  }
  .tablist.segmented .tab {
    flex: 1;
    height: calc(var(--tab-h) - 6px);
    border-radius: var(--tab-radius);
  }
  .tablist.segmented .tab:hover:not(:disabled):not([aria-selected='true']) {
    color: var(--foreground);
    background: color-mix(in oklch, var(--background) 60%, transparent);
  }
  .tablist.segmented .tab[aria-selected='true'] {
    background: var(--background);
    color: var(--foreground);
    box-shadow:
      inset 0 0 0 1px var(--border-subtle),
      var(--shadow-xs);
  }
  .tablist.segmented::after { display: none; }

  /* Count chip — tabular-nums so digit-width changes don't shift the
   * label. Active state re-colors to the tone channel. */
  .count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: calc(var(--tab-count-fs) + 8px);
    padding: 1px var(--tab-count-px);
    background: color-mix(in oklch, var(--muted-foreground) 14%, transparent);
    color: var(--muted-foreground);
    border-radius: var(--radius-full);
    font-size: var(--tab-count-fs);
    font-weight: var(--font-weight-semibold);
    font-variant-numeric: tabular-nums;
    line-height: 1.4;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .tab[aria-selected='true'] .count {
    background: color-mix(in oklch, var(--tone) 18%, transparent);
    color: var(--tone);
  }

  .panels { display: block; }

  /* Vertical orientation — tablist on the left, panels on the right.
   * Underline indicator flips to a right-edge rail; --underline-y/h drive
   * the same transform-based animation on the Y axis. */
  :host([orientation='vertical']) {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--space-6);
  }
  :host([orientation='vertical']) .tablist.underline {
    flex-direction: column;
    align-items: stretch;
    border-bottom: 0;
    border-right: 1px solid var(--border);
    margin-bottom: 0;
    gap: 2px;
  }
  :host([orientation='vertical']) .tablist.underline .tab {
    justify-content: flex-start;
    margin-bottom: 0;
    margin-right: -1px;
    border-radius: var(--tab-radius) 0 0 var(--tab-radius);
  }
  :host([orientation='vertical']) .tablist.underline::after {
    left: auto;
    right: -1px;
    bottom: auto;
    top: 0;
    width: 2px;
    height: var(--underline-h, 0px);
    transform: translateY(var(--underline-y, 0px));
    border-radius: 2px 0 0 2px;
    transition:
      transform var(--duration-medium) var(--ease-out),
      height var(--duration-medium) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }
  :host([orientation='vertical']) .tablist.segmented {
    flex-direction: column;
    padding: 3px;
  }
  :host([orientation='vertical']) .tablist.segmented .tab {
    flex: none;
    justify-content: flex-start;
  }

  @media (forced-colors: active) {
    .tab:focus-visible { outline: 2px solid CanvasText; outline-offset: -2px; }
    .tablist.underline::after { background: CanvasText; }
    .tab[aria-selected='true'] { color: CanvasText; }
  }
`]

export const shipTabPanelStyles = [reducedMotion, css`
  :host { display: none; }
  :host([active]) {
    display: block;
    animation: ship-tab-panel-fade var(--duration-medium) var(--ease-out);
  }
  @keyframes ship-tab-panel-fade {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  :host(:focus-visible) {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    border-radius: var(--radius-md);
  }
`]
