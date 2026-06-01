// sui-table engine — Row<T> factory (Backbone #1, Slice 1b core + Slice 2 selection).
//
// Wraps an original record into the `Row<T>` API shape keyed by the REQUIRED
// `getRowId`. Slice 1b landed the load-bearing core (`id`/`index`/`original`/
// `getValue`). Slice 2 wires the SELECTION members (`getIsSelected`/
// `getIsSomeSelected`/`getCanSelect`/`toggleSelected`) and a minimal GROUP row
// (synthetic parent over leaf children with aggregate-free grouping value) so
// the selection group-cascade + tri-state contract is real. Grouping proper
// (aggregation, footers, deep trees) lands in Slice 3b.
//
// Rows read the selection API through a late-bound `bindings` handle: the
// assembler builds selection AFTER the row pipeline (selection scopes the
// displayed order), then populates `bindings.selection`. Row methods are
// invoked from `on:*`/template — after construction — so the late binding is
// resolved by call time.
//
// PURE TypeScript — no runtime imports (reactivity lives in the pipeline
// computeds that build the row arrays + the selection deriveds).
//
// Authority: build-spec §"Authoritative Row / Cell / Column / Header API",
// §Features #7 (selection cascade + tri-state), Slice 2.

import type { CellBinding } from './cell-api'
import type { Cell, CellValue, ColumnDef, Row } from './types'

/** How a column extracts its value from an original record. */
export type Accessor<T> = (row: T) => CellValue

/** Late-bound handles the row methods route through (selection in Slice 2,
 *  expansion in Slice 3b, cells in Slice 4). */
export interface RowBindings<T> {
  selection?: {
    isSelected: (id: string) => boolean
    canSelectId: (id: string) => boolean
    toggleRowSelected: (
      id: string,
      v?: boolean,
      opts?: { range?: boolean; additive?: boolean },
    ) => void
  }
  expansion?: {
    isExpanded: (id: string) => boolean
    toggleExpanded: (id: string, v?: boolean) => void
  }
  /** Cells — `getVisibleCells(row)` maps visibleLeafColumns to Cell<T> (Slice 4). */
  cells?: CellBinding<T>
}

/** The visible-cell list for a row through the late-bound cell handle. */
function visibleCellsOf<T>(row: Row<T>, bindings: RowBindings<T>): Cell<T>[] {
  return bindings.cells?.getVisibleCells(row) ?? []
}

/**
 * Build the accessor map for a flat column list. Only accessor columns
 * (`accessorKey`) and display columns carrying an `accessorFn` are valued; a
 * column with neither yields no entry (its `getValue` returns undefined).
 */
export function buildAccessors<T>(columns: ColumnDef<T>[]): Map<string, Accessor<T>> {
  const map = new Map<string, Accessor<T>>()
  for (const def of columns) {
    if ('accessorKey' in def && def.accessorKey != null) {
      const key = def.accessorKey
      map.set(def.id, (row: T) => (row as Record<string, unknown>)[key] as CellValue)
    } else if ('accessorFn' in def && typeof def.accessorFn === 'function') {
      const fn = def.accessorFn
      map.set(def.id, (row: T) => fn(row))
    }
  }
  return map
}

/** Tree shape passed when a row carries sub-rows (Feature #13 expansion). */
export interface RowTree<T> {
  /** Pre-built child rows (recursion builds these depth-first). */
  subRows: Row<T>[]
  /** The parent row (undefined at the top level). */
  parent?: Row<T>
  /** The depth in the tree (top-level = 0). */
  depth: number
}

/**
 * Wrap one original record into a `Row<T>`. `index` is the row's position in
 * the core (pre-transform) order. `accessors` is the shared column→value map.
 * `bindings.selection`/`bindings.expansion` (late-bound) route the selection +
 * expansion members; until populated they report the "feature off" defaults.
 *
 * `tree` carries the sub-row structure for tree data (`getSubRows`): when a row
 * has children it `getCanExpand()`s and `getLeafRows()`/`getParentRows()` walk
 * the structure. A leaf row (no `tree`) keeps the flat-row defaults.
 */
