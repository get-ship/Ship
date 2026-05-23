import { controller, signal } from '@ship/sui-runtime'
import type { MutableState, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// The scope-form `on:` handler receives the broader scope with `state`
// attached. `navigate` is injected by the runtime mount context but is
// not on the scope type, so read it through a narrowed view.
type HandlerScope = ViewScope<Scope> & { state: MutableState<Scope> }
type Nav = { navigate: (path: string) => void }

export default controller<Scope>({
  scope: () => {
    const newColOpen = signal(false)
    const newColId = signal('')
    const newColName = signal('')
    const drawerError = signal('')
    return { newColOpen, newColId, newColName, drawerError }
  },
  on: {
    createCollection: async (scope: HandlerScope) => {
      const { state } = scope
      const navigate = (scope as unknown as Nav).navigate
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
      navigate('/studio/collections?collection=' + id)
    },
  },
})
