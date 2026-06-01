// component-viewer — "eyes on active dev" surface.
//
// Browses the campaign's `<sui-*>` tags and mounts the selected one live in a
// <sui-render mode="tag"> stage. COMPONENT-MODE only (no live source editing —
// that needs the daemon-WASM change, task #241).
//
// Why no in-browser WASM: <sui-render>'s tag path (engine/mount-tag.ts →
// ensureDefined) short-circuits on `customElements.get(tag)` — a tag already in
// the registry is NOT recompiled. Every campaign tag this view shows is
// registered into THIS view bundle by the side-effect imports below, so the
// stub `compile()` is never invoked. (It must still be non-null + `source`
// non-empty to clear the effect guard in sui-render.controller.ts.)
//
// Registration seam: a view bundle does NOT inherit the shell's component
// registry at author time — it traces its OWN import graph. So the pack +
// every campaign tag is side-effect-imported here. sui-table + the chart tags
// are also shell-registered, but importing them here keeps the view
// self-contained (one source of truth for what the stage can mount).

import { controller, signal, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './index.sui'
import type { CompileFn } from '../../../packages/components/sui/viewer/src/wasm-bootstrap'

// ── Register the viewing pack + every campaign tag into THIS view bundle ──
// Each import side-effect-calls customElements.define(); the bundler compiles
// the sibling `.sui` per import (deck rolldown SuiPlugin). sui-tree,
// sui-tree-grid, sui-sheet, sui-pagination are NOT in any umbrella — they are
// pulled in directly here (the only place they get registered for the shell).
import '../../../packages/components/sui/viewer/src/index'
import '../../../packages/components/sui/primitives/src/sui-table/sui-table.compiled.js'
import '../../../packages/components/sui/primitives/src/sui-tree/sui-tree.compiled.js'
import '../../../packages/components/sui/primitives/src/sui-tree-grid/sui-tree-grid.compiled.js'
import '../../../packages/components/sui/primitives/src/sui-sheet/sui-sheet.compiled.js'
import '../../../packages/components/sui/primitives/src/sui-pagination/sui-pagination.compiled.js'
import '@ship/sui-charts'

/** A campaign tag the stage can mount, with example props that produce
 *  characterful output (no placeholders). `props` keys are the tag's declared
 *  prop names; string-typed props carry serialized JSON (the tag parses it). */
export interface TagSpec {
  /** The custom-element tag name (drives sui-render component-name). */
  name: string
  /** Human label for the browse list. */
  label: string
  /** Grouping bucket for the browse list. */
  group: 'Data' | 'Charts' | 'Viewer'
  /** One-line description shown under the stage. */
  blurb: string
  /** Example prop values applied via sui-render prop:prop-values. */
  props: Record<string, unknown>
}

// ── Demo data ────────────────────────────────────────────────────────────
// sui-table: rows are `[{id, ...}]`; columns are `[{accessorKey, header}]`.
const TABLE_COLUMNS = JSON.stringify([
  { accessorKey: 'lane', header: 'Lane' },
  { accessorKey: 'model', header: 'Model' },
  { accessorKey: 'composite', header: 'Composite' },
  { accessorKey: 'status', header: 'Status' },
])
const TABLE_ROWS = JSON.stringify([
  { id: '1', lane: 'sui-table', model: 'opus-4.8', composite: 0.91, status: 'pass' },
  { id: '2', lane: 'sui-tree', model: 'opus-4.8', composite: 0.86, status: 'pass' },
  { id: '3', lane: 'sui-sheet', model: 'haiku-4.5', composite: 0.72, status: 'fail' },
  { id: '4', lane: 'sui-pagination', model: 'haiku-4.5', composite: 0.88, status: 'pass' },
])

// sui-tree / sui-tree-grid: nodes are `[{id, label, children}]`.
const TREE_NODES = JSON.stringify([
  {
    id: 'pkg',
    label: 'packages/components',
    children: [
      {
        id: 'primitives',
        label: 'sui/primitives',
        children: [
          { id: 'table', label: 'sui-table' },
          { id: 'tree', label: 'sui-tree' },
          { id: 'sheet', label: 'sui-sheet' },
        ],
      },
      { id: 'viewer', label: 'sui/viewer' },
      { id: 'charts', label: 'sui/charts' },
    ],
  },
])
const TREE_GRID_COLUMNS = JSON.stringify([
  { accessorKey: 'kind', header: 'Kind' },
  { accessorKey: 'files', header: 'Files' },
])
const TREE_GRID_NODES = JSON.stringify([
  {
    id: 'pkg',
    label: 'packages/components',
    kind: 'dir',
    files: 42,
    children: [
      { id: 'primitives', label: 'sui/primitives', kind: 'pack', files: 31 },
      { id: 'viewer', label: 'sui/viewer', kind: 'pack', files: 6 },
      { id: 'charts', label: 'sui/charts', kind: 'pack', files: 7 },
    ],
  },
])

// sui-sheet: workbook is `{ sheets: [{ id, name, rowCount, colCount, cells }] }`.
const SHEET_WORKBOOK = JSON.stringify({
  sheets: [
    {
      id: 's1',
      name: 'Burn',
      rowCount: 4,
      colCount: 3,
      cells: {
        A1: { value: 'Lane' },
        B1: { value: 'Tokens' },
        C1: { value: 'Cost' },
        A2: { value: 'sui-table' },
        B2: { value: 41000 },
        C2: { value: 0.62 },
        A3: { value: 'sui-tree' },
        B3: { value: 28000 },
        C3: { value: 0.41 },
      },
    },
  ],
})

// ── Chart demo data ──────────────────────────────────────────────────────
// sui-line-chart reads `series` (normalizeSeries accepts a `[{x,y}]` array);
// sui-bar-chart reads `data` (same shape, different prop name); sui-sparkline
// reads `data` as a bare comma-number string (parseNums).
const CHART_SERIES = JSON.stringify([
  { x: 'mon', y: 12 },
  { x: 'tue', y: 19 },
  { x: 'wed', y: 7 },
  { x: 'thu', y: 23 },
  { x: 'fri', y: 17 },
])
const SPARK_DATA = '12,19,7,23,17,9,21'

/** The full campaign tag registry the browse list renders. Each entry is a
 *  fully-formed, renderable specimen. */
export const TAGS: TagSpec[] = [
  {
    name: 'sui-table',
    label: 'Table',
    group: 'Data',
    blurb: 'Sortable data grid — the eval-lane comparison surface.',
    props: { columns: TABLE_COLUMNS, rows: TABLE_ROWS, striped: true, hoverable: true, size: 'md' },
  },
  {
    name: 'sui-tree',
    label: 'Tree',
    group: 'Data',
    blurb: 'Hierarchical row tree — the package/file explorer shape.',
    props: { nodes: TREE_NODES, label: 'Components tree', 'show-guides': true, size: 'md' },
  },
  {
    name: 'sui-tree-grid',
    label: 'Tree Grid',
    group: 'Data',
    blurb: 'Tree + columns — a hierarchy with per-row metrics.',
    props: { nodes: TREE_GRID_NODES, columns: TREE_GRID_COLUMNS, label: 'Packages', size: 'md' },
  },
  {
    name: 'sui-sheet',
    label: 'Sheet',
    group: 'Data',
    blurb: 'Spreadsheet grid — formula bar, headers, gridlines.',
    props: { workbook: SHEET_WORKBOOK, label: 'Burn', 'show-formula-bar': true },
  },
  {
    name: 'sui-pagination',
    label: 'Pagination',
    group: 'Data',
    blurb: 'Page control — page/total/page-size with size options.',
    props: { page: 2, total: 240, 'page-size': 25, label: 'Eval rows' },
  },
  {
    name: 'sui-line-chart',
    label: 'Line Chart',
    group: 'Charts',
    blurb: 'Time-series line — token burn across a week.',
    props: { series: CHART_SERIES, label: 'Tokens / day', grid: true, points: true },
  },
  {
    name: 'sui-bar-chart',
    label: 'Bar Chart',
    group: 'Charts',
    blurb: 'Categorical bars — same series, bar geometry.',
    props: { data: CHART_SERIES, label: 'Tokens / day', tone: 'primary' },
  },
  {
    name: 'sui-sparkline',
    label: 'Sparkline',
    group: 'Charts',
    blurb: 'Inline trend — minimal, label-free.',
    props: { data: SPARK_DATA, tone: 'success', fill: true },
  },
]

// NOTE on the `sui-viewer` pack itself: the stage IS a <sui-render mode="tag">,
// so the pack-16 engine primitive is exercised live on every mount. The
// embeddable <sui-component-viewer> is NOT listed as a stage specimen because
// mounting it by bare tag can't supply the `compile`/`source`/`cem` identity
// props it needs — that path needs the in-browser compiler (task #241). Showing
// it as a dead specimen would misrepresent it, so it is omitted by design.

// Stub compile: never invoked for a pre-registered tag (ensureDefined returns
// early on customElements.get). If it WERE reached, it returns a structured
// error so the stage shows a compile-error panel rather than throwing — the
// honest signal that the tag is not registered in this bundle.
const stubCompile: CompileFn = (_source, _opts) => ({
  kind: 'err',
  message:
    'component-mode: this tag is not registered in the view bundle (no in-browser compiler — see task #241)',
  line: null,
  col: null,
  category: 'compile',
})

// Any non-empty string clears the `src == null` guard in sui-render's effect;
// it is never compiled because the tag is already defined.
const PLACEHOLDER_SOURCE = '<!-- pre-registered tag; mounted by name -->'

export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>) => {
    const selected = signal<string>(TAGS[0].name)
    return { selected }
  },
  derived: {
    // The full registry, surfaced to the template (module const → scope read).
    tags: (): TagSpec[] => TAGS,
    // The currently selected tag spec (always resolves — falls back to first).
    current: ({ selected }): TagSpec => TAGS.find((t) => t.name === selected()) ?? TAGS[0],
    // Identity props threaded into <sui-render> via prop:*.
    compile: (): CompileFn => stubCompile,
    source: (): string => PLACEHOLDER_SOURCE,
    propValues: ({ selected }): Record<string, unknown> =>
      (TAGS.find((t) => t.name === selected()) ?? TAGS[0]).props,
  },
})
