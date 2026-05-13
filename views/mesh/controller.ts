import { controller, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

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

export default controller<Scope>({
  scope: ({ snap }) => {
    const actorsFormatted = derived(() => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const v = unwrap(snap as any) as { actors?: Actor[] } | undefined
      const list = v?.actors ?? []
      return list.map(a => ({ ...a, last_event_at_pretty: formatTs(a.last_event_at) }))
    })
    return { actorsFormatted }
  },
})
