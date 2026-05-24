// <sui-activity> — owner-level, cross-space runtime monitor. First-party CHROME
// (apps/shell): the shell renders it at /activity. Live `ship.mesh.snapshot()`
// over @ship/sdk-client. (Relocated from views/activity; was the mesh draft.)
import { controller, derived, live, signal, unwrap } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './sui-activity.sui'

function formatTs(raw: string | null | undefined): string {
  if (!raw) return '—'
  return raw.slice(0, 19).replace('T', ' ')
}

type Actor = {
  actor_id: string
  kind?: string
  agent_id?: string | null
  provider?: string | null
  transport?: string | null
  workspace_id?: string | null
  pid?: number | null
  last_event_at?: string | null
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const snap = live(() => liveExpr<{ actors?: Actor[] }>('ship.mesh.snapshot()'))
    const actorsFormatted = derived(() => {
      const v = unwrap(snap)
      const list = v?.actors ?? []
      return list.map((a) => ({ ...a, last_event_at_pretty: formatTs(a.last_event_at) }))
    })
    const attachSlug = signal<string>('')
    const attachOpen = signal(false)
    return { snap, actorsFormatted, attachSlug, attachOpen }
  },
  on: {
    openAttach: (state, slug: string) => {
      state.attachSlug = slug
      state.attachOpen = true
    },
    closeAttach: (state) => {
      state.attachOpen = false
      state.attachSlug = ''
    },
  },
})
