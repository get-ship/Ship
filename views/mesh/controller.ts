// Mesh view controller — owns the `snap` live signal end-to-end.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so the compiler-emitted `<data live>` path (which closes over
// `ctx.ship`) sticks on `loading`. Wire the live source directly to
// the daemon's `/_sui/data?expr=…` SSE channel instead — same data
// path the sdk-client would take, just without the importmap dance
// that `@ship/sdk-client` requires.

import { controller, derived, live, unwrap } from '@ship/sui-runtime'
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
  scope: () => {
    const snap = live(() => sseSource<{ actors?: Actor[] }>('ship.mesh.snapshot()'))
    const actorsFormatted = derived(() => {
      const v = unwrap(snap)
      const list = v?.actors ?? []
      return list.map(a => ({ ...a, last_event_at_pretty: formatTs(a.last_event_at) }))
    })
    return { snap, actorsFormatted }
  },
})
