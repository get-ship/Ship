import { controller, signal, effect, live, chrome, unwrapOr } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface DashStats { active: number; history: number }
interface DashState {
  stats: DashStats
  agents: unknown[]
  events: unknown[]
  total_events: number
}

interface GitOverview {
  branch: string
  head_short: string
  clean: boolean
  total_changed: number
}

interface EventRow {
  id?: string
  timestamp: string
  event_type: string
  actor?: string
}
interface EventsEnvelope { groups?: Array<{ rows?: EventRow[] }>; rows?: EventRow[] }

const EMPTY_DASH: DashState = { stats: { active: 0, history: 0 }, agents: [], events: [], total_events: 0 }
const EMPTY_GIT: GitOverview = { branch: '', head_short: '', clean: true, total_changed: 0 }

const fmtTime = (ms: number | null): string => {
  if (ms == null) return '—'
  const d = Date.now() - ms
  if (d < 1000) return 'just now'
  if (d < 60_000) return `${Math.floor(d / 1000)}s ago`
  if (d < 3_600_000) return `${Math.floor(d / 60_000)}m ago`
  return `${Math.floor(d / 3_600_000)}h ago`
}

const flattenEvents = (env: EventsEnvelope | undefined): EventRow[] => {
  if (!env) return []
  if (env.rows) return env.rows
  if (env.groups) return env.groups.flatMap((g) => g.rows ?? [])
  return []
}

export default controller<Scope>({
  scope: ({ ship, dash, git }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const evs = live(() => (ship as any).events.list({ limit: 30 }))
    const dashPatches  = signal(0)
    const gitPatches   = signal(0)
    const evsPatches   = signal(0)
    const dashLast     = signal<number | null>(null)
    const gitLast      = signal<number | null>(null)
    const evsLast      = signal<number | null>(null)
    const dashFresh    = signal(false)
    const gitFresh     = signal(false)
    const evsFresh     = signal(false)
    const now          = signal(Date.now())

    const bumpFresh = (sig: { set: (v: boolean) => void }): void => {
      sig.set(true)
      setTimeout(() => sig.set(false), 700)
    }

    effect(() => {
      const s = dash()
      if (s.kind === 'ok') {
        dashPatches.set(dashPatches() + 1)
        dashLast.set(Date.now())
        bumpFresh(dashFresh)
      }
    })
    effect(() => {
      const s = git()
      if (s.kind === 'ok') {
        gitPatches.set(gitPatches() + 1)
        gitLast.set(Date.now())
        bumpFresh(gitFresh)
      }
    })
    effect(() => {
      const s = evs()
      if (s.kind === 'ok') {
        evsPatches.set(evsPatches() + 1)
        evsLast.set(Date.now())
        bumpFresh(evsFresh)
      }
    })

    const tick = setInterval(() => now.set(Date.now()), 1000)

    return {
      evs,
      dashPatches, gitPatches, evsPatches,
      dashLast, gitLast, evsLast,
      dashFresh, gitFresh, evsFresh,
      now,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },
      resetCounters: (): void => {
        dashPatches.set(0); gitPatches.set(0); evsPatches.set(0)
        dashLast.set(null); gitLast.set(null); evsLast.set(null)
      },

      __tick: tick,
    }
  },

  derived: {
    totalPatches: ({ dashPatches, gitPatches, evsPatches }) =>
      dashPatches() + gitPatches() + evsPatches(),

    dashKind:    ({ dash }) => dash().kind,
    gitKind:     ({ git })  => git().kind,
    evsKind:     ({ evs })  => evs().kind,

    dashLastLabel: ({ dashLast, now }) => { now(); return fmtTime(dashLast()) },
    gitLastLabel:  ({ gitLast, now })  => { now(); return fmtTime(gitLast()) },
    evsLastLabel:  ({ evsLast, now })  => { now(); return fmtTime(evsLast()) },

    dashJobs: ({ dash }) => {
      const v = unwrapOr(dash as never, EMPTY_DASH as never) as DashState
      return v.stats.active + v.stats.history
    },
    dashAgents: ({ dash }) => (unwrapOr(dash as never, EMPTY_DASH as never) as DashState).agents.length,
    dashEvents: ({ dash }) => (unwrapOr(dash as never, EMPTY_DASH as never) as DashState).total_events,

    gitBranch: ({ git }) => (unwrapOr(git as never, EMPTY_GIT as never) as GitOverview).branch || '—',
    gitHead:   ({ git }) => (unwrapOr(git as never, EMPTY_GIT as never) as GitOverview).head_short || '—',
    gitStatus: ({ git }) => {
      const v = unwrapOr(git as never, EMPTY_GIT as never) as GitOverview
      return v.clean ? 'clean' : `${v.total_changed} changed`
    },

    recentEvents: ({ evs }) => {
      const env = unwrapOr(evs as never, {} as never) as EventsEnvelope
      return flattenEvents(env).slice(0, 12).map((r, i) => ({
        ...r,
        key: r.id ?? `${r.timestamp}-${i}`,
      }))
    },
  },
})
