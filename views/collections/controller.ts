import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ route }) => {
    const activeId = derived(() => (route as any)().query?.collection || '')
    const tab = signal('rows')
    const newRowOpen = signal(false)
    const editOpen = signal(false)
    const editId = signal('')
    const newColOpen = signal(false)
    const rowDraft = signal<Record<string, unknown>>({})
    const editDraft = signal<Record<string, unknown>>({})
    const newColDraftId = signal('')
    const newColDraftName = signal('')
    const drawerError = signal('')
    return {
      activeId, tab,
      newRowOpen, editOpen, editId, newColOpen,
      rowDraft, editDraft, newColDraftId, newColDraftName,
      drawerError,
    }
  },
  on: {
    setDraftField: (s, which: 'row' | 'edit', name: string, value: unknown) => {
      const key = which === 'row' ? 'rowDraft' : 'editDraft'
      s[key] = { ...(s[key] as object), [name]: value }
    },
    openEditRow: (s, item: { id: string; data?: Record<string, unknown> }) => {
      s.editId = item.id
      s.editDraft = { ...(item.data ?? {}) }
      s.drawerError = ''
      s.editOpen = true
    },
    saveNewRow: async (s) => {
      const id = s.activeId as string
      const res = await fetch(`/api/v1/collections/${id}/items`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ data: s.rowDraft }),
      })
      if (!res.ok) { s.drawerError = await res.text().catch(() => 'Request failed'); return }
      s.newRowOpen = false
      s.rowDraft = {}
      s.drawerError = ''
    },
    saveEdit: async (s) => {
      const colId = s.activeId as string
      const rowId = s.editId as string
      const res = await fetch(`/api/v1/collections/${colId}/items/${rowId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ data: s.editDraft }),
      })
      if (!res.ok) { s.drawerError = await res.text().catch(() => 'Request failed'); return }
      s.editOpen = false
      s.editDraft = {}
      s.drawerError = ''
    },
    deleteRow: async (s) => {
      const colId = s.activeId as string
      const rowId = s.editId as string
      await fetch(`/api/v1/collections/${colId}/items/${rowId}`, {
        method: 'DELETE',
        credentials: 'include',
      })
      s.editOpen = false
      s.editDraft = {}
      s.drawerError = ''
    },
    createCollection: async (s) => {
      const id = (s.newColDraftId as string).trim()
      if (!id) { s.drawerError = 'Collection ID is required'; return }
      const res = await fetch('/api/v1/collections', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ id, name: (s.newColDraftName as string).trim() || id, fields: {} }),
      })
      if (!res.ok) { s.drawerError = await res.text().catch(() => 'Request failed'); return }
      s.newColOpen = false
      s.newColDraftId = ''
      s.newColDraftName = ''
      s.drawerError = ''
      ;(s as any).navigate('/collections?collection=' + id)
    },
  },
})
