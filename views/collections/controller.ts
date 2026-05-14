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
    // SDK mutators captured from the controller ctx so `on` handlers can
    // reach the daemon through the typed surface instead of raw fetch.
    const createItem = (a: { collection: string; data: Record<string, unknown> }) =>
      ship.collections.create_item(a)
    const updateItem = (a: {
      collection: string
      item_id: string
      data: Record<string, unknown>
    }) => ship.collections.update_item(a)
    const deleteItem = (a: { collection: string; item_id: string }) =>
      ship.collections.delete_item(a)
    const updateSchema = (a: {
      id: string
      fields: Array<{
        name: string
        type: string
        required?: boolean
        values?: string[]
      }>
    }) => ship.collections.update_schema(a)
    const createCol = (a: { id: string; name?: string; fields?: Record<string, unknown> }) =>
      ship.collections.create(a)
    return {
      activeId, colDetail, schemaFieldsView, tab,
      newRowOpen, editOpen, editId, newColOpen,
      rowDraft, editDraft, newColDraftId, newColDraftName,
      drawerError,
      createItem, updateItem, deleteItem, updateSchema, createCol,
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
      try {
        await (s.createItem as (a: { collection: string; data: Record<string, unknown> }) => Promise<unknown>)(
          { collection: id, data: s.rowDraft as Record<string, unknown> },
        )
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : 'Request failed'
        return
      }
      s.newRowOpen = false
      s.rowDraft = {}
      s.drawerError = ''
    },
    saveEdit: async (s) => {
      const colId = s.activeId as string
      const rowId = s.editId as string
      try {
        await (s.updateItem as (a: {
          collection: string
          item_id: string
          data: Record<string, unknown>
        }) => Promise<unknown>)({ collection: colId, item_id: rowId, data: s.editDraft as Record<string, unknown> })
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : 'Request failed'
        return
      }
      s.editOpen = false
      s.editDraft = {}
      s.drawerError = ''
    },
    deleteRow: async (s) => {
      const colId = s.activeId as string
      const rowId = s.editId as string
      try {
        await (s.deleteItem as (a: { collection: string; item_id: string }) => Promise<void>)({
          collection: colId, item_id: rowId,
        })
      } catch { /* swallow — UI reflects via SSE refresh */ }
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
      try {
        await (s.updateSchema as (a: {
          id: string
          fields: Array<{ name: string; type: string; required?: boolean; values?: string[] }>
        }) => Promise<unknown>)({ id: colId, fields: fs })
      } catch { /* surface via existing error path on next render */ }
    },
    createCollection: async (s) => {
      const id = (s.newColDraftId as string).trim()
      if (!id) { s.drawerError = 'Collection ID is required'; return }
      try {
        await (s.createCol as (a: { id: string; name?: string; fields?: Record<string, unknown> }) => Promise<unknown>)({
          id,
          name: (s.newColDraftName as string).trim() || id,
          fields: {},
        })
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : 'Request failed'
        return
      }
      s.newColOpen = false
      s.newColDraftId = ''
      s.newColDraftName = ''
      s.drawerError = ''
      ;(s as any).navigate('/collections?collection=' + id)
    },
  },
})
