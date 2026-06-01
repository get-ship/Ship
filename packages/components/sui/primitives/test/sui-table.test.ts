import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-table/sui-table.compiled.js'
import { createTable } from '../src/sui-table/engine/create-table'
import type { ColumnDef } from '../src/sui-table/engine/types'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-table>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-table')).toBeDefined()
  })

  it('has a shadow root with .wrap', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    expect(shadow!.querySelector('.wrap')).not.toBeNull()
  })

  it('slot mode renders thead/tbody in shadow DOM', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    expect(shadow!.querySelector('thead')).not.toBeNull()
    expect(shadow!.querySelector('tbody')).not.toBeNull()
  })

  it('native mode renders wrap without inner table in shadow', () => {
    const { shadow } = mount(reg, 'sui-table', { variant: 'native' }, '<table><tbody><tr><td>A</td></tr></tbody></table>')
    expect(shadow!.querySelector('table')).toBeNull()
    expect(shadow!.querySelector('.wrap')).not.toBeNull()
  })

  // DELETED: `reflects dense attribute`. The `dense` prop is REMOVED
  // (build-spec REPLACE §"BOTH density props" + Slice 0). Density becomes
  // the independent platform axis (`<html data-density>`), not a component
  // prop; rows read `--density-row-py/-px/-gap` directly. The positive
  // assertion that rows read the `--density-*` channel and that NO
  // `density`/`dense` prop exists lands with the view rework (Slice 5 /
  // behavioral spec #9), NOT in this characterization net — this net pins
  // only the slot/native behavior that survives untouched.

  it('hoverable defaults true — not set to false initially', () => {
    const { el } = mount(reg, 'sui-table', {})
    // The runtime skips the initial reflect pass; attribute is absent by default.
    // The light-DOM CSS uses :not([hoverable='false']) so absent = hoverable enabled.
    expect(el.getAttribute('hoverable')).not.toBe('false')
  })

  it('reflects sticky-first-col attribute', () => {
    const { el } = mount(reg, 'sui-table', { 'sticky-first-col': true })
    expect(el.hasAttribute('sticky-first-col')).toBe(true)
  })

  it('reflects gutter attribute', () => {
    const { el } = mount(reg, 'sui-table', { gutter: true })
    expect(el.hasAttribute('gutter')).toBe(true)
  })

  // DELETED: `reflects density attribute`. The `density` prop is REMOVED
  // alongside `dense` (build-spec REPLACE §"BOTH density props"). See the
  // comment above — the density-channel assertion lands with the view
  // rework (Slice 5), not here.

  it('sui-row-click fires when a data-row-id row td is clicked', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th>Name</th></tr></thead><tbody><tr data-row-id="abc"><td>Alpha</td></tr></tbody></table>',
    )
    await flush()
    let detail: { id: string; index: number } | null = null
    el.addEventListener('sui-row-click', (e) => { detail = (e as CustomEvent).detail })
    ;(el.querySelector('td') as HTMLTableCellElement).click()
    expect(detail).not.toBeNull()
    expect(detail!.id).toBe('abc')
  })

  it('sui-row-click does not fire for sui-table-gutter-cell', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><tbody><tr data-row-id="abc"><td class="sui-table-gutter-cell"><input type="checkbox"></td><td>Name</td></tr></tbody></table>',
    )
    await flush()
    let fired = false
    el.addEventListener('sui-row-click', () => { fired = true })
    ;(el.querySelector('.sui-table-gutter-cell') as HTMLTableCellElement).click()
    expect(fired).toBe(false)
  })

  it('sui-row-click does not fire for rows without data-row-id', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><tbody><tr><td>No ID</td></tr></tbody></table>',
    )
    await flush()
    let fired = false
    el.addEventListener('sui-row-click', () => { fired = true })
    ;(el.querySelector('td') as HTMLTableCellElement).click()
    expect(fired).toBe(false)
  })

  it('enhances th[data-sort-key] with aria-sort, tabindex, role', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th data-sort-key="name">Name</th></tr></thead><tbody><tr><td>Alice</td></tr></tbody></table>',
    )
    await flush()
    const th = el.querySelector('th[data-sort-key]') as HTMLTableCellElement
    expect(th.getAttribute('aria-sort')).toBe('none')
    expect(th.getAttribute('tabindex')).toBe('0')
    expect(th.getAttribute('role')).toBe('columnheader')
  })

  it('sui-sort fires and cycles direction on sortable th click', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th data-sort-key="name">Name</th></tr></thead><tbody><tr><td>Alice</td></tr></tbody></table>',
    )
    await flush()
    const events: { key: string; direction: string }[] = []
    el.addEventListener('sui-sort', (e) => events.push((e as CustomEvent).detail))
    const th = el.querySelector('th[data-sort-key]') as HTMLTableCellElement
    th.click()
    expect(events[0]).toEqual({ key: 'name', direction: 'ascending' })
    th.click()
    expect(events[1]).toEqual({ key: 'name', direction: 'descending' })
    th.click()
    expect(events[2]).toEqual({ key: 'name', direction: 'none' })
  })

  it('slot mode has head slot in shadow thead', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    const headSlot = shadow!.querySelector('slot[name="head"]')
    expect(headSlot).not.toBeNull()
    expect(headSlot!.closest('thead')).not.toBeNull()
  })

  // ── Gap-fill: characterize the rest of the KEEP contract (build-spec
  // Slice 0 / KEEP §) so the slot/native net is actually pinned, not assumed.
  // These run against the CURRENT component and MUST be GREEN.

  it('exposes the wrap/table/thead/tbody parts in slot mode', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    expect(shadow!.querySelector('[part="wrap"]')).not.toBeNull()
    expect(shadow!.querySelector('[part="table"]')).not.toBeNull()
    expect(shadow!.querySelector('[part="thead"]')).not.toBeNull()
    expect(shadow!.querySelector('[part="tbody"]')).not.toBeNull()
  })

  it('slot mode has a default (body) slot inside the tbody part', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    const tbody = shadow!.querySelector('[part="tbody"]')
    expect(tbody).not.toBeNull()
    // The default slot (no name) carries body rows.
    expect(tbody!.querySelector('slot:not([name])')).not.toBeNull()
  })

  it('slot mode exposes the empty part + named empty slot', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    const emptyPart = shadow!.querySelector('[part="empty"]')
    expect(emptyPart).not.toBeNull()
    expect(emptyPart!.querySelector('slot[name="empty"]')).not.toBeNull()
  })

  it('native mode projects the light-DOM table through the default slot', () => {
    const { el, shadow } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><tbody><tr data-row-id="x"><td>Cell</td></tr></tbody></table>',
    )
    // native: no shadow <table>, the host keeps its own light-DOM table.
    expect(shadow!.querySelector('table')).toBeNull()
    expect(el.querySelector('table')).not.toBeNull()
    expect(shadow!.querySelector('slot:not([name])')).not.toBeNull()
  })

  it('reflects the aria-label from the label prop in slot mode', () => {
    const { shadow } = mount(reg, 'sui-table', { label: 'People' })
    const table = shadow!.querySelector('[part="table"]')
    expect(table!.getAttribute('aria-label')).toBe('People')
  })

  it('reflects the striped attribute', () => {
    const { el } = mount(reg, 'sui-table', { striped: true })
    expect(el.hasAttribute('striped')).toBe(true)
  })

  it('reflects the sticky-header attribute', () => {
    const { el } = mount(reg, 'sui-table', { 'sticky-header': true })
    expect(el.hasAttribute('sticky-header')).toBe(true)
  })

  it('reflects [data-empty] when an empty slot is filled and no body row exists', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      {},
      '<div slot="empty">Nothing here</div>',
    )
    await flush()
    expect(el.hasAttribute('data-empty')).toBe(true)
  })

  it('clears [data-empty] once a body row is present (native + empty slot)', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><tbody><tr data-row-id="r1"><td>One</td></tr></tbody></table>' +
        '<div slot="empty">Nothing here</div>',
    )
    await flush()
    expect(el.hasAttribute('data-empty')).toBe(false)
  })

  it('cycles sort via keyboard (Enter) on a sortable th', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th data-sort-key="name">Name</th></tr></thead><tbody><tr><td>A</td></tr></tbody></table>',
    )
    await flush()
    const events: { key: string; direction: string }[] = []
    el.addEventListener('sui-sort', (e) => events.push((e as CustomEvent).detail))
    const th = el.querySelector('th[data-sort-key]') as HTMLTableCellElement
    th.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(events[0]).toEqual({ key: 'name', direction: 'ascending' })
    expect(th.getAttribute('aria-sort')).toBe('ascending')
  })

  it('sui-sort bubbles + crosses the shadow boundary (composed)', async () => {
    const { el, container } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th data-sort-key="name">Name</th></tr></thead><tbody><tr><td>A</td></tr></tbody></table>',
    )
    await flush()
    let caught: CustomEvent | null = null
    // Listen on the ANCESTOR container — only a bubbling, composed event reaches it.
    container.addEventListener('sui-sort', (e) => { caught = e as CustomEvent })
    ;(el.querySelector('th[data-sort-key]') as HTMLTableCellElement).click()
    expect(caught).not.toBeNull()
    expect(caught!.detail).toEqual({ key: 'name', direction: 'ascending' })
  })

  it('sui-row-click reports the row index in its detail', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><tbody><tr data-row-id="abc"><td>Alpha</td></tr><tr data-row-id="def"><td>Beta</td></tr></tbody></table>',
    )
    await flush()
    let detail: { id: string; index: number } | null = null
    el.addEventListener('sui-row-click', (e) => { detail = (e as CustomEvent).detail })
    const cells = el.querySelectorAll('td')
    ;(cells[1] as HTMLTableCellElement).click()
    expect(detail).not.toBeNull()
    expect(detail!.id).toBe('def')
    expect(typeof detail!.index).toBe('number')
  })

  it('enhancement injects aria-sort/tabindex/role on a th added AFTER mount (MutationObserver)', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr></tr></thead><tbody><tr><td>A</td></tr></tbody></table>',
    )
    await flush()
    const headerRow = el.querySelector('thead tr') as HTMLTableRowElement
    const th = document.createElement('th')
    th.setAttribute('data-sort-key', 'late')
    th.textContent = 'Late'
    headerRow.appendChild(th)
    await flush()
    expect(th.getAttribute('aria-sort')).toBe('none')
    expect(th.getAttribute('tabindex')).toBe('0')
    expect(th.getAttribute('role')).toBe('columnheader')
  })
})

