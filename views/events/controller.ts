// Events view controller — owns the events live signal end-to-end.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so any `ship.<x>.<y>()` call sticks on `loading`. Wire the live
// source directly to `/_sui/data?expr=…` instead.

import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './page.sui'

interface EventsArgs {
  q?: string
  event_type?: string
  actor?: string
  entity?: string
}

function eventsExpr(args: EventsArgs): string {
  const filtered: Record<string, string> = {}
  for (const [k, v] of Object.entries(args)) {
    if (typeof v === 'string' && v.length > 0) filtered[k] = v
  }
  const lit = Object.keys(filtered).length === 0 ? '' : JSON.stringify(filtered)
  return `ship.events.list(${lit})`
}

type EventsValue = {
  groups: Array<{ rows: unknown[] }>
}

export default controller<Scope>({
  scope: () => {
    const q = signal('')
    const eventType = signal('')
    const actor = signal('')
    const entity = signal('')
    const eventsData = live(() => liveExpr<EventsValue>(eventsExpr({
      q: q(),
      event_type: eventType(),
      actor: actor(),
      entity: entity(),
    })))
    const totalCount = derived(() => {
      const d = unwrap(eventsData)
      if (!d) return 0
      return d.groups.reduce((acc, g) => acc + g.rows.length, 0)
    })
    return { q, eventType, actor, entity, eventsData, totalCount }
  },
})
