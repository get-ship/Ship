import { controller, signal, derived, bindNativeEvent } from '@ship/ui-runtime'
import type { Scope } from './sui-table.sui'
import type { Signal } from '@ship/ui-runtime'

type Row = Record<string, unknown> & { id: string }
type Column = { field: string; label: string }
type SortDirection = 'ascending' | 'descending' | 'none'

const lightDomInstalled = new WeakSet<Document>()

function installLightDomStyles(doc: Document) {
  if (lightDomInstalled.has(doc)) return
  lightDomInstalled.add(doc)
  try {
    if (typeof CSSStyleSheet !== 'undefined' && 'replaceSync' in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet()
      sheet.replaceSync(SUI_TABLE_LIGHT_DOM_CSS)
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet]
      return
    }
  } catch {
    /* fall through */
  }
  const style = doc.createElement('style')
  style.setAttribute('data-sui-table', '')
  style.textContent = SUI_TABLE_LIGHT_DOM_CSS
  doc.head.appendChild(style)
}

export default controller<Scope>({
  scope: () => {
    const rows = signal<Row[]>([])
    const columns = signal<Column[]>([])
    const sortKey = signal<string | null>(null)
    const sortDir = signal<'asc' | 'desc'>('asc')
    const selected = signal<string[]>([])
    const sortedRows = derived(() => {
      const key = sortKey()
      const dir = sortDir()
      const data = rows()
      if (!key) return data
      return [...data].sort((a, b) => {
        const av = String(a[key] ?? '')
        const bv = String(b[key] ?? '')
        const cmp = av < bv ? -1 : av > bv ? 1 : 0
        return dir === 'asc' ? cmp : -cmp
      })
    })
    return { rows, columns, sortKey, sortDir, selected, sortedRows }
  },

  on: {
    sortBy(s: any, field: string) {
      if (s.sortKey === field) {
        s.sortDir = s.sortDir === 'asc' ? 'desc' : 'asc'
      } else {
        s.sortKey = field
        s.sortDir = 'asc'
      }
    },
    toggleSelect(s: any, id: string) {
      const cur = s.selected as string[]
      s.selected = cur.includes(id) ? cur.filter((x: string) => x !== id) : [...cur, id]
    },
    selectAll(s: any) {
      s.selected = (s.rows as Row[]).map((r) => r.id)
    },
    clearSelection(s: any) {
      s.selected = []
    },
  },

  mount(state, host) {
    const scope = state as unknown as Scope & {
      rows: Signal<Row[]>; columns: Signal<Column[]>
    }

    // Wire rows/columns JS properties to their signals so parent can do
    // prop:rows={list} / prop:columns={cols} (bindProp sets el.rows = value).
    const rowsSignal = scope.rows as unknown as Signal<Row[]>
    const colsSignal = scope.columns as unknown as Signal<Column[]>

    const initRows = (host as any).rows
    if (initRows != null) rowsSignal.set(initRows)
    const initCols = (host as any).columns
    if (initCols != null) colsSignal.set(initCols)

    Object.defineProperty(host, 'rows', {
      configurable: true,
      get: () => rowsSignal(),
      set: (v: Row[]) => rowsSignal.set(v),
    })
    Object.defineProperty(host, 'columns', {
      configurable: true,
      get: () => colsSignal(),
      set: (v: Column[]) => colsSignal.set(v),
    })

    installLightDomStyles(host.ownerDocument)

    const paintSortChevrons = () => {
      host.querySelectorAll<HTMLTableCellElement>('th[data-sort-key]').forEach((th) => {
        const dir = (th.getAttribute('aria-sort') || 'none') as SortDirection
        const ind = th.querySelector<HTMLElement>('.sui-sort-indicator')
        if (!ind) return
        if (dir === 'ascending') {
          ind.style.transform = 'rotate(180deg)'
          ind.style.opacity = '1'
        } else if (dir === 'descending') {
          ind.style.transform = 'rotate(0deg)'
          ind.style.opacity = '1'
        } else {
          ind.style.transform = 'rotate(0deg)'
          ind.style.opacity = '0.4'
        }
      })
    }

    const enhanceSortableHeaders = () => {
      host.querySelectorAll<HTMLTableCellElement>('th[data-sort-key]').forEach((th) => {
        if (!th.hasAttribute('aria-sort')) th.setAttribute('aria-sort', 'none')
        if (!th.hasAttribute('tabindex')) th.setAttribute('tabindex', '0')
        if (!th.hasAttribute('role')) th.setAttribute('role', 'columnheader')
        if (!th.querySelector('.sui-sort-indicator')) {
          const chev = document.createElement('span')
          chev.className = 'sui-sort-indicator'
          chev.setAttribute('aria-hidden', 'true')
          chev.style.cssText =
            'margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;'
          const svgNS = 'http://www.w3.org/2000/svg'
          const svg = document.createElementNS(svgNS, 'svg')
          svg.setAttribute('width', '10')
          svg.setAttribute('height', '10')
          svg.setAttribute('viewBox', '0 0 10 10')
          svg.setAttribute('fill', 'none')
          svg.setAttribute('stroke', 'currentColor')
          svg.setAttribute('stroke-width', '1.5')
          svg.setAttribute('stroke-linecap', 'round')
          svg.setAttribute('stroke-linejoin', 'round')
          svg.setAttribute('aria-hidden', 'true')
          const path = document.createElementNS(svgNS, 'path')
          path.setAttribute('d', 'M2 4l3-2 3 2M2 6l3 2 3-2')
          svg.appendChild(path)
          chev.appendChild(svg)
          th.appendChild(chev)
        }
      })
      paintSortChevrons()
    }

    const syncEmptyState = () => {
      const bodyRow =
        host.querySelector(':scope > table tbody > tr') ||
        host.querySelector(':scope > table > tr:not(:has(th))') ||
        host.querySelector(':scope > tbody > tr') ||
        host.querySelector(':scope > tr:not([slot="head"])')
      const hasEmptySlot = host.querySelector('[slot="empty"]') != null
      if (!bodyRow && hasEmptySlot) host.setAttribute('data-empty', '')
      else host.removeAttribute('data-empty')
    }

    const cycleSort = (th: HTMLTableCellElement) => {
      const key = th.getAttribute('data-sort-key') || ''
      const current = (th.getAttribute('aria-sort') || 'none') as SortDirection
      const next: SortDirection =
        current === 'none' ? 'ascending' : current === 'ascending' ? 'descending' : 'none'
      host.querySelectorAll<HTMLTableCellElement>('th[data-sort-key]').forEach((peer) => {
        if (peer !== th) peer.setAttribute('aria-sort', 'none')
      })
      th.setAttribute('aria-sort', next)
      paintSortChevrons()
      host.emit!('sui-sort', { key, direction: next })
    }

    const onHeaderClick = (e: MouseEvent) => {
      const th = (e.target as HTMLElement).closest(
        'th[data-sort-key]',
      ) as HTMLTableCellElement | null
      if (!th || !host.contains(th)) return
      cycleSort(th)
    }

    const onHeaderKey = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return
      const th = (e.target as HTMLElement).closest(
        'th[data-sort-key]',
      ) as HTMLTableCellElement | null
      if (!th || !host.contains(th)) return
      e.preventDefault()
      cycleSort(th)
    }

    const onRowClick = (e: MouseEvent) => {
      const td = (e.target as HTMLElement).closest('td') as HTMLTableCellElement | null
      if (!td || !host.contains(td)) return
      if (td.classList.contains('sui-table-gutter-cell')) return
      const tr = td.closest('tr') as HTMLTableRowElement | null
      if (!tr) return
      const rowId = tr.getAttribute('data-row-id')
      if (!rowId) return
      host.emit!('sui-row-click', { id: rowId, index: tr.rowIndex })
    }

    bindNativeEvent(host, 'click', onHeaderClick as EventListener)
    bindNativeEvent(host, 'click', onRowClick as EventListener)
    bindNativeEvent(host, 'keydown', onHeaderKey as EventListener)

    enhanceSortableHeaders()
    syncEmptyState()

    const observer = new MutationObserver(() => {
      enhanceSortableHeaders()
      syncEmptyState()
    })
    observer.observe(host, { childList: true, subtree: true })

    return () => {
      observer.disconnect()
    }
  },
})

