import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import { route as routeFactory } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

type SchemaField = {
  name: string
  field_type: string
  required?: boolean
  values?: string[]
}

export default controller<Scope>({
  scope: ({ ship }) => {
    const route = routeFactory()
    const activeId = derived(() => route.query.collection() || '')
    const colDetail = live(() => ship.collections.detail({ id: activeId() }))
    const schemaFieldsView = derived(() => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const d = unwrap(colDetail as any) as { schema?: { fields?: SchemaField[] } } | undefined
      const fields = d?.schema?.fields ?? []
      return fields.map(f => ({ ...f, values_pretty: (f.values ?? []).join(' | ') }))
    })
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
      activeId, colDetail, schemaFieldsView, tab,
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
    removeSchemaField: async (s, fieldName: string) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const detail = unwrap(s.colDetail as any) as { schema?: { fields?: SchemaField[] } } | undefined
      const fields = detail?.schema?.fields ?? []
      const fs = fields
        .filter(x => x.name !== fieldName)
        .map(x => ({
          name: x.name,
          type: x.field_type,
          required: x.required,
          values: x.values ?? [],
        }))
      const colId = s.activeId as string
      await fetch('/api/v1/collections/' + colId + '/schema', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ id: colId, fields: fs }),
      })
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