export function makeRow<T>(
  original: T,
  index: number,
  getRowId: (row: T, index: number) => string,
  accessors: Map<string, Accessor<T>>,
  bindings: RowBindings<T>,
  tree?: RowTree<T>,
): Row<T> {
  const id = getRowId(original, index)
  const subRows = tree?.subRows ?? []
  const depth = tree?.depth ?? 0

  const getValue = (colId: string): CellValue => {
    const accessor = accessors.get(colId)
    return accessor ? accessor(original) : undefined
  }

  const collectLeaves = (): Row<T>[] => {
    if (subRows.length === 0) return [row]
    const out: Row<T>[] = []
    for (const child of subRows) out.push(...child.getLeafRows())
    return out
  }

  const row: Row<T> = {
    id,
    index,
    depth,
    original,
    getValue,
    getVisibleCells: () => visibleCellsOf(row, bindings),
    subRows,
    getCanExpand: () => subRows.length > 0,
    getIsExpanded: () => bindings.expansion?.isExpanded(id) ?? false,
    toggleExpanded: (v?: boolean) => bindings.expansion?.toggleExpanded(id, v),
    getParentRow: () => tree?.parent,
    getParentRows: () => {
      const chain: Row<T>[] = []
      let p = tree?.parent
      while (p) {
        chain.unshift(p)
        p = p.getParentRow()
      }
      return chain
    },
    getLeafRows: collectLeaves,
    getIsSelected: () => bindings.selection?.isSelected(id) ?? false,
    getIsSomeSelected: () => false,
    getCanSelect: () => bindings.selection?.canSelectId(id) ?? true,
    toggleSelected: (v?: boolean, opts?: { range?: boolean }) =>
      bindings.selection?.toggleRowSelected(id, v, opts),
    getIsPinned: () => false,
    pin: () => {},
    getIsGrouped: () => false,
    getGroupingValue: (colId: string) => getValue(colId),
  }
  return row
}

/** Inputs for a synthetic group row (Slice 3b — aggregation + nesting). */
export interface GroupRowInputs<T> {
  groupColId: string
  groupingValue: unknown
  /** Direct children: leaf rows OR nested group rows (multi-level grouping). */
  subRows: Row<T>[]
  /** The flat leaf descendants (the aggregation domain). */
  leafRows: Row<T>[]
  index: number
  depth: number
  bindings: RowBindings<T>
  /** Per-column aggregate over the leaf rows: colId → aggregated CellValue. */
  aggregate: (colId: string) => CellValue
}

/**
 * Build a synthetic group row keyed by `groupingValue` for `groupColId`. The
 * group row has no original record; `getValue(colId)` returns the grouping
 * value for the group column and the column AGGREGATE (sum/min/…) for an
 * aggregating column. Selection cascades to the leaf descendants (all →
 * selected; some → tri-state). Expansion is keyed by the group row's id through
 * the late-bound expansion handle so a collapsed group hides its children.
 */
export function makeGroupRow<T>(inputs: GroupRowInputs<T>): Row<T> {
  const { groupColId, groupingValue, subRows, leafRows, index, depth, bindings, aggregate } =
    inputs
  const id = `group:${depth}:${groupColId}:${String(groupingValue)}`
  const selectableLeaves = (): Row<T>[] =>
    leafRows.filter((l) => bindings.selection?.canSelectId(l.id) ?? true)

  const group: Row<T> = {
    id,
    index,
    depth,
    original: undefined as unknown as T,
    getValue: (colId: string) =>
      colId === groupColId ? (groupingValue as CellValue) : aggregate(colId),
    getVisibleCells: () => visibleCellsOf(group, bindings),
    subRows,
    getCanExpand: () => subRows.length > 0,
    getIsExpanded: () => bindings.expansion?.isExpanded(id) ?? true,
    toggleExpanded: (v?: boolean) => bindings.expansion?.toggleExpanded(id, v),
    getParentRow: () => undefined,
    getParentRows: () => [],
    getLeafRows: () => leafRows,
    getIsSelected: () => {
      const sel = selectableLeaves()
      return sel.length > 0 && sel.every((l) => l.getIsSelected())
    },
    getIsSomeSelected: () => {
      const sel = selectableLeaves()
      const some = sel.some((l) => l.getIsSelected())
      return some && !sel.every((l) => l.getIsSelected())
    },
    getCanSelect: () => selectableLeaves().length > 0,
    toggleSelected: (v?: boolean) => {
      const sel = selectableLeaves()
      const target = v ?? !(sel.length > 0 && sel.every((l) => l.getIsSelected()))
      for (const l of sel) l.toggleSelected(target)
    },
    getIsPinned: () => false,
    pin: () => {},
    getIsGrouped: () => true,
    getGroupingValue: (colId: string) =>
      colId === groupColId ? groupingValue : undefined,
  }
  return group
}
