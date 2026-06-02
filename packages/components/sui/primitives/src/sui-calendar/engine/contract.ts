// sui-calendar engine — the createCalendar() public contract (Backbone #1).
//
// `CalendarOptions` (input) + the `Calendar` facade the assembler returns.
// PURE TypeScript — type-only. Imports the Signal/Derived handles from
// `@ship/sui-runtime` and the shared shapes from `./types`. `types.ts` and
// `contract.ts` are the one source of truth for the surface (no parallel defn).

import type { Derived, Signal } from '@ship/sui-runtime'
import type {
  AgendaRow,
  AllDayLane,
  CalendarEvent,
  DateRange,
  DayColumn,
  ISODate,
  MonthCell,
  ReactiveCalendarState,
  TimeSlot,
  ViewMode,
  WeekStart,
} from './types'

export interface CalendarOptions {
  /** Raw array OR a live Signal/Derived identity (growing it adds events). */
  events: CalendarEvent[] | Signal<CalendarEvent[]> | Derived<CalendarEvent[]>
  /** default `e => e.id`; selection/{#for}-key/drag identity all key off it. */
  getEventId?: (e: CalendarEvent) => string
  initialView?: ViewMode
  /** focus anchor; defaults to today. */
  initialDate?: ISODate
  weekStartsOn?: WeekStart
  /** day/week grid granularity in minutes; default 30. */
  slotMinutes?: number
  dayStartHour?: number
  dayEndHour?: number
  /** max events shown per month cell before an "+N more" overflow. */
  monthCellCap?: number
  // callbacks (engine → consumer; consumer writes its own arg signals).
  onViewChange?: (v: ViewMode) => void
  onDateChange?: (d: ISODate) => void
  onSelectionChange?: (ids: string[]) => void
}

export interface Calendar {
  // View-model render surfaces (Derived — read in template/derived).
  view: Derived<ViewMode>
  focusDate: Derived<ISODate>
  title: Derived<string>
  monthCells: Derived<MonthCell[]>
  dayColumns: Derived<DayColumn[]>
  timeSlots: Derived<TimeSlot[]>
  allDayBand: Derived<AllDayLane[]>
  agenda: Derived<AgendaRow[]>
  range: Derived<DateRange | null>
  selectedEventIds: Derived<string[]>
  isEmpty: Derived<boolean>
  // Centralized reactive state (each slice a Signal).
  state: ReactiveCalendarState
  // Mutators — call ONLY from on:* / mount (Cardinal 3).
  setView(v: ViewMode): void
  goToDate(d: ISODate): void
  today(): void
  next(): void
  prev(): void
  selectEvent(id: string, opts?: { additive?: boolean }): void
  clearSelection(): void
}

export type {
  AgendaRow,
  AllDayLane,
  CalendarEvent,
  DateRange,
  DayColumn,
  ISODate,
  MonthCell,
  ReactiveCalendarState,
  TimeSlot,
  ViewMode,
  WeekStart,
} from './types'
