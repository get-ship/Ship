import { controller, signal, derived, live, unwrap, defineRoutes } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

defineRoutes([{ pattern: '/workspaces/:id' }])

interface WorkspaceRow {
  id: string
  name: string
  path: string
  label?: string
  active?: boolean
}

interface WorkspacesResponse {
  items: WorkspaceRow[]
  active: string | null
}

export default controller<Scope>({
  scope: ({ ship, route }) => {
    const wsId = derived(() => route.params.id() ?? '')
    const activatingId = signal<string>('')
    const activationError = signal<string>('')
    const pendingActiveId = signal<string>('')

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const projects = live(() => s.workspaces.list())

    async function activate(): Promise<void> {
      const id = wsId()
      if (!id) return
      activatingId.set(id)
      activationError.set('')
      pendingActiveId.set(id)
      try {
        await s.workspaces.activate({ id })
      } catch (e) {
        pendingActiveId.set('')
        activationError.set((e as Error).message)
      } finally {
        activatingId.set('')
      }
    }

    return {
      wsId,
      projects,
      activatingId,
      activationError,
      pendingActiveId,
      activate,
    }
  },
  derived: {
    workspace: ({ projects, wsId }): WorkspaceRow | null => {
      const resp = unwrap(projects) as WorkspacesResponse | undefined
      const id = wsId()
      if (!resp || !id) return null
      return resp.items.find((r) => r.id === id) ?? null
    },
    notFound: ({ projects, wsId }): boolean => {
      const resp = unwrap(projects) as WorkspacesResponse | undefined
      const id = wsId()
      // Only treat as not-found once we have a payload and id but no match.
      if (!resp || !id) return false
      return !resp.items.some((r) => r.id === id)
    },
    isActive: ({ projects, wsId, pendingActiveId }): boolean => {
      const resp = unwrap(projects) as WorkspacesResponse | undefined
      const id = wsId()
      if (!id) return false
      const live = resp?.active ?? ''
      const pending = pendingActiveId()
      const effective = pending && pending !== live ? pending : live
      return effective === id
    },
  },
})
