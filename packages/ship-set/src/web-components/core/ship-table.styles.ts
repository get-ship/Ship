/**
 * Styles for <ship-table>.
 *
 * Two layers:
 *   1. `shipTableStyles` — Shadow-DOM scoped styles for the wrapper chrome
 *      (border, radius, overflow, empty-state slot, slotted row hover).
 *   2. `shipTableLightDomCss` — A plain CSS string adopted once on the
 *      host document. Targets light-DOM descendants (`<th>`, `<td>`,
 *      `<tr>`) under `ship-table`. Necessary because Shadow DOM's
 *      `::slotted()` only reaches the directly slotted elements; cell
 *      elements live inside a slotted `<thead>` or `<tr>` and are not
 *      reachable from Shadow CSS.
 *
 * Channel model — same as ship-card / ship-input:
 *   --row-py / --row-px / --th-py / --th-fs
 *     Density channel set on the host per `density=[compact|default|
 *     comfortable]`. `dense` (legacy boolean) aliases to compact. The
 *     light-DOM layer reads these off the host via CSS inheritance
 *     so densities stay in sync without JS plumbing.
 *   --row-hover-mix
 *     How strongly a hovered row tint shows; exposed so dense tables
 *     can dial it down.
 *   --table-radius
 *     Wrapper radius. Uses --radius-lg to match shadcn/ui Table.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipTableStyles = [
  reducedMotion,
  css`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
      font-size: var(--text-base);
      color: var(--foreground);

      /* Density channel — default matches the pre-polish 10px/12px cells
       * the SDK table macro uses, so existing views don't shift a pixel. */
      --row-py: 10px;
      --row-px: var(--space-3);
      --th-py: 10px;
      --th-fs: var(--text-xs);
      --table-radius: var(--radius-lg);
      --row-hover-mix: 45%;
    }
    :host([hidden]) { display: none; }

    /* Density: compact — log-table feel. Legacy 'dense' aliases to this. */
    :host([density='compact']),
    :host([dense]) {
      --row-py: 6px;
      --row-px: var(--space-2);
      --th-py: 6px;
    }
    /* Density: comfortable — settings / admin feel. */
    :host([density='comfortable']) {
      --row-py: 14px;
      --row-px: var(--space-4);
      --th-py: 14px;
    }

    /* ------------------------------------------------------------------
     * Wrapper — source-of-truth surface. Border + radius + overflow
     * live here so the table inside can scroll horizontally on narrow
     * viewports without the border clipping mid-cell. Mirrors shadcn's
     * Table's wrapping <div>.
     * ------------------------------------------------------------------ */
    .wrap {
      position: relative;
      width: 100%;
      border: 1px solid var(--border);
      border-radius: var(--table-radius);
      background: var(--card);
      box-shadow: var(--shadow-xs);
      overflow: hidden;
      overflow-x: auto;
      transition: box-shadow var(--duration-fast) var(--ease-out);
    }

    table {
      width: 100%;
      border-collapse: separate;
      border-spacing: 0;
      table-layout: auto;
      font-variant-numeric: tabular-nums;
    }

    /* Sticky-header — pins <thead> while the body scrolls. The consumer
     * sets max-height on the host (inline style); we mirror it onto the
     * wrapper so the scroll container is the wrapper itself, which keeps
     * the border + radius intact around the scrolled body. */
    :host([sticky-header]) {
      display: block;
    }
    :host([sticky-header]) .wrap {
      overflow-y: auto;
      max-height: inherit;
    }

    /* ------------------------------------------------------------------
     * Empty state — rendered when the consumer slots content into the
     * 'empty' slot AND the tbody has no <tr> children. Visibility is
     * toggled via the 'data-empty' attribute set from the component's
     * slotchange / MutationObserver logic.
     * ------------------------------------------------------------------ */
    .empty {
      display: none;
      padding: var(--space-7) var(--space-5);
      text-align: center;
      color: var(--muted-foreground);
      font-size: var(--text-sm);
      background: var(--card);
    }
    :host([data-empty]) .empty { display: block; }
    :host([data-empty]) table { display: none; }

    @media (forced-colors: active) {
      .wrap { border-color: CanvasText; }
    }
  `,
]

/**
 * Light-DOM CSS adopted once on the host document. Styles cells and
 * rows living under a <ship-table>. Scoped by the `ship-table` tag
 * selector so it doesn't leak onto arbitrary tables.
 *
 * Consumers can compose their markup two ways; both land on the same
 * visual result:
 *
 *   A) Slot pattern (preserves the pre-polish API used by tests):
 *      <ship-table>
 *        <tr slot="head"><th>Name</th></tr>
 *        <tr><td>Alpha</td></tr>
 *      </ship-table>
 *
 *   B) Table-fragment pattern (more ergonomic for existing HTML tables):
 *      <ship-table>
 *        <thead><tr><th>Name</th></tr></thead>
 *        <tbody><tr><td>Alpha</td></tr></tbody>
 *      </ship-table>
 *
 * Both patterns live in light DOM as descendants of <ship-table>, so a
 * single set of scoped selectors reaches all cells via `ship-table th`
 * and `ship-table td`. Row-specific concerns (striping, selection,
 * hover) differ slightly between patterns and are expressed below.
 */
export const shipTableLightDomCss = `
  ship-table {
    --ship-table-border-soft: color-mix(in oklch, var(--border) 80%, transparent);
  }

  /* In native mode the consumer's <table> is a direct child — reset
   * its own borders/spacing so the ship-table wrapper chrome owns the
   * surface and nothing double-lines. */
  ship-table[variant='native'] > table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  /* Cell chrome — padding, baseline, separator. Reads density from the
   * host custom properties via inheritance. Scoped under ship-table so
   * we only claim cells inside the component, not arbitrary tables. */
  ship-table th,
  ship-table td {
    text-align: left;
    padding: var(--row-py, 10px) var(--row-px, 12px);
    vertical-align: middle;
    border-bottom: 1px solid var(--ship-table-border-soft);
    font-size: var(--text-base, 13px);
    color: var(--foreground);
    box-sizing: border-box;
  }
  ship-table th {
    padding-top: var(--th-py, 10px);
    padding-bottom: var(--th-py, 10px);
    font-weight: var(--font-weight-semibold, 600);
    font-size: var(--th-fs, 10.5px);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
    user-select: none;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  ship-table thead {
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  /* Last body row — drop separator; the wrapper border closes the table.
   * Works for both the slot pattern (bare <tr> as direct child) and the
   * fragment pattern (<tr> inside <tbody>). */
  ship-table > tr:last-of-type > *,
  ship-table tbody tr:last-child > * {
    border-bottom: 0;
  }

  /* Column alignment. */
  ship-table th[align='right'], ship-table td[align='right'] { text-align: right; }
  ship-table th[align='center'], ship-table td[align='center'] { text-align: center; }

  /* Mono/muted conventions for the data-view SDK table + similar. */
  ship-table .mono {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-sm, 12px);
    letter-spacing: -0.01em;
  }
  ship-table .muted { color: var(--muted-foreground); }
  ship-table .null { color: var(--muted-foreground); font-style: italic; }

  /* Row hover — on by default, opt-out via hoverable="false". Applies
   * to body rows in both patterns: bare <tr> under ship-table, or <tr>
   * under <tbody>. Excludes the thead row via :not([slot='head']). */
  ship-table > tr:not([slot='head']),
  ship-table tbody tr {
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  ship-table:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  ship-table:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  /* Striped — zebra on alternate body rows. Both patterns. */
  ship-table[striped] > tr:not([slot='head']):nth-of-type(even) > *,
  ship-table[striped] tbody tr:nth-child(even) > * {
    background: color-mix(in oklch, var(--muted) 25%, transparent);
  }
  ship-table[striped]:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  ship-table[striped]:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  /* Selected row — data-selected attr or aria-selected="true". */
  ship-table tr[data-selected] > *,
  ship-table tr[aria-selected='true'] > * {
    background: var(--tone-primary-bg);
  }
  ship-table tr[data-selected] > *:first-child,
  ship-table tr[aria-selected='true'] > *:first-child {
    box-shadow: inset 3px 0 0 0 var(--primary);
  }

  /* Sticky-header — applies position:sticky to the thead cells so the
   * header rides above scrolling body rows. Works on both patterns
   * because the thead cells sit inside the shadow <thead> via slot
   * projection, and the slotted element inherits sticky semantics. */
  ship-table[sticky-header] thead th,
  ship-table[sticky-header] tr[slot='head'] > th {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 20);
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
    box-shadow: 0 1px 0 0 var(--border);
  }

  /* Sortable headers — chevron rendered by the component; these rules
   * paint the cursor + color transitions. */
  ship-table th[data-sort-key],
  ship-table th.sortable {
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition: color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  ship-table th[data-sort-key]:hover,
  ship-table th.sortable:hover {
    color: var(--foreground);
  }
  ship-table th[aria-sort='ascending'],
  ship-table th[aria-sort='descending'] {
    color: var(--foreground);
  }
  ship-table th[data-sort-key]:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 var(--ring-width, 3px)
      color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 25%), transparent);
  }

  /* Empty-state — in native mode the <table> is a light-DOM child so
   * hiding it has to happen from the document stylesheet. The host
   * [data-empty] attribute is the single source of truth either way. */
  ship-table[data-empty] > table {
    display: none;
  }

  /* ----------------------------------------------------------------
   * Sticky first column — first td/th stays pinned during horizontal
   * scroll. Pair with sticky-header for the full Drizzle experience.
   * ---------------------------------------------------------------- */
  ship-table[sticky-first-col] th:first-child,
  ship-table[sticky-first-col] td:first-child {
    position: sticky;
    left: 0;
    z-index: 1;
    background: var(--card);
    border-right: 1px solid var(--border);
  }
  ship-table[sticky-first-col] thead th:first-child {
    z-index: 3;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  ship-table[sticky-first-col] tbody tr:hover td:first-child {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  /* ----------------------------------------------------------------
   * Gutter column — narrow leading column for checkboxes / row numbers.
   * Enabled via the gutter attribute. ship-table-gutter slot per row.
   * ---------------------------------------------------------------- */
  ship-table[gutter] th.ship-table-gutter-cell,
  ship-table[gutter] td.ship-table-gutter-cell {
    width: 40px;
    min-width: 40px;
    max-width: 40px;
    padding: 0;
    text-align: center;
    color: var(--muted-foreground);
    background: color-mix(in oklch, var(--muted-foreground) 5%, var(--background));
    border-right: 1px solid var(--border);
    position: sticky;
    left: 0;
    z-index: 2;
  }
  ship-table[gutter] thead th.ship-table-gutter-cell { z-index: 4; }

  /* ----------------------------------------------------------------
   * Field-type badges in column headers. Move .ft-badge from the data
   * view into the element light-DOM stylesheet so all consumers inherit.
   * ---------------------------------------------------------------- */
  ship-table .ft-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 16px;
    padding: 0 5px;
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 9.5px;
    font-weight: var(--font-weight-semibold);
    letter-spacing: 0.02em;
    line-height: 1;
    text-transform: lowercase;
    color: var(--muted-foreground);
    background: color-mix(in oklch, var(--muted-foreground) 12%, transparent);
    vertical-align: middle;
  }
  ship-table .ft-badge.ft-string     { color: var(--primary);       background: var(--tone-primary-bg); }
  ship-table .ft-badge.ft-number     { color: var(--accent);        background: var(--tone-accent-bg); }
  ship-table .ft-badge.ft-boolean    { color: var(--success);       background: var(--tone-success-bg); }
  ship-table .ft-badge.ft-enum       { color: var(--warning);       background: var(--tone-warning-bg); }
  ship-table .ft-badge.ft-array      { color: var(--info);          background: var(--tone-info-bg); }
  ship-table .ft-badge.ft-markdown   { color: var(--status-purple); background: color-mix(in oklch, var(--status-purple) 14%, transparent); }
  ship-table .ft-badge.ft-timestamp  { color: var(--muted-foreground); background: color-mix(in oklch, var(--muted-foreground) 10%, transparent); }

  /* ----------------------------------------------------------------
   * Clickable rows — ship-row-click is dispatched for rows that have
   * a row-id attribute on cells (any cell not .ship-table-gutter-cell).
   * ---------------------------------------------------------------- */
  ship-table tr[data-row-id] > td:not(.ship-table-gutter-cell) {
    cursor: pointer;
  }

  /* Forced colors — keep legibility in high-contrast mode. */
  @media (forced-colors: active) {
    ship-table th, ship-table td { border-bottom-color: CanvasText; }
  }
`
