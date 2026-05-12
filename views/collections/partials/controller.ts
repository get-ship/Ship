import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const newColOpen = signal(false)
    const newColId = signal('')
    const newColName = signal('')
    const drawerError = signal('')
    return { newColOpen, newColId, newColName, drawerError }
  },
  on: {
    createCollection: async ({ state, navigate }) => {
      const id = (state.newColId as string).trim()
      if (!id) { state.drawerError = 'Collection ID is required'; return }
      const res = await fetch('/api/v1/collections', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ id, name: (state.newColName as string).trim() || id, fields: {} }),
      })
      if (!res.ok) {
        const txt = await res.text().catch(() => 'Request failed')
        state.drawerError = txt
        return
      }
      state.newColOpen = false
      state.newColId = ''
      state.newColName = ''
      state.drawerError = ''
      navigate('/collections?collection=' + id)
    },
  },
})
