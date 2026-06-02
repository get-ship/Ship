// <sui-calendar> controller — Slice 2 (the VIEW). Glue-only / tier-2.
//
// The headless engine (`./engine/createCalendar`) owns ALL reactive state +
// view-model derivation (month/week/day, agenda, selection, navigation). This
// controller is the thin seam: it routes the live `Calendar` in via the
// `prop:calendar` IDENTITY path (a JS reference, never a stringified attribute),
// falls back to building an engine from the `events` `type="string"` SSR prop
// (sui-combobox JSON.parse pattern), re-exposes the engine's `Derived` surfaces
// as scope signals the template reads, and emits `sui-*` lifecycle events via
// the runtime `host.emit!` helper (NOT open-coded `dispatchEvent` — the
// date-picker debt this pack avoids).
//
// Cardinals honored: mutations only in on:* / mount; no fetch / ship.* / <data>
// (the engine is pure, asserted by engine/purity.test.ts). Drag-to-create/move
// and range selection are an explicit v1 GAP — the committed engine exposes
// view/navigation/selection only (see contract.ts); no fake drag scaffold is
// shipped here. They land when the engine grows the drag/range slices.

import { controller, derived, effect, signal } from '@ship/sui-runtime'
import { createCalendar } from './engine'
import type { Calendar, CalendarEvent, MonthCell, ViewMode } from './engine'
import type { Scope } from './sui-calendar.sui'

const EMPTY: CalendarEvent[] = []
const DOW_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

/** SSR/declarative fallback decoder: parse the `events` JSON-string prop.
 *  Bad JSON degrades to an empty list (never throws into the render). */
function parseJsonEvents(raw: string): CalendarEvent[] {
  const s = raw.trim()
  if (!s) return EMPTY
  try {
    const parsed = JSON.parse(s)
    return Array.isArray(parsed) ? (parsed as CalendarEvent[]) : EMPTY
  } catch {
    return EMPTY
  }
}

/** Weekday header labels rotated to honor `weekStartsOn`. */
function weekdayLabels(weekStartsOn: number): { key: string; label: string }[] {
  const out: { key: string; label: string }[] = []
  for (let i = 0; i < 7; i++) {
    const idx = (weekStartsOn + i) % 7
    out.push({ key: `dow-${idx}`, label: DOW_LABELS[idx]! })
  }
  return out
}

