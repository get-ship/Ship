import { controller, signal, derived, live, ref, unwrap } from '@ship/sui-runtime'
import { route as routeFactory } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ ship }) => {
    const route = routeFactory()
    const q = signal(route.query.q() || '')
    const selectedId = derived(() => route.query.id() || '')
    const confirmDelete = signal(false)
    const filterInputEl = ref<HTMLElement>(null)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const rail = live(() => s.specs.summaries({ q: q(), selected: selectedId() }))
    const selected = live(() => s.specs.detail({ id: selectedId() }))
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
      const s = unwrap(state.selected as any)
      if (!s) return
      await fetch('/api/v1/specs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ title: s.title + ' (copy)', body: s.body || '' }),
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
