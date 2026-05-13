// Settings view controller — owns the `settings` live signal directly.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so the compiler-emitted `<data live>` path (which closes over
// `ctx.ship`) sticks on `loading`. Wire the live source directly to
// `/_sui/data?expr=ship.settings.state()` instead.

import { controller, signal, live } from '@ship/sui-runtime'
import type { LiveSignal, LiveSource, LiveState } from '@ship/sui-runtime'
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

export default controller<Scope>({
  scope: () => ({
    advancedOpen: signal(false),
    $mutation: signal<LiveState<unknown>>({ kind: 'ok', value: undefined }),
    settings: live(() => sseSource<unknown>('ship.settings.state()')),
  }),
})
