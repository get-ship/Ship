// <ship-home-greeting> — @ship/home · greeting strip for the hub `/`.
//
// Pure-derived presenter. No SDK, no I/O. The clock ticks once every
// 30s via a single setInterval owned by `mount`; the template re-runs
// the derived reads on each tick (the tick signal is the only mutable
// state). The interval is torn down by `mount`'s returned disposer.
//
// `greeting()` / `dayLabel()` / `clock()` are pure functions of the
// `now()` signal; the template binds them as plain expressions (the
// cutover-clarity rules: no I/O, no const/let in the template).

import { controller, derived, signal } from '@ship/sui-runtime'
import type { MutableState, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-home-greeting.sui'

const DOW = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as const
const MONTH = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
  'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
] as const

function partOfDay(hour: number): string {
  if (hour < 12) return 'Good morning'
  if (hour < 18) return 'Good afternoon'
  return 'Good evening'
}

function formatDay(d: Date): string {
  const day = DOW[d.getDay()] ?? 'Sun'
  const mon = MONTH[d.getMonth()] ?? 'Jan'
  return `${day} · ${mon} ${d.getDate()}`
}

function formatClock(d: Date): string {
  const h = String(d.getHours()).padStart(2, '0')
  const m = String(d.getMinutes()).padStart(2, '0')
  return `${h}:${m}`
}

export default controller({
  scope: ({ workspace }: ViewScope<Scope>) => {
    const now = signal<number>(Date.now())

    const greeting = derived<string>(() => partOfDay(new Date(now()).getHours()))
    const dayLabel = derived<string>(() => formatDay(new Date(now())))
    const clock = derived<string>(() => formatClock(new Date(now())))
    const sub = derived<string>(() => {
      const ws = workspace().trim()
      const day = formatDay(new Date(now()))
      return ws
        ? `Your space, ready when you are — ${ws}, ${day}.`
        : 'Your space, ready when you are.'
    })

    return { greeting, dayLabel, clock, sub, now }
  },

  mount(state: MutableState<Scope>) {
    const nowSig = state.$signals.now
    const stop = setInterval(() => nowSig.set(Date.now()), 30_000)
    return () => clearInterval(stop)
  },
})
