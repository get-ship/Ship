// Specs view controller — wires rail (summaries) and detail panes to
// the daemon's `specs.*` registry ops over `/_sui/data?expr=…`.
//
// The runtime-bundled SDK client (`/_sui/sdk-client.js`) doesn't carry
// a `specs` namespace, so `ship.specs.summaries(...)` would throw —
// and the hydrate-while-loading `firstRun` guard then traps the
// SSR-rendered "Loading…" branch because the signal transitions to
// error before `hydrateWhileLoading` subscribes. Skipping the SDK and
// opening the SSE channel directly keeps the signal in `loading`
// until the first daemon patch lands.

import { controller, signal, derived, live, ref, unwrap } from '@ship/sui-runtime'
import type { LiveSignal, LiveSource } from '@ship/sui-runtime'
import { route as routeFactory } from '@ship/sui-runtime/route'
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
  scope: () => {
    const route = routeFactory()
    const q = signal(route.query.q() || '')
    const selectedId = derived(() => route.query.id() || '')
    const confirmDelete = signal(false)
    const filterInputEl = ref<HTMLElement>(null)
    const rail = live(() =>
      sseSource<unknown>(
        `ship.specs.summaries(${JSON.stringify({ q: q(), selected: selectedId() })})`,
      ),
    )
    const selected = live(() =>
      sseSource<unknown>(
        `ship.specs.detail(${JSON.stringify({ id: selectedId() })})`,
      ),
    )
    return { q, selectedId, confirmDelete, filterInputEl, rail, selected }
  },
  on: {
    newSpec: async () => {
      await fetch('/api/v1/specs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ title: 'New spec', body: '## Goal\n\n## Acceptance\n- [ ] ' }),
      })
    },
    selectSpec: ({ navigate }, id: string) => {
      navigate('/specs?id=' + id)
    },
    duplicateSpec: async ({ state }) => {
      const s = unwrap(state.selected as LiveSignal<{ title?: string; body?: string }>)
      if (!s) return
      await fetch('/api/v1/specs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ title: (s.title ?? '') + ' (copy)', body: s.body || '' }),
      })
    },
    launchSpec: async ({ state }) => {
      const id = state.selectedId as string
      if (!id) return
      await fetch(`/api/v1/specs/${id}/launch`, { method: 'POST', credentials: 'include' })
    },
    deleteSpec: async ({ state, navigate }) => {
      const id = state.selectedId as string
      if (!id) return
      await fetch(`/api/v1/specs/${id}`, { method: 'DELETE', credentials: 'include' })
      state.confirmDelete = false
      navigate('/specs')
    },
    focusFilter: ({ state }) => {
      const el = state.filterInputEl as HTMLElement | null
      el?.focus()
    },
  },
})
