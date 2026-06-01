import { controller, derived, effect, ref, signal, type Derived } from '@ship/sui-runtime'
import { createVirtualizer } from '@ship/sui-runtime'
import { createSheet, type Sheet } from './engine/create-sheet'
import { colToLabel } from './engine/a1'
import { parseJsonWorkbook } from './engine/decode'
import type { ResolvedCell } from './engine/contract'

/** A windowed column (header + geometry) keyed by logical index. */
interface ColView {
  index: number
  label: string
  left: number
  width: number
}
/** A windowed row (header + geometry) keyed by logical index. */
interface RowView {
  index: number
  label: string
  top: number
  height: number
}
/** A windowed grid cell — the resolved render contract + its placement. */
interface CellView {
  key: string
  a1: string
  row: number
  col: number
  top: number
  left: number
  width: number
  height: number
  cell: Derived<ResolvedCell>
}

/** An empty single-sheet workbook so the template always has a Sheet to read
 *  before the consumer assigns `prop:sheet` (the identity path) or the
 *  `workbook` SSR string fallback seeds one. */
function emptySheet(): Sheet {
  return createSheet({
    workbook: { sheets: [{ id: '', name: '', rowCount: 0, colCount: 0, cells: {} }] },
  })
}

export default controller({
  scope: () => {
    // The live engine. Null until the consumer assigns the `sheet` host property
    // (wired in mount) OR the `workbook` SSR string is parsed into one.
    const sheetSig = signal<Sheet | null>(null)
    const viewportEl = ref<HTMLDivElement>(null)

    // A stable empty engine backs every read so the template never branches on
    // null; the active engine is `sheetSig() ?? EMPTY`.
    const EMPTY = emptySheet()
    const sheet = derived<Sheet>(() => sheetSig() ?? EMPTY)

    const rowCount = derived<number>(() => sheet().rowCount())
    const colCount = derived<number>(() => sheet().colCount())
    const frozenRows = derived<number>(() => sheet().frozenRowCount())
    const frozenCols = derived<number>(() => sheet().frozenColCount())
    const totalHeight = derived<number>(() => sheet().totalHeight())
    const totalWidth = derived<number>(() => sheet().totalWidth())

    // The windowed center bands (the virtualizers count these; frozen bands sit
    // OUTSIDE). The two virtualizers are created in `mount` once the viewport
    // element exists; their windows — logical index + pixel placement — feed
    // `cols`/`rows`, which carry both the header label and the geometry.
    const cols = signal<ColView[]>([])
    const rows = signal<RowView[]>([])

    // The windowed cell matrix: every (rows × cols) intersection, each carrying
    // its own `Derived<ResolvedCell>` from the engine so a per-cell edit
    // re-renders only that cell. Keyed by A1 for the {#for} reconcile.
    const cells = derived<CellView[]>(() => {
      const s = sheet()
      const out: CellView[] = []
      for (const r of rows()) {
        for (const c of cols()) {
          const cell = s.getCellAt(r.index, c.index)
          out.push({
            key: cell().a1,
            a1: cell().a1,
            row: r.index,
            col: c.index,
            top: r.top,
            left: c.left,
            width: c.width,
            height: r.height,
            cell,
          })
        }
      }
      return out
    })

    return {
      sheetSig,
      viewportEl,
      sheet,
      rowCount,
      colCount,
      frozenRows,
      frozenCols,
      totalHeight,
      totalWidth,
      cols,
      rows,
      cells,
    }
  },

  mount(state, host) {
    // ── prop:sheet identity wiring ──────────────────────────────────────────
    // The engine is a LIVE JS reference assigned to the host `sheet` property
    // (never a stringified attribute). Route it into the scope signal so the
    // windowed render reacts. The `workbook` string prop is the SSR fallback.
    Object.defineProperty(host, 'sheet', {
      configurable: true,
      get: () => state.sheetSig,
      set: (s: Sheet | null) => {
        state.sheetSig = s
      },
    })

    // SSR fallback: parse the `workbook` string prop into an engine when no
    // identity engine was assigned (the sui-combobox JSON-string pattern).
    if (state.sheetSig == null) {
      const raw = (host as { workbook?: string }).workbook ?? host.getAttribute('workbook') ?? ''
      const wb = parseJsonWorkbook(raw)
      if (wb && wb.sheets.length > 0) state.sheetSig = createSheet({ workbook: wb })
    }

    // ── two-axis virtualization (the Backbone #2 stress) ─────────────────────
    // Two independent virtualizers over ONE viewport: rows (vertical) + columns
    // (horizontal). Each counts the engine's center span (frozen bands excluded)
    // and writes its window into the scope signal the template renders.
    const getViewport = (): HTMLDivElement | null => state.viewportEl
    const rowV = createVirtualizer({
      count: () => {
        const r = state.sheet.centerRowRange()
        return Math.max(0, r.end - r.start)
      },
      estimateSize: (i) => state.sheet.rowHeight(i + state.sheet.centerRowRange().start),
      getScrollElement: getViewport,
      overscan: 4,
    })
    const colV = createVirtualizer({
      count: () => {
        const c = state.sheet.centerColRange()
        return Math.max(0, c.end - c.start)
      },
      estimateSize: (i) => state.sheet.colWidth(i + state.sheet.centerColRange().start),
      getScrollElement: getViewport,
      horizontal: true,
      overscan: 4,
    })

    // Bridge each virtualizer's window into the scope signals. The offset re-adds
    // the frozen-band lead so the rendered indices are the LOGICAL ones; the
    // pixel start/size become the absolute placement the template positions on.
    const syncRows = (): void => {
      const lead = state.sheet.centerRowRange().start
      state.rows = rowV.virtualItems().map((it) => {
        const index = it.index + lead
        return { index, label: String(index + 1), top: it.start, height: it.size }
      })
    }
    const syncCols = (): void => {
      const lead = state.sheet.centerColRange().start
      state.cols = colV.virtualItems().map((it) => {
        const index = it.index + lead
        return { index, label: colToLabel(index), left: it.start, width: it.size }
      })
    }
    // An effect reading each virtualizer's `virtualItems()` subscribes to its
    // window; on change it republishes the logical-index slice into the scope
    // signal the template renders.
    const stopRowSync = effect(() => {
      rowV.virtualItems()
      syncRows()
    })
    const stopColSync = effect(() => {
      colV.virtualItems()
      syncCols()
    })

    return () => {
      stopRowSync()
      stopColSync()
      rowV.dispose()
      colV.dispose()
    }
  },
})
