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

import { controller, signal, derived, effect, live, ref, unwrap } from '@ship/sui-runtime'
import type { LiveSignal } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import { route as routeFactory } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

type AgentSummary = { id: string }
type SpecDetail = {
  id: string; title: string; body: string; status: string
  pipeline?: string | null; agent?: string | null
  launch_label?: string; launch_hint?: string
}

type Navigate = (path: string) => void

// State-proxy reads unwrap signals to their values, so `state.selected`
// is the LiveSignalValue, not the signal. `unwrap(sig)` needs the
// callable LiveSignal — fetch it via the proxy's `$signals` escape hatch.
type WithSignals = { $signals: { selected: LiveSignal<SpecDetail> } }
const liveSelected = (state: unknown): LiveSignal<SpecDetail> =>
  (state as WithSignals).$signals.selected

const STATUS_CYCLE = ['draft', 'ready', 'launched', 'archived'] as const

export default controller<Scope>({
  scope: () => {
    const route = routeFactory()
    const q = signal(route.query.q() || '')
    const selectedId = derived(() => route.query.id() || '')
    const confirmDelete = signal(false)
    const filterInputEl = ref<HTMLElement>(null)
    const editing = signal(false)
    const saving = signal(false)
    const draftBody = signal('')
    const draftTitle = signal('')
    const agentsList = signal<AgentSummary[]>([])

    const rail = live(() =>
      liveExpr<unknown>(
        `ship.specs.summaries(${JSON.stringify({ q: q(), selected: selectedId() })})`,
      ),
    )
    const selected = live(() =>
      liveExpr<unknown>(
        `ship.specs.detail(${JSON.stringify({ id: selectedId() })})`,
      ),
    )

    // Best-effort agent profile load. List is small and quasi-static, so
    // a one-shot fetch is enough — no need for a live SSE channel.
    void fetch('/api/v1/agents', { credentials: 'include' })
      .then((r) => (r.ok ? r.json() : { items: [] }))
      .then((b: { items?: AgentSummary[] }) => {
        const items = Array.isArray(b?.items) ? b.items : []
        agentsList.set(items.filter((a) => typeof a?.id === 'string'))
      })
      .catch(() => { /* offline → empty picker */ })

    // Drop edit state on selection change so a stale draft never bleeds
    // across specs.
    effect(() => {
      selectedId()
      editing.set(false)
    })

    return {
      q, selectedId, confirmDelete, filterInputEl, rail, selected,
      editing, saving, draftBody, draftTitle, agentsList,
    }
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
    selectSpec: (state, id: string) => {
      (state.navigate as Navigate)('/specs?id=' + id)
    },
    duplicateSpec: async (state) => {
      const s = unwrap(liveSelected(state))
      if (!s) return
      await fetch('/api/v1/specs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ title: (s.title ?? '') + ' (copy)', body: s.body || '' }),
      })
    },
    launchSpec: async (state) => {
      const id = state.selectedId as string
      if (!id) return
      await fetch(`/api/v1/specs/${id}/launch`, { method: 'POST', credentials: 'include' })
    },
    deleteSpec: async (state) => {
      const id = state.selectedId as string
      if (!id) return
      await fetch(`/api/v1/specs/${id}`, { method: 'DELETE', credentials: 'include' })
      state.confirmDelete = false
      ;(state.navigate as Navigate)('/specs')
    },
    focusFilter: (state) => {
      const el = state.filterInputEl as HTMLElement | null
      el?.focus()
    },
    startEdit: (state) => {
      const sel = unwrap(liveSelected(state))
      if (!sel) return
      state.draftTitle = sel.title ?? ''
      state.draftBody = sel.body ?? ''
      state.editing = true
    },
    cancelEdit: (state) => {
      state.editing = false
    },
    onTitleInput: (state, evt: Event) => {
      const el = evt.target as HTMLElement & { value?: string }
      state.draftTitle = typeof el?.value === 'string' ? el.value : ''
    },
    onBodyInput: (state, evt: Event) => {
      const detail = (evt as CustomEvent<{ value?: string }>).detail
      if (detail && typeof detail.value === 'string') {
        state.draftBody = detail.value
        return
      }
      const el = evt.target as HTMLElement & { value?: string }
      if (typeof el?.value === 'string') state.draftBody = el.value
    },
    saveEdit: async (state) => {
      const id = state.selectedId as string
      if (!id) return
      state.saving = true
      try {
        await fetch(`/api/v1/specs/${id}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({ title: state.draftTitle, body: state.draftBody }),
        })
        state.editing = false
      } finally {
        state.saving = false
      }
    },
    setAgent: async (state, evt: Event) => {
      const id = state.selectedId as string
      if (!id) return
      const detail = (evt as CustomEvent<{ value?: string }>).detail
      const el = evt.target as HTMLElement & { value?: string }
      const next = (detail && typeof detail.value === 'string') ? detail.value
        : (typeof el?.value === 'string' ? el.value : '')
      await fetch(`/api/v1/specs/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ agent: next === '' ? null : next }),
      })
    },
    cycleStatus: async (state) => {
      const id = state.selectedId as string
      if (!id) return
      const sel = unwrap(liveSelected(state))
      if (!sel) return
      const current = sel.status ?? 'draft'
      const idx = STATUS_CYCLE.indexOf(current as typeof STATUS_CYCLE[number])
      const next = STATUS_CYCLE[(idx + 1) % STATUS_CYCLE.length]
      await fetch(`/api/v1/specs/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ status: next }),
      })
    },
  },
})