// ─────────────────────────────────────────────────────────────────────────────
// Data variant — VIEW behavioral specs (build-spec Slice 5 + 7c, "The view",
// "Behavioral spec list"). The engine reaches the view via `prop:table={table}`
// IDENTITY binding (never an object/json attribute): we assign the live
// `createTable()` instance onto the element property. These specs are queried
// by what a USER perceives (rendered rows, aria-sort, selection, filters) and
// are RED until the data variant is rebuilt behind the engine.
//
// REPLACES the old `data mode` describe block (Slice 5's explicit before/after):
//   `renders rows from rows prop`        → engine rows keyed by [data-row-id]
//   `clicking a column header sorts`     → engine toggleSort + numeric order
//   `clicking a row toggles selected …`  → [data-selected] attr + sui-row-select
// The slot/native characterization net above is UNTOUCHED and stays GREEN.

interface Person extends Record<string, unknown> {
  id: string
  name: string
  age: number
}

const PERSON_COLS: ColumnDef<Person>[] = [
  { id: 'name', accessorKey: 'name', header: 'Name' },
  { id: 'age', accessorKey: 'age', header: 'Age' },
]

/** Build a data-variant <sui-table> with a live engine bound via prop:table. */
function mountWithEngine<T extends Record<string, unknown> & { id: string }>(
  data: T[],
  columns: ColumnDef<T>[],
  attrs: Record<string, string | boolean> = {},
) {
  const table = createTable<T>({
    data,
    columns,
    getRowId: (r) => r.id,
    enableRowSelection: true,
  })
  const m = mount(reg, 'sui-table', { variant: 'data', ...attrs })
  // prop:table IDENTITY binding — the engine is a live JS reference, never a
  // stringified attribute. The controller wires the host `table` property to
  // the data-variant render.
  ;(m.el as any).table = table
  return { ...m, table }
}

