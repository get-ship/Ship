import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type Nav = { navigate: (path: string) => void }

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const newColOpen = signal(false)
    const newColId = signal('')
    const newColName = signal('')
    const drawerError = signal('')
    return { newColOpen, newColId, newColName, drawerError }
  },
  on: {
    createCollection: async (state) => {
      const navigate = (state as unknown as Nav).navigate
      const id = state.newColId.trim()
      if (!id) { state.drawerError = 'Collection ID is required'; return }
      const res = await fetch('/api/v1/collections', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ id, name: state.newColName.trim() || id, fields: {} }),
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
      navigate('/studio/collections?collection=' + id)
    },
  },
})
