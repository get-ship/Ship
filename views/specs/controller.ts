import { controller, signal, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ selected, route }) => {
    const q = signal((route as any)().query?.q || '')
    const selectedId = derived(() => (route as any)().query?.id || '')
    const confirmDelete = signal(false)
    const filterInputEl = signal<HTMLElement | null>(null)
    return { q, selectedId, confirmDelete, filterInputEl }
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
