// sui-sheet engine — createSheet() assembler (the headless cell engine).
//
// Wires the derived chain (pipeline.ts) + the interaction slices (selection,
// editing) + geometry into the `Sheet` facade. The reactive cell model lives in
// `state.ts` (one signal per cell → dirty-suffix writes); the read chain in
// `pipeline.ts`; this file is the thin wire-up. Imports `{ batch, derived }` from
// `@ship/sui-runtime` ONLY — no SDK, no network, no DOM (the engine is pure
// reactive state over a cell space; the view renders it via prop:sheet).
//
// SCOPE — FOUNDATIONAL slice: the literal cell path, range selection, the
// editing model, geometry, and live-workbook ingestion. The formula engine
// (parse / DAG / eval), fill-handle series, and clipboard re-basing are DEFERRED
// to follow-up slices; their facade methods throw a clear deferred error rather
// than ship a silent no-op that "looks better than it is".
//
// PURE TypeScript. Authority: build-spec §"Backbone — createSheet()", §"API
// sketch", §"Signal graph", §Slices 1/3, create-sheet.test.ts.

import { batch, derived } from '@ship/sui-runtime'
import { fromA1, toA1 } from './a1'
import { buildEditing } from './editing'
import { buildPipeline } from './pipeline'
import { buildSelection } from './selection'
import {
  buildInteractionState,
  emptySelection,
  liftWorkbook,
  makeSelection,
  reflect,
} from './state'
import { makeRange } from './range'
import type { Sheet, SheetOptions } from './contract'
import type {
  CellAddress,
  CellData,
  CellEdit,
  CellRange,
  ResolvedCell,
  SheetDef,
} from './types'

const DEFERRED = (feature: string): never => {
  throw new Error(`sui-sheet: ${feature} is deferred to a follow-up slice (formula engine)`)
}

