// sui-calendar engine — the THIN ASSEMBLER (Backbone #1, Slice 1b).
//
// Wires the reactive slices and returns the `Calendar` facade. Imports
// `{ signal, derived, batch, isSignal }` from the runtime ONLY — no SDK import,
// no backend calls, no network I/O, no data subscriptions (the purity grep gate
// asserts the boundary). Pure reactive state, emits no DOM. Heavy logic lives
// in the pure siblings (events.ts / view-model.ts / layout.ts / date-math.ts);
// this module is the signal graph + the mutators.
//
// One-direction derived chain (build-spec §"Signal graph"): events → visible →
// {monthCells, dayColumns, timeSlots, allDayBand}. Selection / focus / view are
// orthogonal slices so a write to one recomputes only the relevant suffix.

import { batch, derived, isSignal, signal } from '@ship/sui-runtime'
import {
  addDays,
  addMonths,
  startOfWeek,
  todayIso,
  weekDays,
  ymd,
  type ISODate,
} from './date-math'
import { eventsInRange } from './events'
import {
  buildAllDayBand,
  buildDayColumns,
  buildMonthCells,
  buildTimeSlots,
  titleFor,
} from './view-model'
import type { Calendar, CalendarOptions } from './contract'
import type { AgendaRow, CalendarEvent, DateRange, ViewMode } from './types'

const DEFAULT_CAP = 3

export function createCalendar(options: CalendarOptions): Calendar {
  const weekStartsOn = options.weekStartsOn ?? 0
  const slotMinutes = options.slotMinutes ?? 30
  const dayStartHour = options.dayStartHour ?? 0
  const dayEndHour = options.dayEndHour ?? 24
  const cap = options.monthCellCap ?? DEFAULT_CAP

  // ── State slices ────────────────────────────────────────────────────────────
  // Lift the raw array OR bind to the live signal identity.
  const eventsSig = isSignal(options.events)
    ? (options.events as ReturnType<typeof signal<CalendarEvent[]>>)
    : signal<CalendarEvent[]>(
        Array.isArray(options.events) ? options.events : (options.events as () => CalendarEvent[])(),
      )
  // A Derived passed as `events` is read through a wrapper derived below.
  const eventsRead =
    typeof options.events === 'function' && !isSignal(options.events)
      ? (options.events as () => CalendarEvent[])
      : eventsSig

  const view = signal<ViewMode>(options.initialView ?? 'month')
  const focusDate = signal<ISODate>(options.initialDate ?? todayIso())
  const range = signal<DateRange | null>(null)
  const selectedEventIds = signal<string[]>([])
  const today = signal<ISODate>(todayIso())

  // ── Derived chain ─────────────────────────────────────────────────────────────
  const allEvents = derived(() => eventsRead())

  // The visible day window for the current view.
  const windowDays = derived<ISODate[]>(() => {
    const f = focusDate()
    const v = view()
    if (v === 'day') return [f]
    if (v === 'week') return weekDays(f, weekStartsOn)
    // month: the 42-cell grid span.
    const [y, m] = ymd(f)
    const start = startOfWeek(`${y}-${pad2(m)}-01`, weekStartsOn)
    const out: ISODate[] = []
    for (let i = 0; i < 42; i++) out.push(addDays(start, i))
    return out
  })

  const visibleEvents = derived(() => {
    const days = windowDays()
    if (days.length === 0) return []
    return eventsInRange(allEvents(), days[0]!, days[days.length - 1]!)
  })

  const monthCells = derived(() => {
    const [y, m] = ymd(focusDate())
    return buildMonthCells({
      year: y,
      month: m - 1,
      weekStartsOn,
      today: today(),
      events: visibleEvents(),
      selectedIds: selectedEventIds(),
      focusedIso: null,
      cap,
    })
  })

  const timeSlots = derived(() => buildTimeSlots(dayStartHour, dayEndHour, slotMinutes))

  const dayColumns = derived(() =>
    buildDayColumns({
      days: windowDays(),
      today: today(),
      events: visibleEvents(),
      windowStart: dayStartHour * 60,
      windowEnd: dayEndHour * 60,
    }),
  )

  const allDayBand = derived(() =>
    buildAllDayBand({ days: windowDays(), events: visibleEvents() }),
  )

  const agenda = derived<AgendaRow[]>(() => {
    // Flat chronological day-header/event rows over the visible window.
    const out: AgendaRow[] = []
    for (const iso of windowDays()) {
      const dayEvents = visibleEvents().filter((e) => e.start.slice(0, 10) === iso)
      if (dayEvents.length === 0) continue
      out.push({ key: `h-${iso}`, iso, kind: 'day-header' })
      for (const e of dayEvents) out.push({ key: `e-${e.id}`, iso, kind: 'event', event: e })
    }
    return out
  })

  const title = derived(() => titleFor(view(), focusDate(), weekStartsOn))
  const isEmpty = derived(() => visibleEvents().length === 0)

  // ── Mutators (Cardinal 3 — call only from on:* / mount) ───────────────────────
  const setView = (v: ViewMode) => {
    batch(() => view.set(v))
    options.onViewChange?.(v)
  }
  const emitDate = (d: ISODate) => options.onDateChange?.(d)
  const goToDate = (d: ISODate) => {
    batch(() => focusDate.set(d))
    emitDate(d)
  }
  const stepBy = (dir: 1 | -1) => {
    const v = view()
    const f = focusDate()
    const next = v === 'month' ? addMonths(f, dir) : v === 'week' ? addDays(f, dir * 7) : addDays(f, dir)
    goToDate(next)
  }

  const selectEvent = (id: string, opts?: { additive?: boolean }) => {
    batch(() => {
      if (opts?.additive) {
        const cur = selectedEventIds()
        selectedEventIds.set(cur.includes(id) ? cur.filter((x) => x !== id) : [...cur, id])
      } else {
        selectedEventIds.set([id])
      }
    })
    options.onSelectionChange?.(selectedEventIds())
  }
  const clearSelection = () => {
    batch(() => selectedEventIds.set([]))
    options.onSelectionChange?.([])
  }

  return {
    view: view as Calendar['view'],
    focusDate: focusDate as Calendar['focusDate'],
    title,
    monthCells,
    dayColumns,
    timeSlots,
    allDayBand,
    agenda: agenda as Calendar['agenda'],
    range: range as Calendar['range'],
    selectedEventIds: selectedEventIds as Calendar['selectedEventIds'],
    isEmpty,
    state: { events: eventsSig, view, focusDate, range, selectedEventIds },
    setView,
    goToDate,
    today: () => goToDate(todayIso()),
    next: () => stepBy(1),
    prev: () => stepBy(-1),
    selectEvent,
    clearSelection,
  }
}

const pad2 = (n: number) => (n < 10 ? `0${n}` : String(n))