const rowIds = (shadow: ShadowRoot | null): string[] =>
  Array.from(shadow!.querySelectorAll('[data-row-id]')).map(
    (r) => r.getAttribute('data-row-id') as string,
  )

describe('<sui-table> data variant — engine-backed view', () => {
  it('renders one [data-row-id] row per engine.centerRows()', async () => {
    const { shadow, table } = mountWithEngine(
      [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
      ],
      PERSON_COLS,
    )
    await flush()
    const rows = shadow!.querySelectorAll('[data-row-id]')
    expect(rows.length).toBe(table.centerRows().length)
    expect(rowIds(shadow)).toEqual(['1', '2'])
  })

  it('grows the rendered rows when the engine data signal grows (no remount)', async () => {
    const { el, shadow, table } = mountWithEngine(
      [{ id: '1', name: 'Alice', age: 30 }],
      PERSON_COLS,
    )
    await flush()
    expect(rowIds(shadow)).toEqual(['1'])
    const before = el
    table.setOptions({
      data: [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
        { id: '3', name: 'Cara', age: 40 },
      ],
    })
    await flush()
    expect(el).toBe(before) // same element — grew in place
    expect(rowIds(shadow)).toEqual(['1', '2', '3'])
  })

  it('renders [data-sort-key] on headers + a sui-icon sort caret (not inline svg)', async () => {
    const { shadow } = mountWithEngine([{ id: '1', name: 'Alice', age: 30 }], PERSON_COLS)
    await flush()
    const keys = Array.from(shadow!.querySelectorAll('[data-sort-key]')).map((h) =>
      h.getAttribute('data-sort-key'),
    )
    expect(keys).toContain('name')
    expect(keys).toContain('age')
    // The caret composes sui-icon — the controller's inline <svg> is replaced.
    expect(shadow!.querySelector('th svg')).toBeNull()
    expect(shadow!.querySelector('sui-icon')).not.toBeNull()
  })

  it('numeric sort header orders 2,10,100 numerically + cycles aria-sort', async () => {
    const { shadow } = mountWithEngine(
      [
        { id: 'a', name: 'A', age: 10 },
        { id: 'b', name: 'B', age: 2 },
        { id: 'c', name: 'C', age: 100 },
      ],
      PERSON_COLS,
    )
    await flush()
    const ageTh = shadow!.querySelector('[data-sort-key="age"]') as HTMLElement
    expect(ageTh.getAttribute('aria-sort')).toBe('none')

    ageTh.click()
    await flush()
    expect(ageTh.getAttribute('aria-sort')).toBe('ascending')
    expect(rowIds(shadow)).toEqual(['b', 'a', 'c']) // 2, 10, 100 — NOT '10' < '2'

    ageTh.click()
    await flush()
    expect(ageTh.getAttribute('aria-sort')).toBe('descending')
    expect(rowIds(shadow)).toEqual(['c', 'a', 'b']) // 100, 10, 2

    ageTh.click()
    await flush()
    expect(ageTh.getAttribute('aria-sort')).toBe('none')
    expect(rowIds(shadow)).toEqual(['a', 'b', 'c']) // unsorted (original order)
  })

  it('emits sui-sort-change with the multi-sort state on header click', async () => {
    const { el, shadow } = mountWithEngine([{ id: '1', name: 'A', age: 1 }], PERSON_COLS)
    await flush()
    const events: { sorting: { id: string; desc: boolean }[] }[] = []
    el.addEventListener('sui-sort-change', (e) =>
      events.push((e as CustomEvent).detail),
    )
    ;(shadow!.querySelector('[data-sort-key="name"]') as HTMLElement).click()
    await flush()
    expect(events.length).toBeGreaterThan(0)
    expect(events.at(-1)!.sorting).toEqual([{ id: 'name', desc: false }])
  })

  it('sui-sort-change bubbles + crosses the shadow boundary (composed)', async () => {
    const { shadow, container } = mountWithEngine(
      [{ id: '1', name: 'A', age: 1 }],
      PERSON_COLS,
    )
    await flush()
    let caught: CustomEvent | null = null
    container.addEventListener('sui-sort-change', (e) => {
      caught = e as CustomEvent
    })
    ;(shadow!.querySelector('[data-sort-key="name"]') as HTMLElement).click()
    await flush()
    expect(caught).not.toBeNull()
  })

  it('toggling a gutter sui-checkbox sets [data-selected] + emits sui-row-select', async () => {
    const { el, shadow } = mountWithEngine(
      [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
      ],
      PERSON_COLS,
      { gutter: true, selectable: 'multi' },
    )
    await flush()
    const detail: { id: string; selected: boolean; selectedIds: string[] }[] = []
    el.addEventListener('sui-row-select', (e) => detail.push((e as CustomEvent).detail))

    const firstRow = shadow!.querySelector('[data-row-id="1"]') as HTMLElement
    expect(firstRow.hasAttribute('data-selected')).toBe(false)
    const box = firstRow.querySelector('sui-checkbox') as HTMLElement
    expect(box).not.toBeNull()
    box.click()
    await flush()

    expect(firstRow.hasAttribute('data-selected')).toBe(true)
    expect(detail.length).toBeGreaterThan(0)
    expect(detail.at(-1)!.id).toBe('1')
    expect(detail.at(-1)!.selected).toBe(true)
    expect(detail.at(-1)!.selectedIds).toContain('1')
  })

  it('header sui-checkbox shows indeterminate on partial selection', async () => {
    const { shadow, table } = mountWithEngine(
      [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
      ],
      PERSON_COLS,
      { gutter: true, selectable: 'multi' },
    )
    await flush()
    const headerBox = shadow!.querySelector('thead sui-checkbox') as HTMLElement
    expect(headerBox).not.toBeNull()
    table.toggleRowSelected('1', true)
    await flush()
    // partial → indeterminate (one of two selected), not fully checked
    expect(headerBox.hasAttribute('indeterminate')).toBe(true)
    expect(headerBox.hasAttribute('checked')).toBe(false)
  })

  it('a per-row-disabled row cannot be selected', async () => {
    const table = createTable<Person>({
      data: [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
      ],
      columns: PERSON_COLS,
      getRowId: (r) => r.id,
      enableRowSelection: (row) => row.original.id !== '2',
    })
    const { el, shadow } = mount(reg, 'sui-table', {
      variant: 'data',
      gutter: true,
      selectable: 'multi',
    })
    ;(el as any).table = table
    await flush()
    const secondRow = shadow!.querySelector('[data-row-id="2"]') as HTMLElement
    const box = secondRow.querySelector('sui-checkbox') as HTMLElement
    box.click()
    await flush()
    expect(secondRow.hasAttribute('data-selected')).toBe(false)
    expect(table.selectedRowIds()).not.toContain('2')
  })

  it('a column filter (sui-input) narrows the rendered rows; clearing restores', async () => {
    const cols: ColumnDef<Person>[] = [
      { id: 'name', accessorKey: 'name', header: 'Name', enableColumnFilter: true },
      { id: 'age', accessorKey: 'age', header: 'Age' },
    ]
    const { shadow, table } = mountWithEngine(
      [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
        { id: '3', name: 'Albert', age: 40 },
      ],
      cols,
    )
    await flush()
    expect(rowIds(shadow)).toEqual(['1', '2', '3'])

    // The view debounces user input then calls engine.setColumnFilter. Drive the
    // engine directly so the spec is robust to the (view-owned) debounce timing;
    // the rendered window must reflect the narrowed set.
    table.setColumnFilter('name', 'Al')
    await flush()
    expect(rowIds(shadow)).toEqual(['1', '3'])

    table.setColumnFilter('name', '')
    await flush()
    expect(rowIds(shadow)).toEqual(['1', '2', '3'])
  })

  it('renders a sui-input column-filter affordance for a filterable column', async () => {
    const cols: ColumnDef<Person>[] = [
      { id: 'name', accessorKey: 'name', header: 'Name', enableColumnFilter: true },
    ]
    const { shadow } = mountWithEngine([{ id: '1', name: 'Alice', age: 30 }], cols)
    await flush()
    expect(shadow!.querySelector('sui-input')).not.toBeNull()
  })

  it('empty state branches initial (no core rows) — sui-empty-state + [data-empty]', async () => {
    const { el, shadow } = mountWithEngine([], PERSON_COLS)
    await flush()
    expect(el.hasAttribute('data-empty')).toBe(true)
    const empty = shadow!.querySelector('sui-empty-state') as HTMLElement
    expect(empty).not.toBeNull()
    // initial-empty: no "clear filters" affordance.
    expect(shadow!.querySelector('[data-action="clear-filters"]')).toBeNull()
  })

  it('empty state branches filtered (data present, filter empties) — clear-filters reset', async () => {
    const cols: ColumnDef<Person>[] = [
      { id: 'name', accessorKey: 'name', header: 'Name', enableColumnFilter: true },
    ]
    const { shadow, table } = mountWithEngine(
      [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
      ],
      cols,
    )
    await flush()
    table.setColumnFilter('name', 'zzz-nothing-matches')
    await flush()
    expect(rowIds(shadow).length).toBe(0)
    expect(table.hasActiveFilters()).toBe(true)
    // filtered-empty branch: a working "clear filters" affordance → resetFilters().
    const clear = shadow!.querySelector('[data-action="clear-filters"]') as HTMLElement
    expect(clear).not.toBeNull()
    clear.click()
    await flush()
    expect(table.hasActiveFilters()).toBe(false)
    expect(rowIds(shadow)).toEqual(['1', '2'])
  })

  it('density is the platform axis: <html data-density> drives row padding — NO density/dense prop', async () => {
    const { el } = mountWithEngine([{ id: '1', name: 'Alice', age: 30 }], PERSON_COLS)
    await flush()
    // The removed props must NOT be observed attributes nor reflect.
    el.setAttribute('density', 'compact')
    el.setAttribute('dense', '')
    await flush()
    // The component declares neither prop — rows read the platform --density-*
    // channel from <html data-density> directly. Assert the props are absent
    // from the component's reflected surface (a `density`/`dense` prop would
    // round-trip; these are inert leftover attributes only).
    const proto = Object.getPrototypeOf(el) as object
    expect(Object.getOwnPropertyNames(proto)).not.toContain('density')
    expect(Object.getOwnPropertyNames(proto)).not.toContain('dense')
  })

  it('virtualizes a large dataset: renders ~viewport+overscan rows with honest totalSize', async () => {
    const big: Person[] = Array.from({ length: 100_000 }, (_, i) => ({
      id: String(i),
      name: `Person ${i}`,
      age: i % 99,
    }))
    const { shadow } = mountWithEngine(big, PERSON_COLS)
    await flush()
    const rendered = shadow!.querySelectorAll('[data-row-id]')
    // Only a window renders — NOT all 100k rows.
    expect(rendered.length).toBeGreaterThan(0)
    expect(rendered.length).toBeLessThan(500)
    // The spacer height encodes the FULL dataset so the scrollbar is honest.
    const spacer = shadow!.querySelector('.spacer') as HTMLElement
    expect(spacer).not.toBeNull()
    const h = parseFloat(spacer.style.height || '0')
    expect(h).toBeGreaterThan(10_000)
    // aria-rowcount reports the TOTAL, not the windowed DOM length.
    const grid = shadow!.querySelector('[role="grid"],[role="treegrid"]') as HTMLElement
    expect(grid).not.toBeNull()
    expect(grid.getAttribute('aria-rowcount')).toBe('100000')
  })

  it('keyboard nav uses roving-tabindex (one focusable cell/row at a time)', async () => {
    const { shadow } = mountWithEngine(
      [
        { id: '1', name: 'Alice', age: 30 },
        { id: '2', name: 'Bob', age: 25 },
        { id: '3', name: 'Cara', age: 40 },
      ],
      PERSON_COLS,
    )
    await flush()
    const grid = shadow!.querySelector('[role="grid"],[role="treegrid"]') as HTMLElement
    expect(grid).not.toBeNull()
    const tabbable = () =>
      Array.from(shadow!.querySelectorAll('[data-row-id]')).filter(
        (r) => r.getAttribute('tabindex') === '0',
      )
    // roving-tabindex: exactly one active row/cell carries tabindex=0.
    expect(tabbable().length).toBe(1)
    grid.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    await flush()
    expect(tabbable().length).toBe(1)
    // Focus moved to the second row.
    expect(tabbable()[0]!.getAttribute('data-row-id')).toBe('2')
  })
})

// ─────────────────────────────────────────────────────────────────────────────
// Pagination is verified by DIRECT engine calls (the sui-pagination UI is a
// deferred follow-up — Slice 6). Slice 5 asserts the engine pagination window
// reflects into the rendered rows.

describe('<sui-table> data variant — pagination (direct engine drive)', () => {
  const PEOPLE: Person[] = Array.from({ length: 7 }, (_, i) => ({
    id: String(i),
    name: `P${i}`,
    age: i,
  }))

  it('setPageSize + setPageIndex window the rendered rows', async () => {
    const { shadow, table } = mountWithEngine(PEOPLE, PERSON_COLS)
    await flush()
    table.setPageSize(3)
    await flush()
    expect(rowIds(shadow)).toEqual(['0', '1', '2'])
    expect(table.getCanPreviousPage()).toBe(false)
    expect(table.getCanNextPage()).toBe(true)

    table.setPageIndex(1)
    await flush()
    expect(rowIds(shadow)).toEqual(['3', '4', '5'])

    table.setPageIndex(2)
    await flush()
    expect(rowIds(shadow)).toEqual(['6'])
    expect(table.getCanNextPage()).toBe(false)
  })
})