export default controller<Scope>({
  scope: ({ events, focusDate, weekStartsOn, slotMinutes, dayStartHour, dayEndHour }) => {
    // The live engine. `prop:calendar` identity (set in mount) supersedes; until
    // then we seed from the SSR `events` string + the focus-date attr so a
    // declarative (no-parent-controller) mount still renders.
    const engine = signal<Calendar | null>(null)

    const fallback = createCalendar({
      events: derived(() => parseJsonEvents(events())),
      initialDate: focusDate() || undefined,
      weekStartsOn: (weekStartsOn() || 0) as 0,
      slotMinutes: slotMinutes() || 30,
      dayStartHour: dayStartHour() ?? 0,
      dayEndHour: dayEndHour() ?? 24,
    })

    /** The active engine — the prop:calendar identity if present, else fallback. */
    const cal = derived<Calendar>(() => engine() ?? fallback)

    // Re-expose the engine view-model as scope deriveds the template reads.
    const view = derived<ViewMode>(() => cal().view())
    const title = derived(() => cal().title())
    const monthCells = derived(() => cal().monthCells())
    // Each cell carries a content-sensitive key so a keyed {#for} re-renders the
    // cell (and its inner event {#for}) when the engine swaps in / events change —
    // a stable `iso`-only key would let the runtime reuse the row DOM and skip
    // the inner loop (the nested-for-over-a-plain-array reuse trap).
    type CellRow = { key: string; iso: string; cell: MonthCell }
    const weekRows = derived<{ key: string; cells: CellRow[] }[]>(() => {
      const cells = cal().monthCells()
      const sel = cal().selectedEventIds()
      const sig = (c: MonthCell): string =>
        `${c.iso}|${c.today ? 't' : ''}|${c.events.map((e) => e.id).join(',')}|${c.overflowCount}|${sel.join(',')}`
      const rows: { key: string; cells: CellRow[] }[] = []
      for (let r = 0; r < 6; r++) {
        const slice = cells.slice(r * 7, r * 7 + 7)
        const cellRows = slice.map((c) => ({ key: sig(c), iso: c.iso, cell: c }))
        rows.push({ key: cellRows.map((cr) => cr.key).join('::'), cells: cellRows })
      }
      return rows
    })
    const weekdays = derived(() => weekdayLabels(weekStartsOn() || 0))
    // Day columns with a content-sensitive key (same reuse-trap guard as the
    // month grid): the laid-out timed events change when the engine swaps.
    const dayColumns = derived(() => {
      const sel = cal().selectedEventIds()
      return cal()
        .dayColumns()
        .map((col) => ({
          key: `${col.iso}|${col.laidOut.map((p) => p.event.id).join(',')}|${sel.join(',')}`,
          col,
        }))
    })
    const timeSlots = derived(() => cal().timeSlots())
    const agenda = derived(() => cal().agenda())
    const isEmpty = derived(() => cal().isEmpty())
    const selectedIds = derived(() => cal().selectedEventIds())
    // view-mode branch flags the template reads (CSS also branches via the
    // reflected `view` host attribute synced in mount).
    const isMonth = derived(() => view() === 'month')
    const isWeek = derived(() => view() === 'week')
    const isDay = derived(() => view() === 'day')

    return {
      engine,
      cal,
      view,
      title,
      monthCells,
      weekRows,
      weekdays,
      dayColumns,
      timeSlots,
      agenda,
      isEmpty,
      selectedIds,
      isMonth,
      isWeek,
      isDay,
    }
  },

  mount(state, host) {
    // prop:calendar identity — a live `Calendar` reference (the engine path),
    // routed into the scope signal so every derived re-resolves onto it.
    Object.defineProperty(host, 'calendar', {
      configurable: true,
      get: () => state.$signals.engine(),
      set: (c: Calendar | null) => {
        state.engine = c
      },
    })

    // Keep the reflected `view` attribute synced to the engine view so the
    // host CSS layout branches follow programmatic setView() too.
    const stopView = effect(() => {
      const v = state.$signals.view()
      if (host.getAttribute('view') !== v) host.setAttribute('view', v)
    })

    const active = (): Calendar => state.$signals.cal()

    // Deduped emit (sui-combobox lastEmitted pattern) over host.emit!.
    let lastView: ViewMode | null = null
    let lastDate: string | null = null
    const stopEmit = effect(() => {
      const cal = active()
      const v = cal.view()
      const d = cal.focusDate()
      if (lastView !== null && v !== lastView) host.emit!('sui-view-change', { view: v })
      if (lastDate !== null && d !== lastDate) host.emit!('sui-date-change', { date: d })
      lastView = v
      lastDate = d
    })

    const onShadowClick = (e: Event) => {
      const t = e.target as HTMLElement | null
      if (!t) return
      const cal = active()

      const viewBtn = t.closest?.('[data-view]') as HTMLElement | null
      if (viewBtn) {
        cal.setView(viewBtn.dataset.view as ViewMode)
        return
      }
      const navBtn = t.closest?.('[data-nav]') as HTMLElement | null
      if (navBtn) {
        const act = navBtn.dataset.nav
        if (act === 'next') cal.next()
        else if (act === 'prev') cal.prev()
        else if (act === 'today') cal.today()
        return
      }
      const chip = t.closest?.('[data-event-id]') as HTMLElement | null
      if (chip) {
        const id = chip.dataset.eventId!
        cal.selectEvent(id, { additive: (e as MouseEvent).metaKey })
        host.emit!('sui-event-click', { id })
        host.emit!('sui-event-select', {
          id,
          selected: cal.selectedEventIds().includes(id),
          selectedIds: cal.selectedEventIds(),
        })
        return
      }
      const dayCell = t.closest?.('[role=gridcell][data-iso]') as HTMLElement | null
      if (dayCell) cal.goToDate(dayCell.dataset.iso!)
    }

    host.shadowRoot?.addEventListener('click', onShadowClick)

    host.expose!({
      next: () => active().next(),
      prev: () => active().prev(),
      today: () => active().today(),
      setView: (v: ViewMode) => active().setView(v),
    })

    return () => {
      stopView()
      stopEmit()
      host.shadowRoot?.removeEventListener('click', onShadowClick)
    }
  },
})
