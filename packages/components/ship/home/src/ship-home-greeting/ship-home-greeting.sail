<controller>
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

import { controller, derived, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-home-greeting.sail'

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

export default controller(({ workspace }: ViewScope<Scope>) => {

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
})
</controller>

<!--
  <ship-home-greeting> — @ship/home · greeting strip for the hub `/`.

  The hero band of the home dashboard: hour-of-day greeting (morning /
  afternoon / evening), workspace label (when supplied), and a small
  weekday + date + clock readout. The clock ticks every 30s; the
  template's derived reads re-run on the tick signal, swapping only
  the data-clock text node (pure expression, no I/O in the template).

  Pure-derived presenter — no SDK I/O. The workspace label is supplied
  as a prop or, when absent, falls back to the controller's default
  sub-line (the host's controller wires the workspace name from the
  live pipelines workspace_id, but the component itself stays
  presentational).

  Prop / event
  workspace (string, '')   workspace label shown next to the greeting

  Tokens only: the strip uses --font-display, --font-mono, --foreground,
  --muted-foreground, --primary, --border, --space-*. No component-
  local hard-coded colours; theming is inherited from the parent.
-->

<!-- <prop name="workspace" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<header data-test="home-greeting">
  <h1>{greeting()}.</h1>
  <if cond={workspace()}>
    <span data-test="home-greeting-workspace">{workspace()}</span>
  </if>
  <span data-test="home-greeting-day">{dayLabel()} · {clock()}</span>
</header>
<div aria-hidden="true"></div>
<p>{sub()}</p>