const SUI_TABLE_LIGHT_DOM_CSS = `
  sui-table {
    --sui-table-border-soft: color-mix(in oklch, var(--border) 80%, transparent);
  }

  sui-table[variant='native'] > table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  sui-table th,
  sui-table td {
    text-align: left;
    padding: var(--row-py, 10px) var(--row-px, 12px);
    vertical-align: middle;
    border-bottom: 1px solid var(--sui-table-border-soft);
    font-size: var(--text-base, 13px);
    color: var(--foreground);
    box-sizing: border-box;
  }
  sui-table th {
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
  sui-table thead {
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table > tr:last-of-type > *,
  sui-table tbody tr:last-child > * {
    border-bottom: 0;
  }

  sui-table th[align='right'], sui-table td[align='right'] { text-align: right; }
  sui-table th[align='center'], sui-table td[align='center'] { text-align: center; }

  sui-table .mono {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-sm, 12px);
    letter-spacing: -0.01em;
  }
  sui-table .muted { color: var(--muted-foreground); }
  sui-table .null { color: var(--muted-foreground); font-style: italic; }

  sui-table > tr:not([slot='head']),
  sui-table tbody tr {
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[striped] > tr:not([slot='head']):nth-of-type(even) > *,
  sui-table[striped] tbody tr:nth-child(even) > * {
    background: color-mix(in oklch, var(--muted) 25%, transparent);
  }
  sui-table[striped]:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table[striped]:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table tr[data-selected] > *,
  sui-table tr[aria-selected='true'] > * {
    background: var(--tone-primary-bg);
  }
  sui-table tr[data-selected] > *:first-child,
  sui-table tr[aria-selected='true'] > *:first-child {
    box-shadow: inset 3px 0 0 0 var(--primary);
  }

  sui-table[sticky-header] thead th,
  sui-table[sticky-header] tr[slot='head'] > th {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 20);
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
    box-shadow: 0 1px 0 0 var(--border);
  }

  sui-table th[data-sort-key],
  sui-table th.sortable {
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition: color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table th[data-sort-key]:hover,
  sui-table th.sortable:hover {
    color: var(--foreground);
  }
  sui-table th[aria-sort='ascending'],
  sui-table th[aria-sort='descending'] {
    color: var(--foreground);
  }
  sui-table th[data-sort-key]:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 var(--ring-width, 3px)
      color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 25%), transparent);
  }

  sui-table[data-empty] > table {
    display: none;
  }

  sui-table[sticky-first-col] th:first-child,
  sui-table[sticky-first-col] td:first-child {
    position: sticky;
    left: 0;
    z-index: 1;
    background: var(--card);
    border-right: 1px solid var(--border);
  }
  sui-table[sticky-first-col] thead th:first-child {
    z-index: 3;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table[sticky-first-col] tbody tr:hover td:first-child {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[gutter] th.sui-table-gutter-cell,
  sui-table[gutter] td.sui-table-gutter-cell {
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
  sui-table[gutter] thead th.sui-table-gutter-cell { z-index: 4; }

  sui-table tr[data-row-id] > td:not(.sui-table-gutter-cell) {
    cursor: pointer;
  }

  @media (forced-colors: active) {
    sui-table th, sui-table td { border-bottom-color: CanvasText; }
  }
`