export function createSheet(options: SheetOptions): Sheet {
  const workbook = liftWorkbook(options.workbook)

  // Active-sheet id slice — seeded from the workbook's active id (or the first
  // sheet). The interaction slices (selection / editing) live alongside it.
  const seedWb = workbook()
  const seedActiveId =
    options.initialState?.activeSheetId ?? seedWb.activeSheetId ?? seedWb.sheets[0]?.id ?? null
  const state = buildInteractionState(seedActiveId, options.initialState?.selection)

  const pipeline = buildPipeline(workbook, state)
  const { activeSheetDef, store, cellApi, geometry } = pipeline
  const selection = buildSelection(state, options.onSelectionChange)
  const editing = buildEditing(state)

  const emitCells = (edits: CellEdit[]): void => {
    if (edits.length > 0) options.onCellChange?.(edits)
  }

  // ── render surfaces ──
  const activeSheet = derived<SheetDef>(() => activeSheetDef() as SheetDef)
  const rowCount = pipeline.rowCount
  const colCount = pipeline.colCount

  // ── mutators (foundational) ──
  const setCell = (a1: string, data: CellData): void => {
    batch(() => {
      const before = store.read(a1) ?? {}
      store.write(a1, data)
      emitCells([{ a1, before, after: data }])
    })
  }

  const setCells = (edits: CellEdit[]): void => {
    batch(() => {
      for (const e of edits) store.write(e.a1, e.after)
      emitCells(edits)
    })
  }

  const clearCells = (range: CellRange, what: 'all' | 'values' | 'formats' = 'all'): void => {
    batch(() => {
      const edits: CellEdit[] = []
      for (const addr of range.cells()) {
        const a1 = toA1(addr)
        const before = store.read(a1) ?? {}
        const after: CellData =
          what === 'formats' ? { value: before.value ?? null } : {}
        store.write(a1, after)
        edits.push({ a1, before, after })
      }
      emitCells(edits)
    })
  }

  // ── editing → cell write bridge ──
  const startEditing = (a?: CellAddress, seed?: string): void => {
    editing.startEditing(state.selection().active, a, seed)
  }
  const commitEdit = (opts?: { advance?: 'down' | 'right' | 'none' }): void => {
    const committed = editing.commitEdit(opts)
    if (!committed) return
    setCell(committed.a1, committed.data)
    if (committed.advance !== 'none') {
      const dir = committed.advance === 'right' ? { col: 1, row: 0 } : { col: 0, row: 1 }
      selection.setActiveCell({
        row: committed.cell.row + dir.row,
        col: committed.cell.col + dir.col,
      })
    }
  }

  // ── structure ──
  const setFrozen = (rows: number, cols: number): void => {
    const def = activeSheetDef()
    if (def) {
      def.frozenRows = rows
      def.frozenCols = cols
    }
  }
  const setColWidth = (col: number, px: number): void => {
    const def = activeSheetDef()
    if (def) (def.colWidths ??= {})[col] = px
  }
  const setRowHeight = (row: number, px: number): void => {
    const def = activeSheetDef()
    if (def) (def.rowHeights ??= {})[row] = px
  }
  const setActiveSheet = (sheetId: string): void => {
    batch(() => {
      state.activeSheetId.set(sheetId)
      // A sheet switch resets selection to the origin of the new sheet.
      state.selection.set(emptySelection())
    })
    options.onActiveSheetChange?.(sheetId)
  }

  const reset = (): void => {
    batch(() => {
      state.selection.set(emptySelection())
      state.editing.set(null)
    })
  }

  return {
    activeSheet,
    rowCount,
    colCount,
    frozenRowCount: geometry.frozenRowCount,
    frozenColCount: geometry.frozenColCount,
    centerRowRange: geometry.centerRowRange,
    centerColRange: geometry.centerColRange,
    getCell: cellApi.getCell,
    getCellAt: cellApi.getCellAt,
    rowHeight: (i) => geometry.rowHeightAt(i),
    colWidth: (i) => geometry.colWidthAt(i),
    totalWidth: geometry.totalWidth,
    totalHeight: geometry.totalHeight,
    selection: selection.selection,
    activeCell: selection.activeCell,
    isInRange: selection.isInRange,
    dependents: () => derived<string[]>(() => []),
    precedents: () => derived<string[]>(() => []),
    hasCycle: derived<boolean>(() => false),
    isLoading: reflect(options.isLoading, false),
    error: reflect(options.error, null),
    editing: derived<{ cell: CellAddress; draft: string } | null>(() => state.editing()),
    setActiveCell: selection.setActiveCell,
    setSelection: selection.setSelection,
    moveActive: () => DEFERRED('moveActive keyboard nav'),
    startEditing,
    updateDraft: editing.updateDraft,
    commitEdit,
    cancelEdit: editing.cancelEdit,
    setCell,
    setCells,
    clearCells,
    fillRange: () => DEFERRED('fillRange (relative-reference series)'),
    setFormat: (range, fmt) => applyToRange(range, (c) => ({ ...c, format: { ...c.format, ...fmt } })),
    setStyle: (range, style) =>
      applyToRange(range, (c) => ({
        ...c,
        format: { ...c.format, style: { ...c.format?.style, ...style } },
      })),
    setFrozen,
    setColWidth,
    setRowHeight,
    setActiveSheet,
    recalc: () => {},
    reset,
    setOptions: () => DEFERRED('setOptions hot-swap'),
  }

  /** Apply a per-cell transform over a range, emitting one batched onCellChange. */
  function applyToRange(range: CellRange, fn: (c: CellData) => CellData): void {
    batch(() => {
      const edits: CellEdit[] = []
      for (const addr of range.cells()) {
        const a1 = toA1(addr)
        const before = store.read(a1) ?? {}
        const after = fn(before)
        store.write(a1, after)
        edits.push({ a1, before, after })
      }
      emitCells(edits)
    })
  }
}

// Re-exports so consumers import the engine + its types from one module.
export type { Sheet, SheetOptions } from './contract'
export { fromA1, toA1, makeRange, makeSelection }
export type { ResolvedCell }
