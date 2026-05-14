// Events view controller — owns the events live signal end-to-end.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so any `ship.<x>.<y>()` call sticks on `loading`. Wire the live
// source directly to `/_sui/data?expr=…` instead.

import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import type { LiveSignal, LiveSource } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

function sseSource<T>(expr: string): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      const es = new EventSource(`/_sui/data?expr=${encodeURIComponent(expr)}`)
      const onPatch = (ev: MessageEvent): void => {
        try {
          const patch = JSON.parse(ev.data as string) as { op: string; value: T }
          if (patch.op === 'set') sig.set({ kind: 'ok', value: patch.value })
        } catch (err) {
          sig.set({ kind: 'error', error: err instanceof Error ? err : new Error(String(err)) })
        }
      }
      es.addEventListener('patch', onPatch as EventListener)
      return () => { es.close() }
    },
  }
}

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
    const eventsData = live(() => sseSource<EventsValue>(eventsExpr({
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
